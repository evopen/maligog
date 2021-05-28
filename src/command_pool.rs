use ash::vk;

use crate::device::Device;

pub(crate) struct CommandPool {
    pub(crate) handle: vk::CommandPool,
    device: Device,
}

impl CommandPool {
    pub fn new(device: Device, queue_family_index: u32) -> Self {
        unsafe {
            let handle = device
                .inner
                .handle
                .create_command_pool(
                    &vk::CommandPoolCreateInfo::builder()
                        .queue_family_index(queue_family_index)
                        .build(),
                    None,
                )
                .unwrap();
            Self { handle, device }
        }
    }

    pub fn allocate_command_buffer(&self) {
        unsafe {
            let command_buffers = self
                .device
                .inner
                .handle
                .allocate_command_buffers(&vk::CommandBufferAllocateInfo::builder().build())
                .unwrap();
        }
    }
}
