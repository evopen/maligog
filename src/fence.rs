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
                device
                    .inner
                    .pdevice
                    .instance
                    .inner
                    .debug_utils_loader
                    .as_ref()
                    .unwrap()
                    .debug_utils_set_object_name(
                        device.inner.handle.handle(),
                        &vk::DebugUtilsObjectNameInfoEXT::builder()
                            .object_handle(handle.as_raw())
                            .object_type(vk::ObjectType::FENCE)
                            .object_name(CString::new(name).unwrap().as_ref())
                            .build(),
                    )
                    .unwrap();
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
