use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::sync::Arc;

use ash::vk;

use crate::physical_device::PhysicalDevice;

use crate::entry::Entry;
use crate::name;
use crate::queue_family::QueueFamilyProperties;

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number: i32 = callback_data.message_id_number as i32;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };
    if message_id_number == 1151829889 {
        log::debug!("suppress device address validation");
        return vk::FALSE;
    }

    use vk::DebugUtilsMessageSeverityFlagsEXT;
    match message_severity {
        DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
            log::debug!("{:?} : {}\n", message_type, message,);
        }
        DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            log::warn!("{:?} : {}\n", message_type, message,);
        }
        DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            log::error!("{:?} : {}\n", message_type, message,);
        }
        DebugUtilsMessageSeverityFlagsEXT::INFO => {
            log::info!("{:?} : {}\n", message_type, message,);
        }
        _ => {}
    }

    vk::FALSE
}

pub(crate) struct InstanceRef {
    pub(crate) handle: ash::Instance,
    pub(crate) entry: Entry,
    enabled_layers: Vec<name::instance::Layer>,
    enabled_extensions: Vec<name::instance::Extension>,
    pub(crate) surface_loader: Option<ash::extensions::khr::Surface>,
    pub debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_call_back: vk::DebugUtilsMessengerEXT,
    display_loader: ash::extensions::khr::Display,
}

#[derive(Clone)]
pub struct Instance {
    pub(crate) inner: Arc<InstanceRef>,
}

impl Instance {
    pub fn new(
        entry: Entry,
        layers: &[name::instance::Layer],
        extensions: &[name::instance::Extension],
    ) -> Self {
        let app_name = CString::new(env!("CARGO_PKG_NAME")).unwrap();
        let engine_name = CString::new("maligog").unwrap();

        let mut extensions = extensions.to_owned();
        extensions.push(crate::name::instance::Extension::ExtDebugUtils);

        let appinfo = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(0)
            .engine_name(&engine_name)
            .engine_version(0)
            .api_version(vk::make_version(1, 2, 0));

        let layer_names = layers
            .iter()
            .map(|layer| CString::new(layer.as_ref()).unwrap())
            .collect::<Vec<_>>();
        let layers_names_raw: Vec<*const i8> = layer_names
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        let supported_layers = entry.supported_instance_layers();
        for layer in layers {
            if !supported_layers.contains(layer) {
                panic!("not support layer {}", layer.as_ref());
            }
        }

        let extension_names = extensions
            .iter()
            .map(|extension| CString::new(extension.as_ref()).unwrap())
            .collect::<Vec<_>>();
        let extension_names_raw = extension_names
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();

        let supported_extensions = entry.supported_instance_extensions();
        for extension in &extensions {
            if !supported_extensions.contains(&extension.to_owned()) {
                panic!("not support extension {}", &extension.as_ref());
            }
        }

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&appinfo)
            .enabled_layer_names(&layers_names_raw)
            .enabled_extension_names(&extension_names_raw);
        let handle = unsafe { entry.handle.create_instance(&create_info, None).unwrap() };

        let surface_loader = match extensions.contains(&name::instance::Extension::KhrSurface) {
            true => Some(ash::extensions::khr::Surface::new(&entry.handle, &handle)),
            false => None,
        };

        let debug_utils_loader = ash::extensions::ext::DebugUtils::new(&entry.handle, &handle);
        let debug_call_back = unsafe {
            debug_utils_loader.create_debug_utils_messenger(
                &vk::DebugUtilsMessengerCreateInfoEXT::builder()
                    .message_severity(
                        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                            | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                            | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
                    )
                    .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
                    .pfn_user_callback(Some(vulkan_debug_callback)),
                None,
            )
        }
        .unwrap();

        let display_loader = ash::extensions::khr::Display::new(&entry.handle, &handle);

        Self {
            inner: Arc::new(InstanceRef {
                handle,
                entry,
                surface_loader,
                debug_utils_loader,
                display_loader,
                enabled_layers: layers.to_vec(),
                enabled_extensions: extensions.to_vec(),
                debug_call_back,
            }),
        }
    }

    pub fn enumerate_physical_device(&self) -> Vec<PhysicalDevice> {
        unsafe {
            let pdevices = self.inner.handle.enumerate_physical_devices().unwrap();

            pdevices
                .iter()
                .map(|pdevice| {
                    let props = self.inner.handle.get_physical_device_properties(*pdevice);
                    let mut props2 = vk::PhysicalDeviceRayTracingPipelinePropertiesKHR::default();
                    self.inner.handle.get_physical_device_properties2(
                        *pdevice,
                        &mut vk::PhysicalDeviceProperties2::builder()
                            .push_next(&mut props2)
                            .build(),
                    );
                    let ray_tracing_pipeline_properties =
                        crate::physical_device::PhysicalDeviceRayTracingPipelineProperties {
                            shader_group_handle_size: props2.shader_group_handle_size,
                            max_ray_recursion_depth: props2.max_ray_recursion_depth,
                            max_shader_group_stride: props2.max_shader_group_stride,
                            shader_group_base_alignment: props2.shader_group_base_alignment,
                            max_ray_dispatch_invocation_count: props2
                                .max_ray_dispatch_invocation_count,
                            shader_group_handle_alignment: props2.shader_group_handle_alignment,
                            max_ray_hit_attribute_size: props2.max_ray_hit_attribute_size,
                        };

                    let queue_families = self
                        .inner
                        .handle
                        .get_physical_device_queue_family_properties(*pdevice)
                        .into_iter()
                        .enumerate()
                        .map(|(index, properties)| {
                            QueueFamilyProperties {
                                index: index as u32,
                                support_graphics: properties
                                    .queue_flags
                                    .contains(vk::QueueFlags::GRAPHICS),
                                support_compute: properties
                                    .queue_flags
                                    .contains(vk::QueueFlags::COMPUTE),
                                support_transfer: properties
                                    .queue_flags
                                    .contains(vk::QueueFlags::TRANSFER),
                                count: properties.queue_count,
                            }
                        })
                        .collect();

                    PhysicalDevice {
                        name: CStr::from_ptr(props.device_name.as_ptr())
                            .to_str()
                            .unwrap()
                            .to_owned(),
                        device_type: props.device_type,
                        handle: *pdevice,
                        instance: self.clone(),
                        ray_tracing_pipeline_properties,
                        queue_families,
                    }
                })
                .collect()
        }
    }
}

impl Drop for InstanceRef {
    fn drop(&mut self) {
        unsafe {
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_call_back, None);
            self.handle.destroy_instance(None);
        }
    }
}

#[test]
fn test_enumerate() {
    let entry = Entry::new().unwrap();
    let instance = entry.create_instance(&[], &[]);
    let pdevices = instance.enumerate_physical_device();
    dbg!(&pdevices);
    let _pdevice = instance
        .enumerate_physical_device()
        .iter()
        .find(|p| {
            p.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
                && p.queue_families
                    .iter()
                    .any(|f| f.support_compute() && f.support_graphics())
        })
        .unwrap();
}

#[test]
fn test_create_device() {
    let entry = Entry::new().unwrap();
    let instance = entry.create_instance(&[], &[]);
    let pdevices = instance.enumerate_physical_device();
    dbg!(&pdevices);
    let _pdevice = instance
        .enumerate_physical_device()
        .iter()
        .find(|p| {
            p.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
                && p.queue_families
                    .iter()
                    .any(|f| f.support_compute() && f.support_graphics())
        })
        .unwrap();
}
