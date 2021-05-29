use ash::vk;

use crate::Device;

pub struct Fence {
    pub(crate) handle: vk::Fence,
    device: Device,
}

impl Fence {
    pub fn new(device: Device, signaled: bool) -> Self {
        let handle = unsafe {
            device.inner.handle.create_fence(
                &vk::FenceCreateInfo::builder()
                    .flags(match signaled {
                        true => vk::FenceCreateFlags::SIGNALED,
                        false => vk::FenceCreateFlags::empty(),
                    })
                    .build(),
                None,
            )
        }
        .unwrap();
        Self { handle, device }
    }

    pub fn wait(&self) {
        unsafe {
            self.device
                .inner
                .handle
                .wait_for_fences(&[self.handle], true, std::u64::MAX)
                .unwrap();
        }
    }

    pub fn reset(&self) {
        unsafe {
            self.device
                .inner
                .handle
                .reset_fences(&[self.handle])
                .unwrap();
        }
    }

    pub fn get_status(&self) -> bool {
        unsafe { self.device.handle().get_fence_status(self.handle).unwrap() }
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe { self.device.inner.handle.destroy_fence(self.handle, None) };
    }
}
