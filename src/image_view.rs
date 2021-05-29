use std::sync::Arc;

use ash::vk;

use crate::{Device, Image};

pub struct ImageViewRef {
    pub(crate) device: Device,
    pub(crate) handle: vk::ImageView,
    image: Image,
}

pub struct ImageView {
    inner: Arc<ImageViewRef>,
}

impl ImageView {
    pub fn new(device: &Device, image: &Image) -> Self {
        unsafe {
            let handle = device
                .handle()
                .create_image_view(
                    &vk::ImageViewCreateInfo::builder()
                        .components(
                            vk::ComponentMapping::builder()
                                .r(vk::ComponentSwizzle::IDENTITY)
                                .g(vk::ComponentSwizzle::IDENTITY)
                                .b(vk::ComponentSwizzle::IDENTITY)
                                .a(vk::ComponentSwizzle::IDENTITY)
                                .build(),
                        )
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(image.format())
                        .subresource_range(
                            vk::ImageSubresourceRange::builder()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .base_mip_level(0)
                                .level_count(1)
                                .base_array_layer(0)
                                .layer_count(1)
                                .build(),
                        )
                        .image(image.handle())
                        .build(),
                    None,
                )
                .unwrap();
            Self {
                inner: Arc::new(ImageViewRef {
                    image: image.clone(),
                    handle,
                    device: device.clone(),
                }),
            }
        }
    }

    pub fn image(&self) -> Image {
        self.inner.image.clone()
    }
}

impl Drop for ImageView {
    fn drop(&mut self) {
        unsafe {
            self.inner
                .device
                .handle()
                .destroy_image_view(self.inner.handle, None);
        }
    }
}

impl Image {
    pub fn create_view(&self) -> ImageView {
        ImageView::new(&self.inner.device, self)
    }
}
