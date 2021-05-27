use crate::device::Device;
use crate::sampler::Sampler;
use ash::vk::{self, Handle};
use std::ffi::CString;

#[derive(Clone)]
pub enum DescriptorType {
    Sampler(Option<Sampler>),
    SampledImage,
    UniformBuffer,
    StorageBuffer,
    AccelerationStructure,
    StorageImage,
}

#[derive(Clone)]
pub struct DescriptorSetLayoutBinding {
    pub binding: u32,
    pub descriptor_type: DescriptorType,
    pub stage_flags: vk::ShaderStageFlags,
}

pub struct DescriptorSetLayout {
    handle: vk::DescriptorSetLayout,
    device: Device,
    bindings: Vec<DescriptorSetLayoutBinding>,
    vk_bindings: Vec<vk::DescriptorSetLayoutBinding>,
}

impl DescriptorSetLayout {
    pub fn new(
        device: Device,
        name: Option<&str>,
        bindings: &[DescriptorSetLayoutBinding],
    ) -> Self {
        let vk_bindings = bindings
            .iter()
            .map(|binding| match &binding.descriptor_type {
                DescriptorType::Sampler(immutable_sampler) => {
                    if let Some(sampler) = immutable_sampler {
                        vk::DescriptorSetLayoutBinding::builder()
                            .binding(binding.binding)
                            .descriptor_type(vk::DescriptorType::SAMPLER)
                            .descriptor_count(1)
                            .immutable_samplers(&[sampler.inner.handle])
                            .stage_flags(binding.stage_flags)
                            .build()
                    } else {
                        vk::DescriptorSetLayoutBinding::builder()
                            .binding(binding.binding)
                            .descriptor_type(vk::DescriptorType::SAMPLER)
                            .descriptor_count(1)
                            .stage_flags(binding.stage_flags)
                            .build()
                    }
                }
                DescriptorType::SampledImage => vk::DescriptorSetLayoutBinding::builder()
                    .binding(binding.binding)
                    .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                    .descriptor_count(1)
                    .stage_flags(binding.stage_flags)
                    .build(),
                DescriptorType::UniformBuffer => vk::DescriptorSetLayoutBinding::builder()
                    .binding(binding.binding)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count(1)
                    .stage_flags(binding.stage_flags)
                    .build(),
                DescriptorType::StorageBuffer => vk::DescriptorSetLayoutBinding::builder()
                    .binding(binding.binding)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .descriptor_count(1)
                    .stage_flags(binding.stage_flags)
                    .build(),
                DescriptorType::AccelerationStructure => vk::DescriptorSetLayoutBinding::builder()
                    .binding(binding.binding)
                    .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
                    .descriptor_count(1)
                    .stage_flags(binding.stage_flags)
                    .build(),
                DescriptorType::StorageImage => vk::DescriptorSetLayoutBinding::builder()
                    .binding(binding.binding)
                    .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                    .descriptor_count(1)
                    .stage_flags(binding.stage_flags)
                    .build(),
            })
            .collect::<Vec<_>>();
        let info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(vk_bindings.as_slice())
            .build();
        unsafe {
            let handle = device
                .inner
                .handle
                .create_descriptor_set_layout(&info, None)
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
                            .object_type(vk::ObjectType::DESCRIPTOR_SET_LAYOUT)
                            .object_name(CString::new(name).unwrap().as_ref())
                            .build(),
                    )
                    .unwrap();
            }

            Self {
                handle,
                device,
                bindings: bindings.to_owned(),
                vk_bindings,
            }
        }
    }
}

impl Drop for DescriptorSetLayout {
    fn drop(&mut self) {
        unsafe {
            self.device
                .inner
                .handle
                .destroy_descriptor_set_layout(self.handle, None);
        }
    }
}

impl Device {
    pub fn create_descriptor_set_layout(
        &self,
        name: Option<&str>,
        bindings: &[DescriptorSetLayoutBinding],
    ) -> DescriptorSetLayout {
        DescriptorSetLayout::new(self.clone(), name, bindings)
    }
}
