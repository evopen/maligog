use std::ffi::CString;
use std::sync::Arc;

use ash::version::{EntryV1_0, InstanceV1_0};
use ash::vk;

use crate::entry::Entry;

use crate::name;

pub struct Instance {
    handle: ash::Instance,
    entry: Arc<Entry>,
    enabled_layers: Vec<name::instance::Layer>,
    enabled_extensions: Vec<name::instance::Extension>,
    surface_loader: ash::extensions::khr::Surface,
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    display_loader: ash::extensions::khr::Display,
}

impl Instance {
    pub fn new(
        entry: Arc<Entry>,
        layers: &[name::instance::Layer],
        extensions: &[name::instance::Extension],
    ) -> Self {
        let app_name = CString::new(env!("CARGO_PKG_NAME")).unwrap();
        let engine_name = CString::new("Silly Cat Engine").unwrap();

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
        for extension in extensions {
            if !supported_extensions.contains(&extension.to_owned()) {
                panic!("not support extension {}", &extension.as_ref());
            }
        }

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&appinfo)
            .enabled_layer_names(&layers_names_raw)
            .enabled_extension_names(&extension_names_raw);
        let handle = unsafe { entry.handle.create_instance(&create_info, None).unwrap() };

        let surface_loader = ash::extensions::khr::Surface::new(&entry.handle, &handle);

        let debug_utils_loader = ash::extensions::ext::DebugUtils::new(&entry.handle, &handle);

        let display_loader = ash::extensions::khr::Display::new(&entry.handle, &handle);

        let result = Self {
            handle,
            entry,
            surface_loader,
            debug_utils_loader,
            display_loader,
            enabled_layers: layers.to_vec(),
            enabled_extensions: extensions.to_vec(),
        };

        result
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            self.handle.destroy_instance(None);
        }
    }
}
