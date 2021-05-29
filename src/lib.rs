#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unused))]

pub mod buffer;
pub mod command_buffer;
pub mod command_pool;
pub mod descriptor;
mod descriptor_pool;
// mod descriptor_set;
pub mod device;
pub mod entry;
mod image;
pub mod instance;
pub mod name;
pub mod physical_device;
// mod pipeline;
mod pipeline_layout;
pub mod queue;
pub mod queue_family;
// mod render_pass;
mod command_recorder;
mod fence;
pub mod sampler;
mod semaphore;
mod shader_stage;
mod surface;
mod swapchain;

mod shader_module;

pub use descriptor::DescriptorSetLayoutBinding;
pub use descriptor::DescriptorType;
pub use device::Device;
pub use instance::Instance;
// pub use render_pass::RenderPass;
// pub use command_pool::CommandPool;
pub use buffer::Buffer;
pub use command_buffer::CommandBuffer;
pub use entry::Entry;
pub use fence::Fence;
pub use semaphore::{BinarySemaphore, TimelineSemaphore};
pub use shader_module::ShaderModule;
pub use shader_stage::ShaderStage;
pub use surface::Surface;
pub use swapchain::Swapchain;

pub use ash::vk;
pub use ash::vk::{BufferUsageFlags, DescriptorPoolSize, PresentModeKHR, ShaderStageFlags};
pub use gpu_allocator::MemoryLocation;
