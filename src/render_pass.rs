use std::sync::Arc;

use ash::vk;

use crate::Device;

pub(crate) struct RenderPassRef {
    pub(crate) handle: vk::RenderPass,
    device: Device,
}

#[derive(Clone)]
pub struct RenderPass {
    pub(crate) inner: Arc<RenderPassRef>,
}

impl RenderPass {
    pub fn new(device: &Device, info: &vk::RenderPassCreateInfo) -> Self {
        unsafe {
            let handle = device.inner.handle.create_render_pass(&info, None).unwrap();
            Self {
                inner: Arc::new(RenderPassRef {
                    handle,
                    device: device.clone(),
                }),
            }
        }
    }

    pub fn handle(&self) -> vk::RenderPass {
        self.inner.handle
    }
}

impl Drop for RenderPassRef {
    fn drop(&mut self) {
        unsafe {
            self.device
                .inner
                .handle
                .destroy_render_pass(self.handle, None);
        }
    }
}

impl Device {
    pub fn create_render_pass(&self, info: &vk::RenderPassCreateInfo) -> RenderPass {
        RenderPass::new(&self, info)
    }
}
