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
    pub fn new(device: Device, name: Option<&str>) -> Self {
        let info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .build();
        unsafe {
            let handle = device.inner.handle.create_sampler(&info, None).unwrap();
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
                            .object_type(vk::ObjectType::SAMPLER)
                            .object_name(CString::new(name).unwrap().as_ref())
                            .build(),
                    )
                    .unwrap();
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
    pub fn create_sampler(&self, name: Option<&str>) -> Sampler {
        Sampler::new(self.clone(), name)
    }
}

impl Drop for SamplerRef {
    fn drop(&mut self) {
        unsafe {
            self.device.inner.handle.destroy_sampler(self.handle, None);
        }
    }
}
