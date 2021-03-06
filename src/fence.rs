use std::ffi::CString;
use std::sync::Arc;

use ash::vk::{self, Handle};

use crate::Device;

pub struct FenceRef {
    pub(crate) handle: vk::Fence,
    device: Device,
}

pub struct Fence {
    inner: Arc<FenceRef>,
}

impl Fence {
    pub fn new(device: Device, name: Option<&str>, signaled: bool) -> Self {
        unsafe {
            let handle = device
                .inner
                .handle
                .create_fence(
                    &vk::FenceCreateInfo::builder()
                        .flags(match signaled {
                            true => vk::FenceCreateFlags::SIGNALED,
                            false => vk::FenceCreateFlags::empty(),
                        })
                        .build(),
                    None,
                )
                .unwrap();

            if let Some(name) = name {
                device.debug_set_object_name(name, handle.as_raw(), vk::ObjectType::FENCE);
            }
            Self {
                inner: Arc::new(FenceRef { handle, device }),
            }
        }
    }

    pub fn wait(&self) {
        unsafe {
            self.inner
                .device
                .inner
                .handle
                .wait_for_fences(&[self.inner.handle], true, std::u64::MAX)
                .unwrap();
        }
    }

    pub fn reset(&self) {
        unsafe {
            self.inner
                .device
                .inner
                .handle
                .reset_fences(&[self.inner.handle])
                .unwrap();
        }
    }

    pub fn get_status(&self) -> bool {
        unsafe {
            self.inner
                .device
                .handle()
                .get_fence_status(self.inner.handle)
                .unwrap()
        }
    }
}

impl Drop for FenceRef {
    fn drop(&mut self) {
        unsafe { self.device.inner.handle.destroy_fence(self.handle, None) };
    }
}
