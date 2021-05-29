use std::sync::Arc;

use ash::vk;

use crate::device::Device;
use crate::queue_family::QueueFamilyProperties;
use crate::CommandBuffer;

pub(crate) struct QueueRef {
    handle: vk::Queue,
    queue_family_properties: QueueFamilyProperties,
    device: Device,
    command_buffers: Vec<CommandBuffer>,
}

#[derive(Clone)]
pub struct Queue {
    pub(crate) inner: Arc<QueueRef>,
}

impl Queue {
    pub(crate) fn new(
        device: &Device,
        queue_family_properties: &QueueFamilyProperties,
        queue_index: u32,
    ) -> Self {
        unsafe {
            let handle = device
                .inner
                .handle
                .get_device_queue(queue_family_properties.index, queue_index);
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
                .handle()
                .create_fence(&vk::FenceCreateInfo::builder().build(), None)
                .unwrap();
            self.inner
                .device
                .handle()
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
                .handle()
                .wait_for_fences(&[fence_handle], true, std::u64::MAX);
        }
    }
}
