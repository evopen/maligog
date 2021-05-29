use std::ffi::CString;
use std::sync::Arc;
use std::sync::LockResult;
use std::sync::Mutex;
use std::sync::MutexGuard;

use ash::vk::{self, Handle};

use crate::device::Device;

pub(crate) struct BufferRef {
    name: Option<String>,
    pub(crate) device: Device,
    pub(crate) handle: vk::Buffer,
    allocation: Mutex<gpu_allocator::SubAllocation>,
    device_address: vk::DeviceAddress,
    size: usize,
    location: gpu_allocator::MemoryLocation,
}

pub struct Buffer {
    pub(crate) inner: Arc<BufferRef>,
}

impl std::fmt::Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Buffer")
            .field("name", &self.inner.name)
            .field("handle", &self.inner.handle)
            .field("size", &self.inner.size)
            .field(
                "device_address",
                &format!("0x{:X?}", &self.inner.device_address),
            )
            .field("memory_location", &self.inner.location)
            .finish()
    }
}

impl Buffer {
    pub fn new<I>(
        name: Option<&str>,
        device: &Device,
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
            let allocation = device
                .inner
                .allocator
                .lock()
                .unwrap()
                .allocate(&gpu_allocator::AllocationCreateDesc {
                    name: name.unwrap_or("default"),
                    requirements: device.inner.handle.get_buffer_memory_requirements(handle),
                    location: location,
                    linear: true,
                })
                .unwrap();

            device
                .inner
                .handle
                .bind_buffer_memory(handle, allocation.memory(), allocation.offset())
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
                            .object_type(vk::ObjectType::BUFFER)
                            .object_name(CString::new(name).unwrap().as_ref())
                            .build(),
                    )
                    .unwrap();
            }
            let device_address = device.inner.handle.get_buffer_device_address(
                &vk::BufferDeviceAddressInfo::builder()
                    .buffer(handle)
                    .build(),
            );

            Self {
                inner: Arc::new(BufferRef {
                    name: name.map(|a| a.to_owned()),
                    device: device.clone(),
                    handle,
                    allocation: Mutex::new(allocation),
                    device_address,
                    size: size.to_usize().unwrap(),
                    location,
                }),
            }
        }
    }

    pub fn new_with_data<I: AsRef<[u8]>>(
        name: Option<&str>,
        device: Device,
        buffer_usage: vk::BufferUsageFlags,
        location: gpu_allocator::MemoryLocation,
        data: I,
    ) -> Self {
        let data = data.as_ref();
        let mut buffer = Self::new(
            name,
            &device,
            data.len(),
            buffer_usage
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | vk::BufferUsageFlags::TRANSFER_DST,
            location,
        );
        let mut guard = buffer.lock_memory().unwrap();
        match guard.mapped_slice_mut() {
            Some(mapped) => {
                mapped[0..data.len()].copy_from_slice(data.as_ref());
            }
            None => {
                let staging_buffer = device.create_buffer_init(
                    Some("staging buffer"),
                    data,
                    vk::BufferUsageFlags::empty(),
                    gpu_allocator::MemoryLocation::CpuToGpu,
                );
                let mut cmd_buf =
                    device.create_command_buffer(device.find_transfer_queue_family_index());
                cmd_buf.encode(|recorder| {
                    unsafe {
                        recorder.copy_buffer_raw(
                            &staging_buffer,
                            &buffer,
                            &[vk::BufferCopy::builder().size(data.len() as u64).build()],
                        )
                    }
                });
                device.transfer_queue().submit_blocking(&[cmd_buf]);
            }
        }
        drop(guard);
        buffer
    }

    pub fn lock_memory(&self) -> LockResult<MutexGuard<gpu_allocator::SubAllocation>> {
        self.inner.allocation.lock()
    }

    // pub fn memory_type(&self) -> u32 {
    //     self.allocation_info.get_memory_type()
    // }

    pub fn device_address(&self) -> vk::DeviceAddress {
        self.inner.device_address
    }

    pub fn copy_from<I: AsRef<[u8]>>(&self, data: I) {
        let data = data.as_ref();
        let mut guard = self.lock_memory().unwrap();
        let mapped = guard.mapped_slice_mut().unwrap();
        mapped.copy_from_slice(data);
    }

    pub fn size(&self) -> usize {
        self.inner.size
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

impl Drop for BufferRef {
    fn drop(&mut self) {
        self.device
            .inner
            .allocator
            .lock()
            .unwrap()
            .free(self.allocation.lock().unwrap().to_owned())
            .unwrap();
        unsafe {
            self.device.inner.handle.destroy_buffer(self.handle, None);
        }
    }
}

impl Device {
    pub fn create_buffer<I>(
        &self,
        name: Option<&str>,
        size: I,
        buffer_usage: vk::BufferUsageFlags,
        location: gpu_allocator::MemoryLocation,
    ) -> Buffer
    where
        I: num_traits::PrimInt,
    {
        Buffer::new(name, self, size, buffer_usage, location)
    }

    pub fn create_buffer_init<D>(
        &self,
        name: Option<&str>,
        data: D,
        buffer_usage: vk::BufferUsageFlags,
        location: gpu_allocator::MemoryLocation,
    ) -> Buffer
    where
        D: AsRef<[u8]>,
    {
        Buffer::new_with_data(name, self.clone(), buffer_usage, location, data)
    }
}

#[test]
fn test_create_buffer() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .try_init()
        .ok();
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
    let buffer = device.create_buffer(
        None,
        123,
        vk::BufferUsageFlags::STORAGE_BUFFER,
        gpu_allocator::MemoryLocation::GpuOnly,
    );
    assert!(buffer.lock_memory().unwrap().mapped_slice() == None);
    dbg!(&buffer.device_address());
}
