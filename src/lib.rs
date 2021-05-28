#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unused))]

pub mod buffer;
pub mod command_buffer;
pub mod command_pool;
pub mod descriptor;
pub mod device;
pub mod entry;
pub mod instance;
pub mod name;
pub mod physical_device;
mod pipeline;
mod pipeline_layout;
pub mod queue;
pub mod queue_family;
mod render_pass;
pub mod sampler;
mod shader_stage;

mod shader_module;

pub use descriptor::DescriptorSetLayoutBinding;
pub use descriptor::DescriptorType;
pub use device::Device;
pub use render_pass::RenderPass;
pub use shader_module::ShaderModule;
pub use shader_stage::ShaderStage;

pub use ash::vk::{BufferUsageFlags, ShaderStageFlags};
pub use gpu_allocator::MemoryLocation;
