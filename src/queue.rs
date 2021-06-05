use std::sync::Arc;
use std::sync::Mutex;

use ash::vk;

use crate::device::Device;
use crate::queue_family::QueueFamilyProperties;
use crate::CommandBuffer;

pub(crate) struct QueueRef {
    pub(crate) handle: vk::Queue,
    pub(crate) queue_family_properties: QueueFamilyProperties,
    device: ash::Device,
    command_buffers: Vec<CommandBuffer>,
    synchronization2_loader: ash::extensions::khr::Synchronization2,
    lock: Mutex<()>,
}

pub struct Queue {
    pub(crate) inner: Arc<QueueRef>,
}

impl Queue {
    pub(crate) fn new(
        device: &ash::Device,
        synchronization2_loader: ash::extensions::khr::Synchronization2,
        queue_family_properties: &QueueFamilyProperties,
        queue_index: u32,
    ) -> Self {
        unsafe {
            let handle = device.get_device_queue(queue_family_properties.index, queue_index);
            Self {
                inner: Arc::new(QueueRef {
                    handle,
                    synchronization2_loader,
                    queue_family_properties: queue_family_properties.clone(),
                    device: device.clone(),
                    command_buffers: vec![],
                    lock: Mutex::new(()),
                }),
            }
        }
    }

    pub fn submit_blocking(&self, command_buffers: &[CommandBuffer]) {
        unsafe {
            let command_buffer_submit_infos = command_buffers
                .iter()
                .map(|cmd_buf| {
                    vk::CommandBufferSubmitInfoKHR::builder()
                        .command_buffer(cmd_buf.handle)
                        .build()
                })
                .collect::<Vec<_>>();

            let fence_handle = self
                .inner
                .device
                .create_fence(&vk::FenceCreateInfo::builder().build(), None)
                .unwrap();

            let lock = self.inner.lock.lock().unwrap();
            self.inner
                .synchronization2_loader
                .queue_submit2(
                    self.inner.handle,
                    &[vk::SubmitInfo2KHR::builder()
                        .command_buffer_infos(&command_buffer_submit_infos)
                        .build()],
                    fence_handle,
                )
                .unwrap();
            drop(lock);

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
