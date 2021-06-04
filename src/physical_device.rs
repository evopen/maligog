use ash::vk;

use crate::device::{Device, DeviceFeatures};
use crate::instance::Instance;
use crate::name;
use crate::queue::Queue;
use crate::queue_family::{QueueFamily, QueueFamilyProperties};
use std::ffi::CStr;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct PhysicalDeviceRayTracingPipelineProperties {
    pub shader_group_handle_size: u32,
    pub max_ray_recursion_depth: u32,
    pub max_shader_group_stride: u32,
    pub shader_group_base_alignment: u32,
    pub max_ray_dispatch_invocation_count: u32,
    pub shader_group_handle_alignment: u32,
    pub max_ray_hit_attribute_size: u32,
}

#[derive(Clone)]
pub struct PhysicalDevice {
    pub(crate) name: String,
    pub(crate) device_type: vk::PhysicalDeviceType,
    pub(crate) handle: vk::PhysicalDevice,
    pub(crate) instance: Instance,
    pub(crate) ray_tracing_pipeline_properties: PhysicalDeviceRayTracingPipelineProperties,
    pub queue_families: Vec<QueueFamilyProperties>,
}

impl PhysicalDevice {
    pub fn device_type(&self) -> vk::PhysicalDeviceType {
        self.device_type
    }

    pub fn supported_device_extensions_raw(&self) -> Vec<String> {
        unsafe {
            self.instance
                .inner
                .handle
                .enumerate_device_extension_properties(self.handle)
                .unwrap()
                .iter()
                .map(|ext| {
                    CStr::from_ptr(ext.extension_name.as_ptr() as *const std::os::raw::c_char)
                        .to_str()
                        .unwrap()
                        .to_owned()
                })
                .collect::<Vec<_>>()
        }
    }

    pub fn supported_device_extensions(&self) -> Vec<name::device::Extension> {
        self.supported_device_extensions_raw()
            .iter()
            .filter_map(|ext| {
                match name::device::Extension::from_str(ext) {
                    Ok(ext) => Some(ext),
                    Err(_) => None,
                }
            })
            .collect::<Vec<_>>()
    }

    pub fn create_device(&self) -> Device {
        let device = Device::new(
            self.instance.clone(),
            self.clone(),
            &DeviceFeatures {},
            &self.supported_device_extensions(),
        );

        device
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn queue_families(&self) -> &[QueueFamilyProperties] {
        self.queue_families.as_slice()
    }

    pub(crate) fn graphics_queue_family(&self) -> QueueFamilyProperties {
        self.queue_families
            .iter()
            .find(|qf| qf.support_graphics)
            .unwrap()
            .to_owned()
    }

    pub(crate) fn transfer_queue_family(&self) -> QueueFamilyProperties {
        self.queue_families
            .iter()
            .find(|qf| qf.support_transfer && !qf.support_graphics && !qf.support_compute)
            .unwrap()
            .to_owned()
    }

    pub(crate) fn compute_queue_family(&self) -> QueueFamilyProperties {
        self.queue_families
            .iter()
            .find(|qf| qf.support_compute && !qf.support_graphics)
            .unwrap()
            .to_owned()
    }
}

impl std::fmt::Debug for PhysicalDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhysicalDevice")
            .field("name", &self.name)
            .field("device_type", &self.device_type)
            .field(
                "ray_tracing_pipeline_properties",
                &self.ray_tracing_pipeline_properties,
            )
            .field("queue_families", &self.queue_families)
            .finish()
    }
}

#[test]
fn test_create_device() {
    use crate::entry::Entry;

    let entry = Entry::new().unwrap();
    let instance = entry.create_instance(&[], &[]);
    let pdevices = instance.enumerate_physical_device();
    dbg!(&pdevices);

    let pdevice = instance
        .enumerate_physical_device()
        .into_iter()
        .find(|p| {
            p.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
                && p.queue_families
                    .iter()
                    .any(|f| f.support_compute() && f.support_graphics())
        })
        .unwrap();
    dbg!(pdevice.supported_device_extensions());
    dbg!(pdevice.supported_device_extensions_raw());
    let pdevice = Arc::new(pdevice);
    let queue_family = pdevice
        .queue_families()
        .iter()
        .find(|f| f.support_graphics() && f.support_compute())
        .unwrap();
    let _device = pdevice.create_device();
}
