use std::sync::Arc;

use crate::{BinarySemaphore, Device, Image, Surface};
use ash::vk::{self, Handle};

pub struct SwapchainRef {
    pub(crate) handle: vk::SwapchainKHR,
    pub(crate) device: Device,
    surface: Surface,
    width: u32,
    height: u32,
    format: vk::Format,
    image_available_semaphore: BinarySemaphore,
    present_mode: vk::PresentModeKHR,
    images: Vec<Image>,
}

#[derive(Clone)]
pub struct Swapchain {
    pub(crate) inner: Arc<SwapchainRef>,
}

impl Swapchain {
    pub fn new(device: &Device, surface: Surface, present_mode: vk::PresentModeKHR) -> Self {
        unsafe {
            let surface_loader = &device
                .inner
                .pdevice
                .instance
                .inner
                .surface_loader
                .as_ref()
                .unwrap();
            let surface_capabilities = surface_loader
                .get_physical_device_surface_capabilities(
                    device.inner.pdevice.handle,
                    surface.inner.handle,
                )
                .unwrap();

            let surface_format = surface_loader
                .get_physical_device_surface_formats(
                    device.inner.pdevice.handle,
                    surface.inner.handle,
                )
                .unwrap()[0];

            let format = surface_format.format;

            let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
                .surface(surface.inner.handle)
                .min_image_count(2)
                .image_color_space(surface_format.color_space)
                .image_format(format)
                .image_extent(surface_capabilities.current_extent)
                .image_usage(
                    vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST,
                )
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(device.all_queue_family_indices())
                .pre_transform(vk::SurfaceTransformFlagsKHR::IDENTITY)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .clipped(true)
                .image_array_layers(1);
            let handle = device
                .inner
                .swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .unwrap();
            let image_available_semaphore = BinarySemaphore::new(&device);
            let image_handles = device
                .inner
                .swapchain_loader
                .get_swapchain_images(handle)
                .unwrap();
            let images = image_handles
                .into_iter()
                .map(|i| {
                    Image::from_handle(
                        device,
                        i,
                        surface_capabilities.current_extent.width,
                        surface_capabilities.current_extent.height,
                        format,
                    )
                })
                .collect();

            Self {
                inner: Arc::new(SwapchainRef {
                    handle,
                    device: device.clone(),
                    surface,
                    width: surface_capabilities.current_extent.width,
                    height: surface_capabilities.current_extent.height,
                    format,
                    image_available_semaphore,
                    present_mode,
                    images,
                }),
            }
        }
    }

    pub fn acquire_next_image(&self) -> Result<u32, u32> {
        unsafe {
            let (index, sub) = self
                .inner
                .device
                .inner
                .swapchain_loader
                .acquire_next_image(
                    self.inner.handle,
                    0,
                    self.inner.image_available_semaphore.inner.handle,
                    vk::Fence::null(),
                )
                .unwrap();
            match sub {
                true => Err(index),
                false => Ok(index),
            }
        }
    }

    pub fn present(&self, index: u32, wait_semaphore: &[&BinarySemaphore]) {
        let wait_handles = wait_semaphore
            .iter()
            .map(|s| s.inner.handle)
            .collect::<Vec<_>>();

        let info = vk::PresentInfoKHR::builder()
            .swapchains(&[self.handle()])
            .wait_semaphores(wait_handles.as_slice())
            .image_indices(&[index])
            .build();
        unsafe {
            if let Err(e) = self
                .inner
                .device
                .inner
                .swapchain_loader
                .queue_present(self.inner.device.graphics_queue().inner.handle, &info)
            {
                log::warn!("{:?}", e);
            }
        }
    }

    // pub fn renew(&self) {
    //     let swapchain_loader = &self.inner.device.inner.swapchain_loader;
    //     let surface_loader = &self
    //         .inner
    //         .device
    //         .inner
    //         .pdevice
    //         .instance
    //         .inner
    //         .surface_loader
    //         .as_ref()
    //         .unwrap();
    //     let pdevice = &self.inner.device.inner.pdevice;
    //     unsafe {
    //         let surface_capabilities = surface_loader
    //             .get_physical_device_surface_capabilities(
    //                 pdevice.handle,
    //                 self.inner.surface.inner.handle,
    //             )
    //             .unwrap();

    //         let surface_format = surface_loader
    //             .get_physical_device_surface_formats(
    //                 pdevice.handle,
    //                 self.inner.surface.inner.handle,
    //             )
    //             .unwrap()[0];

    //         let old_swapchain = self.handle();
    //         let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
    //             .surface(self.inner.surface.inner.handle)
    //             .min_image_count(2)
    //             .image_color_space(surface_format.color_space)
    //             .image_format(surface_format.format)
    //             .image_extent(surface_capabilities.current_extent)
    //             .image_usage(
    //                 vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST,
    //             )
    //             .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
    //             .pre_transform(vk::SurfaceTransformFlagsKHR::IDENTITY)
    //             .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
    //             .present_mode(self.inner.present_mode)
    //             .clipped(true)
    //             .image_array_layers(1)
    //             .old_swapchain(old_swapchain);

    //         self.inner.handle = swapchain_loader
    //             .create_swapchain(&swapchain_create_info, None)
    //             .unwrap();
    //         self.inner
    //             .device
    //             .inner
    //             .swapchain_loader
    //             .destroy_swapchain(old_swapchain, None);
    //         self.inner.width.store(
    //             surface_capabilities.current_extent.width,
    //             std::sync::atomic::Ordering::SeqCst,
    //         );
    //         self.inner.height.store(
    //             surface_capabilities.current_extent.height,
    //             std::sync::atomic::Ordering::SeqCst,
    //         );
    //     }
    // }

    pub fn image_available_semaphore(&self) -> BinarySemaphore {
        self.inner.image_available_semaphore.clone()
    }

    pub fn handle(&self) -> vk::SwapchainKHR {
        self.inner.handle
    }

    pub fn width(&self) -> u32 {
        self.inner.width
    }

    pub fn height(&self) -> u32 {
        self.inner.height
    }

    pub fn format(&self) -> vk::Format {
        self.inner.format
    }
}

impl Drop for SwapchainRef {
    fn drop(&mut self) {
        unsafe {
            self.device
                .inner
                .swapchain_loader
                .destroy_swapchain(self.handle, None)
        }
    }
}

impl Device {
    pub fn create_swapchain(
        &self,
        surface: Surface,
        present_mode: vk::PresentModeKHR,
    ) -> Swapchain {
        Swapchain::new(self, surface, present_mode)
    }
}
