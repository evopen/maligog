use std::ffi::CString;
use std::sync::Arc;

use ash::version::DeviceV1_2;
use ash::vk::{self, Handle};

use crate::device::Device;

pub struct Buffer {
    device: Arc<Device>,
    handle: vk::Buffer,
    allocation: vk_mem::Allocation,
    mapped: std::sync::atomic::AtomicBool,
    device_address: vk::DeviceAddress,
    size: usize,
    allocation_info: vk_mem::AllocationInfo,
    property_flags: vk::MemoryPropertyFlags,
}

impl std::fmt::Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Buffer")
            .field("handle", &self.handle)
            .field("size", &self.size)
            .field("mapped", &self.mapped)
            .finish()
    }
}

impl Buffer {
    pub fn new<I>(
        name: Option<&str>,
        device: Arc<Device>,
        size: I,
        buffer_usage: vk::BufferUsageFlags,
        memory_usage: vk_mem::MemoryUsage,
    ) -> Self
    where
        I: num_traits::PrimInt,
    {
        let (handle, allocation, allocation_info) = device
            .allocator
            .create_buffer(
                &vk::BufferCreateInfo::builder()
                    .usage(
                        buffer_usage
                            | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
                            | vk::BufferUsageFlags::all(),
                    )
                    .size(size.to_u64().unwrap())
                    .build(),
                &vk_mem::AllocationCreateInfo {
                    usage: memory_usage,
                    ..Default::default()
                },
            )
            .unwrap();

        unsafe {
            if let Some(name) = name {
                device
                    .pdevice
                    .instance
                    .debug_utils_loader
                    .as_ref()
                    .unwrap()
                    .debug_utils_set_object_name(
                        device.handle.handle(),
                        &vk::DebugUtilsObjectNameInfoEXT::builder()
                            .object_handle(handle.as_raw())
                            .object_type(vk::ObjectType::BUFFER)
                            .object_name(CString::new(name).unwrap().as_ref())
                            .build(),
                    )
                    .unwrap();
            }
            let device_address = device.handle.get_buffer_device_address(
                &vk::BufferDeviceAddressInfo::builder()
                    .buffer(handle)
                    .build(),
            );

            let property_flags = device
                .allocator
                .get_memory_type_properties(allocation_info.get_memory_type())
                .unwrap();

            Self {
                device,
                handle,
                allocation,
                mapped: std::sync::atomic::AtomicBool::new(false),
                device_address,
                size: size.to_usize().unwrap(),
                allocation_info,
                property_flags,
            }
        }
    }

    pub fn new_init_host<I: AsRef<[u8]>>(
        name: Option<&str>,
        device: Arc<Device>,
        buffer_usage: vk::BufferUsageFlags,
        memory_usage: vk_mem::MemoryUsage,
        data: I,
    ) -> Self {
        let data = data.as_ref();
        let buffer = Self::new(
            name,
            device,
            data.len(),
            buffer_usage
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | vk::BufferUsageFlags::TRANSFER_DST,
            memory_usage,
        );
        let mapped = buffer.map();
        let mapped_slice = unsafe { std::slice::from_raw_parts_mut(mapped, buffer.size) };
        mapped_slice.copy_from_slice(data.as_ref());
        buffer.unmap();
        buffer
    }

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

    pub fn map(&self) -> *mut u8 {
        if !self.is_mappable() {
            panic!("memory is not host visible");
        }

        let ptr = self.device.allocator.map_memory(&self.allocation).unwrap();
        self.mapped
            .compare_exchange(
                false,
                true,
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
            )
            .expect("already mapped");
        ptr
    }

    pub fn unmap(&self) {
        self.mapped
            .compare_exchange(
                true,
                false,
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
            )
            .expect("not mapped");
        self.device.allocator.unmap_memory(&self.allocation);
    }

    pub fn memory_type(&self) -> u32 {
        self.allocation_info.get_memory_type()
    }

    pub fn device_address(&self) -> vk::DeviceAddress {
        self.device_address
    }

    pub fn copy_from<I: AsRef<[u8]>>(&self, data: I) {
        let data = data.as_ref();
        let mapped = self.map();
        let mapped_bytes = unsafe { std::slice::from_raw_parts_mut(mapped, self.size) };
        mapped_bytes.copy_from_slice(data);
        self.unmap();
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn is_device_local(&self) -> bool {
        self.property_flags & vk::MemoryPropertyFlags::DEVICE_LOCAL
            != vk::MemoryPropertyFlags::empty()
    }

    pub fn is_mappable(&self) -> bool {
        self.property_flags & vk::MemoryPropertyFlags::HOST_VISIBLE
            != vk::MemoryPropertyFlags::empty()
    }

    pub fn flush(&self) {
        self.device
            .allocator
            .flush_allocation(&self.allocation, 0, vk::WHOLE_SIZE as usize);
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        if self.mapped.load(std::sync::atomic::Ordering::SeqCst) {
            self.unmap();
        }
        self.device
            .allocator
            .destroy_buffer(self.handle, &self.allocation);
    }
}

#[test]
fn test_create_buffer() {
    use crate::entry::Entry;

    let entry = Entry::new().unwrap();
    let instance = entry.create_instance(&[], &[]);
    let pdevices = instance.enumerate_physical_device();
    dbg!(&pdevices);

    let pdevice = instance
        .enumerate_physical_device()
        .into_iter()
        .find(|p| {
            p.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
                && p.queue_families
                    .iter()
                    .find(|f| f.support_compute() && f.support_graphics())
                    .is_some()
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
        100,
        vk::BufferUsageFlags::STORAGE_BUFFER,
        vk_mem::MemoryUsage::GpuOnly,
    );
}
