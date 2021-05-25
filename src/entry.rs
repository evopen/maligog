use std::ffi::CStr;
use std::str::FromStr;
use std::sync::Arc;

use ash::vk;

// use crate::instance::Instance;

use crate::instance::Instance;
use crate::name;

pub struct Entry {
    pub(crate) handle: ash::Entry,
}

impl Entry {
    pub fn new() -> Result<Arc<Self>, ash::LoadingError> {
        let handle = unsafe { ash::Entry::new()? };

        let result = Self { handle };

        Ok(Arc::new(result))
    }

    pub fn vulkan_loader_version(&self) -> String {
        let version_str = match self.handle.try_enumerate_instance_version().unwrap() {
            // Vulkan 1.1+
            Some(version) => {
                let major = vk::version_major(version);
                let minor = vk::version_minor(version);
                let patch = vk::version_patch(version);
                format!("{}.{}.{}", major, minor, patch)
            }
            // Vulkan 1.0
            None => String::from("1.0"),
        };
        version_str
    }

    pub fn supported_instance_layers_raw(&self) -> Vec<String> {
        self.handle
            .enumerate_instance_layer_properties()
            .unwrap()
            .iter()
            .map(|layer| {
                unsafe { CStr::from_ptr(layer.layer_name.as_ptr() as *const std::os::raw::c_char) }
                    .to_str()
                    .unwrap()
                    .to_owned()
            })
            .collect::<Vec<_>>()
    }

    pub fn supported_instance_layers(&self) -> Vec<name::instance::Layer> {
        self.handle
            .enumerate_instance_layer_properties()
            .unwrap()
            .iter()
            .filter_map(|layer| {
                match name::instance::Layer::from_str(
                    unsafe {
                        CStr::from_ptr(layer.layer_name.as_ptr() as *const std::os::raw::c_char)
                    }
                    .to_str()
                    .unwrap(),
                ) {
                    Ok(l) => Some(l),
                    Err(_) => None,
                }
            })
            .collect::<Vec<_>>()
    }

    pub fn supported_instance_extensions(&self) -> Vec<name::instance::Extension> {
        self.handle
            .enumerate_instance_extension_properties()
            .unwrap()
            .iter()
            .filter_map(|ext| {
                match name::instance::Extension::from_str(
                    unsafe {
                        CStr::from_ptr(ext.extension_name.as_ptr() as *const std::os::raw::c_char)
                    }
                    .to_str()
                    .unwrap(),
                ) {
                    Ok(ext) => Some(ext),
                    Err(_) => None,
                }
            })
            .collect::<Vec<_>>()
    }

    pub fn supported_instance_extensions_raw(&self) -> Vec<String> {
        self.handle
            .enumerate_instance_extension_properties()
            .unwrap()
            .iter()
            .map(|ext| {
                unsafe {
                    CStr::from_ptr(ext.extension_name.as_ptr() as *const std::os::raw::c_char)
                }
                .to_str()
                .unwrap()
                .to_owned()
            })
            .collect::<Vec<_>>()
    }

    pub fn create_instance(
        self: &Arc<Self>,
        layers: &[name::instance::Layer],
        extensions: &[name::instance::Extension],
    ) -> Arc<Instance> {
        let instance = Instance::new(self.clone(), layers, extensions);
        Arc::new(instance)
    }
}

#[test]
fn entry_test() {
    let entry = Entry::new().unwrap();
    dbg!(entry.vulkan_loader_version());
    dbg!(entry.supported_instance_extensions());
    dbg!(entry.supported_instance_layers());
    let _instance = entry.create_instance(&[], &[]);
}
