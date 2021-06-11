use std::ffi::CString;
use std::sync::Arc;

use crate::device::Device;
use ash::vk;
use ash::vk::Handle;

pub(crate) struct SamplerRef {
    pub(crate) handle: vk::Sampler,
    device: Device,
    name: Option<String>,
}

#[derive(Clone)]
pub struct Sampler {
    pub(crate) inner: Arc<SamplerRef>,
}

impl Sampler {
    pub fn new(
        device: Device,
        name: Option<&str>,
        mag_filter: vk::Filter,
        min_filter: vk::Filter,
        address_mode_u: vk::SamplerAddressMode,
        address_mode_v: vk::SamplerAddressMode,
    ) -> Self {
        let info = vk::SamplerCreateInfo::builder()
            .mag_filter(mag_filter)
            .min_filter(min_filter)
            .address_mode_u(address_mode_u)
            .address_mode_v(address_mode_v)
            .build();
        unsafe {
            let handle = device.inner.handle.create_sampler(&info, None).unwrap();
            if let Some(name) = name {
                device.debug_set_object_name(name, handle.as_raw(), vk::ObjectType::SAMPLER);
            }
            Self {
                inner: Arc::new(SamplerRef {
                    handle,
                    device,
                    name: name.map(|s| s.to_owned()),
                }),
            }
        }
    }
}

impl Device {
    pub fn create_sampler(
        &self,
        name: Option<&str>,
        mag_filter: vk::Filter,
        min_filter: vk::Filter,
        address_mode_u: vk::SamplerAddressMode,
        address_mode_v: vk::SamplerAddressMode,
    ) -> Sampler {
        Sampler::new(
            self.clone(),
            name,
            mag_filter,
            min_filter,
            address_mode_u,
            address_mode_v,
        )
    }
}

impl Drop for SamplerRef {
    fn drop(&mut self) {
        unsafe {
            self.device.inner.handle.destroy_sampler(self.handle, None);
        }
    }
}
