use std::collections::BTreeMap;
use std::ffi::CString;
use std::sync::Arc;

use crate::{DescriptorType, Device};
use ash::vk::{self, Handle};

#[derive(Clone)]
pub struct DescriptorSetLayoutBinding {
    pub binding: u32,
    pub descriptor_type: DescriptorType,
    pub stage_flags: vk::ShaderStageFlags,
    pub descriptor_count: u32,
}

pub(crate) struct DescriptorSetLayoutRef {
    pub(crate) handle: vk::DescriptorSetLayout,
    device: Device,
    bindings: Vec<DescriptorSetLayoutBinding>,
    pub(crate) vk_bindings: BTreeMap<u32, vk::DescriptorSetLayoutBinding>,
}

#[derive(Clone)]
pub struct DescriptorSetLayout {
    pub(crate) inner: Arc<DescriptorSetLayoutRef>,
}

impl DescriptorSetLayout {
    pub fn new(
        device: Device,
        name: Option<&str>,
        bindings: &[DescriptorSetLayoutBinding],
    ) -> Self {
        let vk_bindings = bindings
            .iter()
            .map(|binding| {
                match &binding.descriptor_type {
                    DescriptorType::Sampler(immutable_sampler) => {
                        if let Some(sampler) = immutable_sampler {
                            vk::DescriptorSetLayoutBinding::builder()
                                .binding(binding.binding)
                                .descriptor_type(vk::DescriptorType::SAMPLER)
                                .descriptor_count(binding.descriptor_count)
                                .immutable_samplers(&[sampler.inner.handle])
                                .stage_flags(binding.stage_flags)
                                .build()
                        } else {
                            vk::DescriptorSetLayoutBinding::builder()
                                .binding(binding.binding)
                                .descriptor_type(vk::DescriptorType::SAMPLER)
                                .descriptor_count(binding.descriptor_count)
                                .stage_flags(binding.stage_flags)
                                .build()
                        }
                    }
                    DescriptorType::SampledImage => {
                        vk::DescriptorSetLayoutBinding::builder()
                            .binding(binding.binding)
                            .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                            .descriptor_count(binding.descriptor_count)
                            .stage_flags(binding.stage_flags)
                            .build()
                    }
                    DescriptorType::UniformBuffer => {
                        vk::DescriptorSetLayoutBinding::builder()
                            .binding(binding.binding)
                            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                            .descriptor_count(binding.descriptor_count)
                            .stage_flags(binding.stage_flags)
                            .build()
                    }
                    DescriptorType::StorageBuffer => {
                        vk::DescriptorSetLayoutBinding::builder()
                            .binding(binding.binding)
                            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                            .descriptor_count(binding.descriptor_count)
                            .stage_flags(binding.stage_flags)
                            .build()
                    }
                    DescriptorType::AccelerationStructure => {
                        vk::DescriptorSetLayoutBinding::builder()
                            .binding(binding.binding)
                            .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
                            .descriptor_count(binding.descriptor_count)
                            .stage_flags(binding.stage_flags)
                            .build()
                    }
                    DescriptorType::StorageImage => {
                        vk::DescriptorSetLayoutBinding::builder()
                            .binding(binding.binding)
                            .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                            .descriptor_count(binding.descriptor_count)
                            .stage_flags(binding.stage_flags)
                            .build()
                    }
                }
            })
            .collect::<Vec<_>>();
        let info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(vk_bindings.as_slice())
            .build();
        let vk_bindings = vk_bindings
            .iter()
            .map(|b| (b.binding, b.to_owned()))
            .collect::<BTreeMap<u32, vk::DescriptorSetLayoutBinding>>();
        unsafe {
            let handle = device
                .inner
                .handle
                .create_descriptor_set_layout(&info, None)
                .unwrap();
            if let Some(name) = name {
                device.debug_set_object_name(
                    name,
                    handle.as_raw(),
                    vk::ObjectType::DESCRIPTOR_SET_LAYOUT,
                );
            }

            Self {
                inner: Arc::new(DescriptorSetLayoutRef {
                    handle,
                    device,
                    bindings: bindings.to_owned(),
                    vk_bindings,
                }),
            }
        }
    }
}

impl Drop for DescriptorSetLayoutRef {
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
