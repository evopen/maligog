use crate::device::Device;
use crate::sampler::Sampler;
use ash::vk::{self, Handle};
use std::ffi::CString;
use std::sync::Arc;

pub trait Descriptor {}

#[derive(Clone)]
pub enum DescriptorType {
    Sampler(Option<Sampler>),
    SampledImage,
    UniformBuffer,
    StorageBuffer,
    AccelerationStructure,
    StorageImage,
}
