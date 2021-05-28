use std::sync::Arc;

use ash::vk;

use crate::Device;

pub struct TimelineSemaphoreRef {
    handle: vk::Semaphore,
    device: Device,
}

pub struct TimelineSemaphore {
    inner: Arc<TimelineSemaphoreRef>,
}

impl TimelineSemaphore {
    pub fn new(device: &Device) -> Self {
        unsafe {
            let handle = device
                .inner
                .handle
                .create_semaphore(
                    &vk::SemaphoreCreateInfo::builder()
                        .push_next(
                            &mut vk::SemaphoreTypeCreateInfo::builder()
                                .semaphore_type(vk::SemaphoreType::TIMELINE)
                                .initial_value(0)
                                .build(),
                        )
                        .build(),
                    None,
                )
                .unwrap();
            Self {
                inner: Arc::new(TimelineSemaphoreRef {
                    handle,
                    device: device.clone(),
                }),
            }
        }
    }

    pub fn wait_for(&self, value: u64) {
        unsafe {
            self.inner
                .device
                .inner
                .handle
                .wait_semaphores(
                    &vk::SemaphoreWaitInfo::builder()
                        .semaphores(&[self.inner.handle])
                        .values(&[value])
                        .build(),
                    std::u64::MAX,
                )
                .unwrap();
        }
    }

    pub fn signal(&self, value: u64) {
        unsafe {
            self.inner
                .device
                .inner
                .handle
                .signal_semaphore(
                    &vk::SemaphoreSignalInfo::builder()
                        .semaphore(self.inner.handle)
                        .value(value)
                        .build(),
                )
                .unwrap();
        }
    }
}

impl Drop for TimelineSemaphoreRef {
    fn drop(&mut self) {
        unsafe {
            self.device
                .inner
                .handle
                .destroy_semaphore(self.handle, None);
        }
    }
}

pub struct BinarySemaphoreRef {
    pub(crate) handle: vk::Semaphore,
    device: Device,
}

#[derive(Clone)]
pub struct BinarySemaphore {
    pub(crate) inner: Arc<BinarySemaphoreRef>,
}

impl BinarySemaphore {
    pub fn new(device: &Device) -> Self {
        unsafe {
            let handle = device
                .inner
                .handle
                .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                .unwrap();
            Self {
                inner: Arc::new(BinarySemaphoreRef {
                    handle,
                    device: device.clone(),
                }),
            }
        }
    }
}

impl Drop for BinarySemaphoreRef {
    fn drop(&mut self) {
        unsafe {
            self.device
                .inner
                .handle
                .destroy_semaphore(self.handle, None);
        }
    }
}
