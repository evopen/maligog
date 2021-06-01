use std::sync::Arc;

use ash::vk;

use crate::{Device, ImageView, RenderPass};

pub(crate) struct FramebufferRef {
    pub(crate) device: Device,
    pub(crate) handle: vk::Framebuffer,
    render_pass: RenderPass,
    attachments: Vec<ImageView>,
    width: u32,
    height: u32,
}

pub struct Framebuffer {
    pub(crate) inner: Arc<FramebufferRef>,
}

impl Framebuffer {
    pub fn new(
        device: &Device,
        render_pass: RenderPass,
        width: u32,
        height: u32,
        attachments: Vec<&ImageView>,
    ) -> Self {
        unsafe {
            let attachment_handles = attachments
                .iter()
                .map(|view| view.inner.handle)
                .collect::<Vec<_>>();
            let handle = device
                .handle()
                .create_framebuffer(
                    &vk::FramebufferCreateInfo::builder()
                        .width(width)
                        .height(height)
                        .layers(1)
                        .attachments(attachment_handles.as_slice())
                        .render_pass(render_pass.inner.handle)
                        .build(),
                    None,
                )
                .unwrap();
            let attachments = attachments.into_iter().map(|att| att.to_owned()).collect();
            Self {
                inner: Arc::new(FramebufferRef {
                    device: device.clone(),
                    handle,
                    render_pass,
                    attachments,
                    width,
                    height,
                }),
            }
        }
    }

    pub fn width(&self) -> u32 {
        self.inner.width
    }
    pub fn height(&self) -> u32 {
        self.inner.height
    }
}

impl Drop for FramebufferRef {
    fn drop(&mut self) {
        unsafe {
            self.device.handle().destroy_framebuffer(self.handle, None);
        }
    }
}

impl Device {
    pub fn create_framebuffer(
        &self,
        render_pass: RenderPass,
        width: u32,
        height: u32,
        attachments: Vec<&ImageView>,
    ) -> Framebuffer {
        Framebuffer::new(self, render_pass, width, height, attachments)
    }
}
