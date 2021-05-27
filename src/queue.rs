use std::sync::Arc;

use ash::vk;

use crate::device::Device;
use crate::queue_family::QueueFamilyProperties;

pub(crate) struct QueueRef {
    handle: vk::Queue,
    queue_family_properties: QueueFamilyProperties,
    device: Device,
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
                }),
            }
        }
    }
}
