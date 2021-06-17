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
    pub variable_count: bool,
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
        let mut immutable_samplers = Vec::new();
        let mut binding_flags = Vec::new();
        let vk_bindings = bindings
            .iter()
            .map(|binding| {
                match &binding.descriptor_type {
                    DescriptorType::Sampler(immutable_sampler) => {
                        if binding.variable_count {
                            binding_flags.push(
                                vk::DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT
                                    | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                                    | vk::DescriptorBindingFlags::PARTIALLY_BOUND
                                    | vk::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING,
                            );
                        } else {
                            binding_flags.push(
                                vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                                    | vk::DescriptorBindingFlags::PARTIALLY_BOUND
                                    | vk::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING,
                            );
                        }

                        if let Some(sampler) = immutable_sampler {
                            immutable_samplers.push(sampler.inner.handle);
                            vk::DescriptorSetLayoutBinding::builder()
                                .binding(binding.binding)
                                .descriptor_type(vk::DescriptorType::SAMPLER)
                                .immutable_samplers(&immutable_samplers)
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
                        if binding.variable_count {
                            binding_flags.push(
                                vk::DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT
                                    | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                                    | vk::DescriptorBindingFlags::PARTIALLY_BOUND
                                    | vk::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING,
                            );
                        } else {
                            binding_flags.push(
                                vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                                    | vk::DescriptorBindingFlags::PARTIALLY_BOUND
                                    | vk::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING,
                            );
                        }
                        vk::DescriptorSetLayoutBinding::builder()
                            .binding(binding.binding)
                            .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                            .descriptor_count(binding.descriptor_count)
                            .stage_flags(binding.stage_flags)
                            .build()
                    }
                    DescriptorType::UniformBuffer => {
                        if binding.variable_count {
                            binding_flags.push(
                                vk::DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT
                                    | vk::DescriptorBindingFlags::PARTIALLY_BOUND
                                    | vk::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING,
                            );
                        } else {
                            binding_flags.push(
                                vk::DescriptorBindingFlags::PARTIALLY_BOUND
                                    | vk::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING,
                            );
                        }
                        vk::DescriptorSetLayoutBinding::builder()
                            .binding(binding.binding)
                            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                            .descriptor_count(binding.descriptor_count)
                            .stage_flags(binding.stage_flags)
                            .build()
                    }
                    DescriptorType::StorageBuffer => {
                        if binding.variable_count {
                            binding_flags.push(
                                vk::DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT
                                    | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                                    | vk::DescriptorBindingFlags::PARTIALLY_BOUND
                                    | vk::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING,
                            );
                        } else {
                            binding_flags.push(
                                vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                                    | vk::DescriptorBindingFlags::PARTIALLY_BOUND
                                    | vk::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING,
                            );
                        }
                        vk::DescriptorSetLayoutBinding::builder()
                            .binding(binding.binding)
                            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                            .descriptor_count(binding.descriptor_count)
                            .stage_flags(binding.stage_flags)
                            .build()
                    }
                    DescriptorType::AccelerationStructure => {
                        if binding.variable_count {
                            binding_flags.push(
                                vk::DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT
                                    | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                                    | vk::DescriptorBindingFlags::PARTIALLY_BOUND
                                    | vk::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING,
                            );
                        } else {
                            binding_flags.push(
                                vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                                    | vk::DescriptorBindingFlags::PARTIALLY_BOUND
                                    | vk::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING,
                            );
                        }
                        vk::DescriptorSetLayoutBinding::builder()
                            .binding(binding.binding)
                            .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
                            .descriptor_count(binding.descriptor_count)
                            .stage_flags(binding.stage_flags)
                            .build()
                    }
                    DescriptorType::StorageImage => {
                        if binding.variable_count {
                            binding_flags.push(
                                vk::DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT
                                    | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                                    | vk::DescriptorBindingFlags::PARTIALLY_BOUND
                                    | vk::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING,
                            );
                        } else {
                            binding_flags.push(
                                vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                                    | vk::DescriptorBindingFlags::PARTIALLY_BOUND
                                    | vk::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING,
                            );
                        }
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

        let mut binding_flags = vk::DescriptorSetLayoutBindingFlagsCreateInfo::builder()
            .binding_flags(&binding_flags)
            .build();

        let mut info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(vk_bindings.as_slice())
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
            .push_next(&mut binding_flags);
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

    pub(crate) fn variable_descriptor_count(&self) -> u32 {
        let last_binding = self.inner.bindings.last().unwrap();
        if !last_binding.variable_count {
            0
        } else {
            last_binding.descriptor_count
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
