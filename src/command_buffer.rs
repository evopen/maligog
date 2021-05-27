use ash::vk;

use crate::device::Device;

struct CommandBuffer {
    device: Device,
    handle: vk::CommandBuffer,
}

impl CommandBuffer {
    pub fn new(device: Device) -> Self {
        unsafe {
            let handle = device
                .inner
                .handle
                .allocate_command_buffers(
                    &vk::CommandBufferAllocateInfo::builder()
                        .command_pool(device.command_pool().handle)
                        .command_buffer_count(1)
                        .level(vk::CommandBufferLevel::PRIMARY)
                        .build(),
                )
                .unwrap()
                .first()
                .unwrap()
                .to_owned();

            Self { handle, device }
        }
    }

    pub fn begin(&self) {
        unsafe {
            self.device
                .inner
                .handle
                .begin_command_buffer(self.handle, &vk::CommandBufferBeginInfo::default())
                .unwrap();
        }
    }

    pub fn end(&self) {
        unsafe {
            self.device
                .inner
                .handle
                .end_command_buffer(self.handle)
                .unwrap();
        }
    }
}

#[test]
fn test_command_buffer() {
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
    let pdevice = pdevice;
    let queue_family = pdevice
        .queue_families()
        .iter()
        .find(|f| f.support_graphics() && f.support_compute())
        .unwrap();
    let (device, _) = pdevice.create_device(&[(&queue_family, &[1.0])]);
    device.allocate_command_buffer();
}
