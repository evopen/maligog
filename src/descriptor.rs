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

impl Descriptor for crate::Sampler {}
impl Descriptor for crate::ImageView {}
impl Descriptor for crate::BufferView {}
impl Descriptor for crate::TopAccelerationStructure {}
