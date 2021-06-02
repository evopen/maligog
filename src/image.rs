use std::ffi::CString;
use std::sync::Arc;
use std::sync::LockResult;
use std::sync::Mutex;
use std::sync::MutexGuard;

use ash::vk;
use ash::vk::Handle;

use crate::buffer::Buffer;
use crate::Device;
use crate::Swapchain;
use crate::TimelineSemaphore;

enum ImageType {
    Allocated {
        allocation: Mutex<gpu_allocator::SubAllocation>,
    },
    FromHandle,
}

pub struct ImageRef {
    pub(crate) handle: vk::Image,
    pub(crate) device: Device,
    image_type: ImageType,
    width: u32,
    height: u32,
    layout: std::sync::atomic::AtomicI32,
    format: vk::Format,
}

#[derive(Clone)]
pub struct Image {
    pub(crate) inner: Arc<ImageRef>,
}

impl Image {
    pub fn new(
        name: Option<&str>,
        device: &Device,
        format: vk::Format,
        width: u32,
        height: u32,
        image_usage: vk::ImageUsageFlags,
        location: gpu_allocator::MemoryLocation,
    ) -> Self {
        unsafe {
            let handle = device
                .inner
                .handle
                .create_image(
                    &vk::ImageCreateInfo::builder()
                        .image_type(vk::ImageType::TYPE_2D)
                        .format(format)
                        .extent(vk::Extent3D {
                            width,
                            height,
                            depth: 1,
                        })
                        .samples(vk::SampleCountFlags::TYPE_1)
                        .mip_levels(1)
                        .array_layers(1)
                        .tiling(match location {
                            gpu_allocator::MemoryLocation::Unknown => {
                                unimplemented!()
                            }
                            gpu_allocator::MemoryLocation::GpuOnly => vk::ImageTiling::OPTIMAL,
                            gpu_allocator::MemoryLocation::CpuToGpu => vk::ImageTiling::LINEAR,
                            gpu_allocator::MemoryLocation::GpuToCpu => vk::ImageTiling::LINEAR,
                        })
                        .usage(image_usage)
                        .sharing_mode(vk::SharingMode::CONCURRENT)
                        .queue_family_indices(device.all_queue_family_indices())
                        .initial_layout(vk::ImageLayout::UNDEFINED)
                        .build(),
                    None,
                )
                .unwrap();
            let allocation = device
                .inner
                .allocator
                .lock()
                .unwrap()
                .allocate(&gpu_allocator::AllocationCreateDesc {
                    name: name.unwrap_or("default"),
                    requirements: device.inner.handle.get_image_memory_requirements(handle),
                    location: location,
                    linear: match location {
                        gpu_allocator::MemoryLocation::Unknown => {
                            unimplemented!()
                        }
                        gpu_allocator::MemoryLocation::GpuOnly => true,
                        gpu_allocator::MemoryLocation::CpuToGpu => false,
                        gpu_allocator::MemoryLocation::GpuToCpu => false,
                    },
                })
                .unwrap();

            device
                .inner
                .handle
                .bind_image_memory(handle, allocation.memory(), allocation.offset())
                .unwrap();
            if let Some(name) = name {
                device
                    .inner
                    .pdevice
                    .instance
                    .inner
                    .debug_utils_loader
                    .as_ref()
                    .unwrap()
                    .debug_utils_set_object_name(
                        device.inner.handle.handle(),
                        &vk::DebugUtilsObjectNameInfoEXT::builder()
                            .object_handle(handle.as_raw())
                            .object_type(vk::ObjectType::IMAGE)
                            .object_name(CString::new(name).unwrap().as_ref())
                            .build(),
                    )
                    .unwrap();
            }

            let image_type = ImageType::Allocated {
                allocation: Mutex::new(allocation),
            };

            let layout = std::sync::atomic::AtomicI32::new(vk::ImageLayout::UNDEFINED.as_raw());

            Self {
                inner: Arc::new(ImageRef {
                    device: device.clone(),
                    handle,
                    width,
                    height,
                    layout,
                    image_type,
                    format,
                }),
            }
        }
    }

    pub fn new_with_data<I: AsRef<[u8]>>(
        name: Option<&str>,
        device: &Device,
        format: vk::Format,
        width: u32,
        height: u32,
        image_usage: vk::ImageUsageFlags,
        location: gpu_allocator::MemoryLocation,
        data: I,
    ) -> Self {
        let data = data.as_ref();
        let mut image = Self::new(
            name,
            &device,
            format,
            width,
            height,
            image_usage | vk::ImageUsageFlags::TRANSFER_DST,
            location,
        );
        image.set_layout(
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        );
        let mut guard = image.lock_memory().unwrap().unwrap();
        match guard.mapped_slice_mut() {
            Some(mapped) => {
                mapped[0..data.len()].copy_from_slice(data.as_ref());
            }
            None => {
                let staging_buffer = device.create_buffer_init(
                    Some("staging buffer"),
                    data,
                    vk::BufferUsageFlags::TRANSFER_SRC,
                    gpu_allocator::MemoryLocation::CpuToGpu,
                );
                let mut cmd_buf =
                    device.create_command_buffer(device.transfer_queue_family_index());
                cmd_buf.encode(|recorder| {
                    unsafe {
                        recorder.copy_buffer_to_image_raw(
                            &staging_buffer,
                            &image,
                            &[vk::BufferImageCopy::builder()
                                .image_extent(vk::Extent3D {
                                    width: image.width(),
                                    height: image.height(),
                                    depth: 1,
                                })
                                .image_offset(vk::Offset3D::default())
                                .image_subresource(
                                    vk::ImageSubresourceLayers::builder()
                                        .layer_count(1)
                                        .base_array_layer(0)
                                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                                        .mip_level(0)
                                        .build(),
                                )
                                .buffer_offset(0)
                                .buffer_image_height(0)
                                .buffer_row_length(0)
                                .build()],
                        )
                    }
                });
                device.transfer_queue().submit_blocking(&[cmd_buf]);
            }
        }
        drop(guard);
        image
    }

    pub fn lock_memory(&self) -> Option<LockResult<MutexGuard<gpu_allocator::SubAllocation>>> {
        match &self.inner.image_type {
            ImageType::Allocated { allocation } => Some(allocation.lock()),
            ImageType::FromHandle => None,
        }
    }

    pub fn layout(&self) -> vk::ImageLayout {
        vk::ImageLayout::from_raw(self.inner.layout.load(std::sync::atomic::Ordering::SeqCst))
    }

    // pub fn new_with<I: AsRef<[u8]>>(
    //     name: Option<&str>,
    //     device: &Device,
    //     format: vk::Format,
    //     width: u32,
    //     height: u32,
    //     tiling: vk::ImageTiling,
    //     image_usage: vk::ImageUsageFlags,
    //     memory_usage: vk_mem::MemoryUsage,
    //     queue: &mut Queue,
    //     command_pool: Arc<CommandPool>,
    //     data: I,
    // ) -> Self {
    //     let mut image = Self::new(
    //         name,
    //         allocator.clone(),
    //         format,
    //         width,
    //         height,
    //         tiling,
    //         image_usage,
    //         memory_usage,
    //     );
    //     let data = data.as_ref();

    //     let staging_buffer = Buffer::new_with(
    //         Some("staging buffer"),
    //         allocator,
    //         vk::BufferUsageFlags::TRANSFER_SRC,
    //         gpu_allocator::MemoryLocation::CpuToGpu,
    //         data,
    //     );

    //     image.copy_from_buffer(&staging_buffer, queue, command_pool);

    //     image
    // }

    // pub fn copy_from_buffer(&self, buffer: &Buffer, queue: &mut Queue, command_pool: CommandPool) {
    //     let mut command_buffer = CommandBuffer::new(command_pool);

    //     unsafe {
    //         command_buffer.encode(|recorder| {
    //             recorder.set_image_layout_raw(self, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
    //             recorder.copy_buffer_to_image_raw(
    //                 buffer,
    //                 self,
    //                 &[vk::BufferImageCopy::builder()
    //                     .image_extent(vk::Extent3D {
    //                         width: self.width,
    //                         height: self.height,
    //                         depth: 1,
    //                     })
    //                     .image_offset(vk::Offset3D::default())
    //                     .image_subresource(
    //                         vk::ImageSubresourceLayers::builder()
    //                             .layer_count(1)
    //                             .base_array_layer(0)
    //                             .aspect_mask(vk::ImageAspectFlags::COLOR)
    //                             .mip_level(0)
    //                             .build(),
    //                     )
    //                     .buffer_offset(0)
    //                     .buffer_image_height(0)
    //                     .buffer_row_length(0)
    //                     .build()],
    //             );
    //         });
    //     }
    //     self.layout.store(
    //         vk::ImageLayout::TRANSFER_DST_OPTIMAL.as_raw(),
    //         std::sync::atomic::Ordering::SeqCst,
    //     );

    //     let semaphore = TimelineSemaphore::new(&self.inner.device);
    //     queue.submit_timeline(
    //         command_buffer,
    //         &[&semaphore],
    //         &[0],
    //         &[vk::PipelineStageFlags::ALL_COMMANDS],
    //         &[1],
    //     );
    //     semaphore.wait_for(1);
    // }

    // pub fn set_layout(
    //     &mut self,
    //     layout: vk::ImageLayout,
    //     queue: &mut Queue,
    //     command_pool: Arc<CommandPool>,
    // ) {
    //     let mut command_buffer = CommandBuffer::new(command_pool);
    //     unsafe {
    //         command_buffer.encode(|recorder| {
    //             recorder.set_image_layout_raw(self, layout);
    //         });
    //     }
    //     self.layout
    //         .store(layout.as_raw(), std::sync::atomic::Ordering::SeqCst);

    //     let semaphore = TimelineSemaphore::new(&self.inner.device);
    //     queue.submit_timeline(
    //         command_buffer,
    //         &[&semaphore],
    //         &[0],
    //         &[vk::PipelineStageFlags::ALL_COMMANDS],
    //         &[1],
    //     );
    //     semaphore.wait_for(1);
    // }

    pub fn from_handle(
        device: &Device,
        handle: vk::Image,
        width: u32,
        height: u32,
        format: vk::Format,
    ) -> Self {
        Self {
            inner: Arc::new(ImageRef {
                device: device.clone(),
                handle,
                image_type: ImageType::FromHandle,
                width,
                height,
                layout: std::sync::atomic::AtomicI32::new(vk::ImageLayout::UNDEFINED.as_raw()),
                format,
            }),
        }
    }

    // pub fn from_swapchain(swapchain: Swapchain) -> Vec<Self> {
    //     unsafe {
    //         let device = &swapchain.inner.device;
    //         let images = device
    //             .inner
    //             .swapchain_loader
    //             .get_swapchain_images(swapchain.vk_handle())
    //             .unwrap();

    //         let results = images
    //             .into_iter()
    //             .map(|handle| {
    //                 Self {
    //                     inner: Arc::new(ImageRef {
    //                         device: device.clone(),
    //                         handle,
    //                         image_type: ImageType::Swapchain {
    //                             swapchain: swapchain.clone(),
    //                         },
    //                         width: swapchain.width(),
    //                         height: swapchain.height(),
    //                         layout: std::sync::atomic::AtomicI32::new(
    //                             vk::ImageLayout::UNDEFINED.as_raw(),
    //                         ),
    //                         format: swapchain.format(),
    //                     }),
    //                 }
    //             })
    //             .collect::<Vec<_>>();
    //         results.iter().for_each(|image| {
    //             device
    //                 .inner
    //                 .pdevice
    //                 .instance
    //                 .inner
    //                 .debug_utils_loader
    //                 .as_ref()
    //                 .unwrap()
    //                 .debug_utils_set_object_name(
    //                     device.inner.handle.handle(),
    //                     &vk::DebugUtilsObjectNameInfoEXT::builder()
    //                         .object_handle(image.inner.handle.as_raw())
    //                         .object_type(vk::ObjectType::IMAGE)
    //                         .object_name(CString::new("swapchain image").unwrap().as_ref())
    //                         .build(),
    //                 )
    //                 .unwrap();
    //         });

    //         results
    //     }
    // }

    fn device(&self) -> Device {
        self.inner.device.clone()
    }

    // pub fn cmd_set_layout(
    //     &mut self,
    //     command_buffer: &CommandBuffer,
    //     layout: vk::ImageLayout,
    //     need_old_data: bool,
    // ) {
    //     let old_layout = match need_old_data {
    //         true => {
    //             vk::ImageLayout::from_raw(self.layout.load(std::sync::atomic::Ordering::SeqCst))
    //         }
    //         false => vk::ImageLayout::UNDEFINED,
    //     };
    //     cmd_set_image_layout(old_layout, command_buffer, self.handle, layout);
    //     self.layout
    //         .store(layout.as_raw(), std::sync::atomic::Ordering::SeqCst);
    // }

    pub fn format(&self) -> vk::Format {
        self.inner.format
    }

    pub fn width(&self) -> u32 {
        self.inner.width
    }

    pub fn height(&self) -> u32 {
        self.inner.height
    }

    pub(crate) fn handle(&self) -> vk::Image {
        self.inner.handle
    }

    pub fn set_layout(&self, old_layout: vk::ImageLayout, new_layout: vk::ImageLayout) {
        let mut cmd_buf = self
            .device()
            .create_command_buffer(self.device().transfer_queue_family_index());
        let image_memory_barrier = vk::ImageMemoryBarrier2KHR::builder()
            .src_stage_mask(vk::PipelineStageFlags2KHR::ALL_COMMANDS)
            .src_access_mask(vk::AccessFlags2KHR::MEMORY_READ | vk::AccessFlags2KHR::MEMORY_WRITE)
            .dst_stage_mask(vk::PipelineStageFlags2KHR::ALL_COMMANDS)
            .dst_access_mask(vk::AccessFlags2KHR::MEMORY_READ | vk::AccessFlags2KHR::MEMORY_WRITE)
            .old_layout(old_layout)
            .new_layout(new_layout)
            .image(self.handle())
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build(),
            )
            .build();
        cmd_buf.encode(|recorder| {
            recorder.pipeline_barrier(
                &vk::DependencyInfoKHR::builder()
                    .image_memory_barriers(&[image_memory_barrier])
                    .build(),
            )
        });
        self.device().transfer_queue().submit_blocking(&[cmd_buf]);
    }
}

impl Drop for ImageRef {
    fn drop(&mut self) {
        match &self.image_type {
            ImageType::Allocated { allocation, .. } => unsafe {
                self.device
                    .inner
                    .allocator
                    .lock()
                    .unwrap()
                    .free(allocation.lock().unwrap().clone())
                    .unwrap();
                self.device.inner.handle.destroy_image(self.handle, None);
            },
            ImageType::FromHandle => {}
        }
    }
}

impl Device {
    pub fn create_image(
        &self,
        name: Option<&str>,
        format: vk::Format,
        width: u32,
        height: u32,
        image_usage: vk::ImageUsageFlags,
        location: gpu_allocator::MemoryLocation,
    ) -> Image {
        Image::new(name, self, format, width, height, image_usage, location)
    }

    pub fn create_image_init<D>(
        &self,
        name: Option<&str>,
        format: vk::Format,
        width: u32,
        height: u32,
        image_usage: vk::ImageUsageFlags,
        location: gpu_allocator::MemoryLocation,
        data: D,
    ) -> Image
    where
        D: AsRef<[u8]>,
    {
        Image::new_with_data(
            name,
            self,
            format,
            width,
            height,
            image_usage,
            location,
            data,
        )
    }
}
