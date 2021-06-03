use std::sync::Arc;

use ash::vk;

use crate::device::Device;
use crate::queue_family::QueueFamilyProperties;
use crate::CommandBuffer;

pub(crate) struct QueueRef {
    pub(crate) handle: vk::Queue,
    pub(crate) queue_family_properties: QueueFamilyProperties,
    device: ash::Device,
    command_buffers: Vec<CommandBuffer>,
}

pub struct Queue {
    pub(crate) inner: Arc<QueueRef>,
}

impl Queue {
    pub(crate) fn new(
        device: &ash::Device,
        queue_family_properties: &QueueFamilyProperties,
        queue_index: u32,
    ) -> Self {
        unsafe {
            let handle = device.get_device_queue(queue_family_properties.index, queue_index);
            Self {
                inner: Arc::new(QueueRef {
                    handle,
                    queue_family_properties: queue_family_properties.clone(),
                    device: device.clone(),
                    command_buffers: vec![],
                }),
            }
        }
    }

    pub fn submit_blocking(&self, command_buffers: &[CommandBuffer]) {
        unsafe {
            let command_buffer_handles = command_buffers
                .iter()
                .map(|cmd_buf| cmd_buf.handle)
                .collect::<Vec<_>>();

            let fence_handle = self
                .inner
                .device
                .create_fence(&vk::FenceCreateInfo::builder().build(), None)
                .unwrap();
            self.inner
                .device
                .queue_submit(
                    self.inner.handle,
                    &[vk::SubmitInfo::builder()
                        .command_buffers(&command_buffer_handles)
                        .build()],
                    fence_handle,
                )
                .unwrap();
            self.inner
                .device
                .wait_for_fences(&[fence_handle], true, std::u64::MAX);
            self.inner.device.destroy_fence(fence_handle, None);
        }
    }
}

impl Drop for QueueRef {
    fn drop(&mut self) {
        log::debug!("dropping queue and its command buffers");
    }
}
