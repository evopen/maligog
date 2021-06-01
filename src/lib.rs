#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unused))]

mod acceleration_structure;
pub mod buffer;
mod buffer_view;
pub mod command_buffer;
pub mod command_pool;
mod command_recorder;
pub mod descriptor;
mod descriptor_pool;
mod descriptor_set;
mod descriptor_set_layout;
pub mod device;
pub mod entry;
mod fence;
mod image;
mod image_view;
pub mod instance;
pub mod name;
pub mod physical_device;
mod pipeline;
mod pipeline_layout;
pub mod queue;
pub mod queue_family;
mod render_pass;
pub mod sampler;
mod semaphore;
mod shader_stage;
mod surface;
mod swapchain;

mod shader_module;

pub use acceleration_structure::AccelerationStructure;
pub use buffer::Buffer;
pub use buffer_view::BufferView;
pub use command_buffer::CommandBuffer;
pub use descriptor::Descriptor;
pub use descriptor::DescriptorType;
pub use descriptor_set::DescriptorSet;
pub use descriptor_set::DescriptorUpdate;
pub use descriptor_set_layout::DescriptorSetLayout;
pub use descriptor_set_layout::DescriptorSetLayoutBinding;
pub use device::Device;
pub use entry::Entry;
pub use fence::Fence;
pub use image::Image;
pub use image_view::ImageView;
pub use instance::Instance;
pub use render_pass::RenderPass;
pub use sampler::Sampler;
pub use semaphore::{BinarySemaphore, TimelineSemaphore};
pub use shader_module::ShaderModule;
pub use shader_stage::ShaderStage;
pub use surface::Surface;
pub use swapchain::Swapchain;

pub use ash::vk;
pub use ash::vk::{
    BufferUsageFlags, DescriptorPoolSize, ImageUsageFlags, PresentModeKHR, ShaderStageFlags,
};
pub use gpu_allocator::MemoryLocation;
