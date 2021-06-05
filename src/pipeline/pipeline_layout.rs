use std::ffi::CString;
use std::sync::Arc;

use crate::DescriptorSetLayout;
use crate::Device;
use ash::vk;
use ash::vk::Handle;

pub(crate) struct PipelineLayoutRef {
    pub(crate) handle: vk::PipelineLayout,
    device: Device,
}

#[derive(Clone)]
pub struct PipelineLayout {
    pub(crate) inner: Arc<PipelineLayoutRef>,
}

impl PipelineLayout {
    pub fn new(
        device: &Device,
        name: Option<&str>,
        set_layouts: &[&DescriptorSetLayout],
        push_constant_ranges: &[vk::PushConstantRange],
    ) -> Self {
        let set_layouts = set_layouts
            .iter()
            .map(|layout| layout.inner.handle)
            .collect::<Vec<_>>();
        let info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(set_layouts.as_slice())
            .push_constant_ranges(push_constant_ranges)
            .build();
        unsafe {
            let handle = device
                .inner
                .handle
                .create_pipeline_layout(&info, None)
                .unwrap();
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
                            .object_type(vk::ObjectType::PIPELINE_LAYOUT)
                            .object_name(CString::new(name).unwrap().as_ref())
                            .build(),
                    )
                    .unwrap();
            }
            Self {
                inner: Arc::new(PipelineLayoutRef {
                    handle,
                    device: device.clone(),
                }),
            }
        }
    }
}

impl Drop for PipelineLayoutRef {
    fn drop(&mut self) {
        unsafe {
            self.device
                .inner
                .handle
                .destroy_pipeline_layout(self.handle, None);
        }
    }
}

impl Device {
    pub fn create_pipeline_layout(
        &self,
        name: Option<&str>,
        set_layouts: &[&DescriptorSetLayout],
        push_constant_ranges: &[vk::PushConstantRange],
    ) -> PipelineLayout {
        PipelineLayout::new(&self, name, set_layouts, push_constant_ranges)
    }
}
