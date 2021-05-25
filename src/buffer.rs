use std::ffi::CString;
use std::sync::Arc;

use ash::version::DeviceV1_0;
use ash::version::DeviceV1_2;
use ash::vk::{self, Handle};

use crate::device::Device;

pub struct Buffer {
    device: Device,
    handle: vk::Buffer,
    // allocation: gpu_allocator::SubAllocation,
    // device_address: vk::DeviceAddress,
    size: usize,
    location: gpu_allocator::MemoryLocation,
}

impl std::fmt::Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Buffer")
            .field("handle", &self.handle)
            .field("size", &self.size)
            .finish()
    }
}

impl Buffer {
    pub fn new<I>(
        name: Option<&str>,
        device: Device,
        size: I,
        buffer_usage: vk::BufferUsageFlags,
        location: gpu_allocator::MemoryLocation,
    ) -> Self
    where
        I: num_traits::PrimInt,
    {
        unsafe {
            let handle = device
                .inner
                .handle
                .create_buffer(
                    &vk::BufferCreateInfo::builder()
                        .size(size.to_u64().unwrap())
                        .usage(buffer_usage)
                        .sharing_mode(vk::SharingMode::EXCLUSIVE),
                    None,
                )
                .unwrap();
            // let allocation = device
            //     .inner
            //     .allocator
            //     .lock()
            //     .unwrap()
            //     .allocate(&gpu_allocator::AllocationCreateDesc {
            //         name: name.unwrap_or("default"),
            //         requirements: device.inner.handle.get_buffer_memory_requirements(handle),
            //         location: location,
            //         linear: true,
            //     })
            //     .unwrap();
            // device
            //     .inner
            //     .allocator
            //     .lock()
            //     .unwrap()
            //     .free(allocation)
            //     .unwrap();

            // device
            //     .handle
            //     .bind_buffer_memory(handle, allocation.memory(), allocation.offset())
            //     .unwrap();
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
                            .object_type(vk::ObjectType::BUFFER)
                            .object_name(CString::new(name).unwrap().as_ref())
                            .build(),
                    )
                    .unwrap();
            }
            // let device_address = device.handle.get_buffer_device_address(
            //     &vk::BufferDeviceAddressInfo::builder()
            //         .buffer(handle)
            //         .build(),
            // );

            Self {
                device,
                handle,
                // allocation,
                // device_address,
                size: size.to_usize().unwrap(),
                location,
            }
        }
    }

    // pub fn new_init_host<I: AsRef<[u8]>>(
    //     name: Option<&str>,
    //     device: Arc<Device>,
    //     buffer_usage: vk::BufferUsageFlags,
    //     location: gpu_allocator::MemoryLocation,
    //     data: I,
    // ) -> Self {
    //     let data = data.as_ref();
    //     let mut buffer = Self::new(
    //         name,
    //         device,
    //         data.len(),
    //         buffer_usage
    //             | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
    //             | vk::BufferUsageFlags::TRANSFER_DST,
    //         location,
    //     );
    //     let mapped = buffer.mapped_slice_mut().unwrap();
    //     mapped.copy_from_slice(data.as_ref());
    //     buffer
    // }

    // pub fn new_init_device<I: AsRef<[u8]>>(
    //     name: Option<&str>,
    //     device: Arc<Device>,
    //     buffer_usage: vk::BufferUsageFlags,
    //     memory_usage: vk_mem::MemoryUsage,
    //     queue: &mut Queue,
    //     command_pool: Arc<CommandPool>,
    //     data: I,
    // ) -> Self {
    //     let data = data.as_ref();
    //     let buffer = Self::new(
    //         name,
    //         allocator.clone(),
    //         data.len(),
    //         buffer_usage
    //             | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
    //             | vk::BufferUsageFlags::TRANSFER_DST,
    //         memory_usage,
    //     );
    //     if !buffer.is_mappable() {
    //         let staging_buffer = Arc::new(Self::new(
    //             Some("staging buffer"),
    //             allocator.clone(),
    //             data.len(),
    //             vk::BufferUsageFlags::TRANSFER_SRC,
    //             vk_mem::MemoryUsage::CpuToGpu,
    //         ));
    //         staging_buffer.copy_from(data);
    //         let mut cmd_buf = CommandBuffer::new(command_pool);
    //         cmd_buf.encode(|manager| unsafe {
    //             manager.copy_buffer_raw(
    //                 &staging_buffer,
    //                 &buffer,
    //                 &[vk::BufferCopy::builder().size(data.len() as u64).build()],
    //             );
    //         });
    //         let timeline_semaphore = TimelineSemaphore::new(allocator.device.clone());
    //         queue.submit_timeline(
    //             cmd_buf,
    //             &[&timeline_semaphore],
    //             &[0],
    //             &[vk::PipelineStageFlags::ALL_COMMANDS],
    //             &[1],
    //         );
    //         timeline_semaphore.wait_for(1);
    //     } else {
    //         buffer.copy_from(data);
    //         buffer.flush();
    //     }
    //     buffer
    // }

    // pub fn mapped_slice(&self) -> Option<&[u8]> {
    //     self.allocation.mapped_slice()
    // }

    // pub fn mapped_slice_mut(&mut self) -> Option<&mut [u8]> {
    //     self.allocation.mapped_slice_mut()
    // }

    // pub fn memory_type(&self) -> u32 {
    //     self.allocation_info.get_memory_type()
    // }

    // pub fn device_address(&self) -> vk::DeviceAddress {
    //     self.device_address
    // }

    // pub fn copy_from<I: AsRef<[u8]>>(&mut self, data: I) {
    //     let data = data.as_ref();
    //     let mapped = self.mapped_slice_mut().unwrap();
    //     mapped.copy_from_slice(data);
    // }

    pub fn size(&self) -> usize {
        self.size
    }

    // pub fn is_device_local(&self) -> bool {
    //     self.property_flags & vk::MemoryPropertyFlags::DEVICE_LOCAL
    //         != vk::MemoryPropertyFlags::empty()
    // }

    // pub fn is_mappable(&self) -> bool {
    //     self.property_flags & vk::MemoryPropertyFlags::HOST_VISIBLE
    //         != vk::MemoryPropertyFlags::empty()
    // }

    // pub fn flush(&self) {
    //     self.device
    //         .allocator
    //         .flush_allocation(&self.allocation, 0, vk::WHOLE_SIZE as usize);
    // }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        // self.device
        //     .allocator
        //     .lock()
        //     .unwrap()
        //     .free(self.allocation.to_owned())
        //     .unwrap();
    }
}

#[test]
fn test_create_buffer() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();
    use crate::entry::Entry;

    let entry = Entry::new().unwrap();
    let instance = entry.create_instance(&[], &[]);
    let _pdevices = instance.enumerate_physical_device();

    let pdevice = instance
        .enumerate_physical_device()
        .into_iter()
        .find(|p| {
            p.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
                && p.queue_families
                    .iter()
                    .any(|f| f.support_compute() && f.support_graphics())
        })
        .unwrap();
    let pdevice = Arc::new(pdevice);
    let queue_family = pdevice
        .queue_families()
        .iter()
        .find(|f| f.support_graphics() && f.support_compute())
        .unwrap();
    let device = pdevice.create_device(&[(&queue_family, &[1.0])]);
    let _buffer = device.create_buffer(
        None,
        512,
        vk::BufferUsageFlags::empty(),
        gpu_allocator::MemoryLocation::CpuToGpu,
    );
    // let a = _buffer.mapped_slice();
}
