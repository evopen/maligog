use std::ffi::CString;
use std::mem::ManuallyDrop;
use std::sync::Arc;
use std::sync::Mutex;

use ash::vk;
use thread_local::ThreadLocal;

use crate::buffer::Buffer;
use crate::command_pool::CommandPool;
use crate::instance::Instance;
use crate::name;
use crate::physical_device::PhysicalDevice;
use crate::queue_family::QueueFamily;

pub struct DeviceFeatures {}

pub(crate) struct DeviceRef {
    pub handle: ash::Device,
    pub pdevice: PhysicalDevice,
    acceleration_structure_loader: ash::extensions::khr::AccelerationStructure,
    swapchain_loader: ash::extensions::khr::Swapchain,
    ray_tracing_pipeline_loader: ash::extensions::khr::RayTracingPipeline,
    pub(crate) allocator: Mutex<ManuallyDrop<gpu_allocator::VulkanAllocator>>,
    command_pool: ThreadLocal<CommandPool>,
}

#[derive(Clone)]
pub struct Device {
    pub(crate) inner: Arc<DeviceRef>,
}

impl Device {
    pub(crate) fn new(
        instance: Instance,
        pdevice: PhysicalDevice,
        _device_features: &DeviceFeatures,
        device_extensions: &[name::device::Extension],
        queues: &[(&QueueFamily, &[f32])],
    ) -> Self {
        unsafe {
            let mut queue_infos = Vec::new();
            for (family, priorities) in queues {
                queue_infos.push(
                    vk::DeviceQueueCreateInfo::builder()
                        .queue_family_index(family.index)
                        .queue_priorities(&priorities)
                        .build(),
                )
            }

            let device_extension_names = device_extensions
                .iter()
                .map(|extension| CString::new(extension.as_ref()).unwrap())
                .collect::<Vec<_>>();
            let device_extension_names_raw: Vec<*const i8> = device_extension_names
                .iter()
                .map(|raw_name| raw_name.as_ptr())
                .collect();

            let mut ray_tracing_pipeline_pnext =
                vk::PhysicalDeviceRayTracingPipelineFeaturesKHR::builder()
                    .ray_tracing_pipeline(true)
                    .build();
            let mut acceleration_structure_pnext =
                vk::PhysicalDeviceAccelerationStructureFeaturesKHR::builder()
                    .acceleration_structure(true)
                    .build();
            let mut ray_query_pnext = vk::PhysicalDeviceRayQueryFeaturesKHR::builder()
                .ray_query(true)
                .build();
            let mut device_buffer_address_pnext =
                vk::PhysicalDeviceBufferDeviceAddressFeatures::builder()
                    .buffer_device_address(true)
                    .build();
            let mut fea_16_bit_storage_pnext = vk::PhysicalDevice16BitStorageFeatures::builder()
                .uniform_and_storage_buffer16_bit_access(true)
                .storage_buffer16_bit_access(true)
                .storage_input_output16(false)
                .storage_push_constant16(true)
                .build();
            let mut scalar_block_layout_pnext =
                vk::PhysicalDeviceScalarBlockLayoutFeatures::builder()
                    .scalar_block_layout(true)
                    .build();

            let vk_device_features = vk::PhysicalDeviceFeatures {
                ..Default::default()
            };

            let mut device_create_info = vk::DeviceCreateInfo::builder()
                .queue_create_infos(&queue_infos)
                .enabled_extension_names(&device_extension_names_raw)
                .enabled_features(&vk_device_features);

            device_create_info =
                if device_extensions.contains(&name::device::Extension::KhrRayTracingPipeline) {
                    device_create_info.push_next(&mut ray_tracing_pipeline_pnext)
                } else {
                    device_create_info
                };
            device_create_info =
                if device_extensions.contains(&name::device::Extension::KhrRayQuery) {
                    device_create_info.push_next(&mut ray_query_pnext)
                } else {
                    device_create_info
                };
            device_create_info =
                if device_extensions.contains(&name::device::Extension::KhrAccelerationStructure) {
                    device_create_info.push_next(&mut acceleration_structure_pnext)
                } else {
                    device_create_info
                };

            device_create_info = device_create_info
                .push_next(&mut device_buffer_address_pnext)
                .push_next(&mut fea_16_bit_storage_pnext)
                .push_next(&mut scalar_block_layout_pnext);

            let handle = instance
                .inner
                .handle
                .create_device(pdevice.handle, &device_create_info, None)
                .unwrap();

            let acceleration_structure_loader = ash::extensions::khr::AccelerationStructure::new(
                &pdevice.instance.inner.handle,
                &handle,
            );

            let swapchain_loader =
                ash::extensions::khr::Swapchain::new(&pdevice.instance.inner.handle, &handle);

            let ray_tracing_pipeline_loader = ash::extensions::khr::RayTracingPipeline::new(
                &pdevice.instance.inner.handle,
                &handle,
            );

            let allocator =
                gpu_allocator::VulkanAllocator::new(&gpu_allocator::VulkanAllocatorCreateDesc {
                    instance: instance.inner.handle.clone(),
                    device: handle.clone(),
                    physical_device: pdevice.handle,
                    debug_settings: gpu_allocator::AllocatorDebugSettings {
                        log_memory_information: false,
                        log_leaks_on_shutdown: true,
                        store_stack_traces: false,
                        log_allocations: true,
                        log_frees: true,
                        log_stack_traces: false,
                    },
                });

            Self {
                inner: Arc::new(DeviceRef {
                    handle,
                    pdevice,
                    acceleration_structure_loader,
                    swapchain_loader,
                    ray_tracing_pipeline_loader,
                    allocator: Mutex::new(ManuallyDrop::new(allocator)),
                    command_pool: ThreadLocal::new(),
                }),
            }
        }
    }

    pub fn create_buffer<I>(
        &self,
        name: Option<&str>,
        size: I,
        buffer_usage: vk::BufferUsageFlags,
        location: gpu_allocator::MemoryLocation,
    ) -> Buffer
    where
        I: num_traits::PrimInt,
    {
        Buffer::new(name, self.clone(), size, buffer_usage, location)
    }

    pub(crate) fn command_pool(&self) -> &CommandPool {
        self.inner
            .command_pool
            .get_or(|| CommandPool::new(self.clone(), 0))
    }

    pub fn allocate_command_buffer(&self) {
        let command_pool = self.command_pool();
        let a = command_pool.allocate_command_buffer();
    }
}

impl Drop for DeviceRef {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.allocator.lock().unwrap());
            self.handle.destroy_device(None);
        }
    }
}

#[test]
fn test_create_command_buffer() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .try_init()
        .ok();
    use crate::entry::Entry;

    let entry = Entry::new().unwrap();
    let instance = entry.create_instance(&[], &[]);
    let _pdevices = instance.enumerate_physical_device();

    let pdevice = instance
        .enumerate_physical_device()
        .into_iter()
        .find(|p| {
            p.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
                && p.queue_families
                    .iter()
                    .any(|f| f.support_compute() && f.support_graphics())
        })
        .unwrap();
    let pdevice = Arc::new(pdevice);
    let queue_family = pdevice
        .queue_families()
        .iter()
        .find(|f| f.support_graphics() && f.support_compute())
        .unwrap();
    let device = pdevice.create_device(&[(&queue_family, &[1.0])]);
    device.allocate_command_buffer();
}
