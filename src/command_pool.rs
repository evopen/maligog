use std::sync::Arc;

use ash::vk;

use crate::device::Device;

#[derive(Clone)]
pub(crate) struct CommandPool {
    pub(crate) handle: vk::CommandPool,
    device: Device,
}

impl std::fmt::Debug for CommandPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandPool")
            .field("handle", &self.handle)
            .finish()
    }
}

impl CommandPool {
    pub fn new(device: Device, queue_family_index: u32) -> Self {
        log::debug!("creating command pool");
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

    // pub fn allocate_command_buffer(&self) {
    //     unsafe {
    //         let command_buffers = self
    //             .device
    //             .inner
    //             .handle
    //             .allocate_command_buffers(&vk::CommandBufferAllocateInfo::builder().build())
    //             .unwrap();
    //     }
    // }
}

// impl Drop for CommandPool {
//     fn drop(&mut self) {
//         unsafe {
//             log::debug!("destroying command pool");
//             self.device.handle().destroy_command_pool(self.handle, None);
//         }
//     }
// }
