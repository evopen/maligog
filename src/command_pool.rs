use std::sync::Arc;

use ash::vk;

use crate::device::Device;

pub(crate) struct CommandPoolRef {
    pub(crate) handle: vk::CommandPool,
    device_handle: ash::Device,
}

#[derive(Clone)]
pub(crate) struct CommandPool {
    pub(crate) inner: Arc<CommandPoolRef>,
}

impl std::fmt::Debug for CommandPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandPool")
            .field("handle", &self.inner.handle)
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
            Self {
                inner: Arc::new(CommandPoolRef {
                    handle,
                    device_handle: device.handle().clone(),
                }),
            }
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

impl Drop for CommandPoolRef {
    fn drop(&mut self) {
        unsafe {
            log::debug!("destroying command pool");
            self.device_handle.destroy_command_pool(self.handle, None);
        }
    }
}
