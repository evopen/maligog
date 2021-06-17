use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ffi::CString;
use std::iter::FromIterator;
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
use crate::queue;
use crate::queue::Queue;
use crate::queue_family::QueueFamily;
use crate::queue_family::QueueFamilyProperties;
use crate::CommandBuffer;

pub struct DeviceFeatures {}

pub(crate) struct DeviceRef {
    pub handle: ash::Device,
    pub pdevice: PhysicalDevice,
    pub(crate) acceleration_structure_loader: ash::extensions::khr::AccelerationStructure,
    pub(crate) swapchain_loader: ash::extensions::khr::Swapchain,
    ray_tracing_pipeline_loader: ash::extensions::khr::RayTracingPipeline,
    synchronization2_loader: ash::extensions::khr::Synchronization2,
    pub(crate) allocator: Mutex<ManuallyDrop<gpu_allocator::VulkanAllocator>>,
    graphics_queue: ManuallyDrop<Queue>,
    transfer_queue: ManuallyDrop<Queue>,
    compute_queue: ManuallyDrop<Queue>,
    command_pool: ManuallyDrop<ThreadLocal<RefCell<BTreeMap<u32, CommandPool>>>>,
    all_queue_family_indices: Vec<u32>,
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
    ) -> Self {
        unsafe {
            let graphics_queue_family_properties = pdevice.graphics_queue_family();
            let transfer_queue_family_properties = pdevice.transfer_queue_family();
            let compute_queue_family_properties = pdevice.compute_queue_family();
            let mut queue_infos = Vec::new();
            queue_infos.push(
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(graphics_queue_family_properties.index)
                    .queue_priorities(&[1.0])
                    .build(),
            );
            queue_infos.push(
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(compute_queue_family_properties.index)
                    .queue_priorities(&[1.0])
                    .build(),
            );
            queue_infos.push(
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(transfer_queue_family_properties.index)
                    .queue_priorities(&[1.0])
                    .build(),
            );

            let mut device_extensions = device_extensions.to_vec();
            device_extensions.push(crate::name::device::Extension::KhrSynchronization2);
            device_extensions.push(crate::name::device::Extension::KhrVulkanMemoryModel);
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
            let mut vulkan_memory_model_pnext =
                vk::PhysicalDeviceVulkanMemoryModelFeatures::builder()
                    .vulkan_memory_model(true)
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
            let mut shader_float16_int8_pnext =
                vk::PhysicalDeviceShaderFloat16Int8Features::builder()
                    .shader_int8(true)
                    .build();
            let mut descriptor_indexing_pnext =
                vk::PhysicalDeviceDescriptorIndexingFeatures::builder()
                    .runtime_descriptor_array(true)
                    .descriptor_binding_variable_descriptor_count(true)
                    .shader_storage_image_array_non_uniform_indexing(true)
                    .shader_sampled_image_array_non_uniform_indexing(true)
                    .shader_storage_buffer_array_non_uniform_indexing(true)
                    .shader_uniform_buffer_array_non_uniform_indexing(true)
                    .build();

            let vk_device_features = vk::PhysicalDeviceFeatures {
                shader_storage_image_write_without_format: vk::TRUE,
                shader_storage_image_read_without_format: vk::TRUE,
                fill_mode_non_solid: vk::TRUE,
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
                .push_next(&mut scalar_block_layout_pnext)
                .push_next(&mut vulkan_memory_model_pnext)
                .push_next(&mut shader_float16_int8_pnext)
                .push_next(&mut descriptor_indexing_pnext);

            let handle = instance
                .inner
                .handle
                .create_device(pdevice.handle, &device_create_info, None)
                .unwrap();

            let acceleration_structure_loader = ash::extensions::khr::AccelerationStructure::new(
                &pdevice.instance.inner.handle,
                &handle,
            );

            let synchronization2_loader = ash::extensions::khr::Synchronization2::new(
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

            let graphics_queue = Queue::new(
                &handle,
                synchronization2_loader.clone(),
                &graphics_queue_family_properties,
                0,
            );
            log::debug!(
                "graphics queue family index: {}",
                graphics_queue_family_properties.index
            );
            let compute_queue = Queue::new(
                &handle,
                synchronization2_loader.clone(),
                &compute_queue_family_properties,
                0,
            );
            log::debug!(
                "compute queue family index: {}",
                compute_queue_family_properties.index
            );
            let transfer_queue = Queue::new(
                &handle,
                synchronization2_loader.clone(),
                &transfer_queue_family_properties,
                0,
            );
            log::debug!(
                "transfer queue family index: {}",
                transfer_queue_family_properties.index
            );
            let all_queue_family_indices = std::collections::BTreeSet::from_iter([
                graphics_queue_family_properties.index,
                compute_queue_family_properties.index,
                transfer_queue_family_properties.index,
            ])
            .into_iter()
            .collect();

            Self {
                inner: Arc::new(DeviceRef {
                    handle,
                    pdevice,
                    graphics_queue: ManuallyDrop::new(graphics_queue),
                    compute_queue: ManuallyDrop::new(compute_queue),
                    transfer_queue: ManuallyDrop::new(transfer_queue),
                    acceleration_structure_loader,
                    synchronization2_loader,
                    swapchain_loader,
                    ray_tracing_pipeline_loader,
                    allocator: Mutex::new(ManuallyDrop::new(allocator)),
                    command_pool: ManuallyDrop::new(ThreadLocal::new()),
                    all_queue_family_indices,
                }),
            }
        }
    }

    pub(crate) fn command_pool(&self, queue_family_index: u32) -> CommandPool {
        let mut pools = self.inner.command_pool.get_or_default().borrow_mut();
        let pool = pools
            .entry(queue_family_index)
            .or_insert_with(|| CommandPool::new(self.clone(), queue_family_index));
        pool.clone()
    }

    pub(crate) fn handle(&self) -> &ash::Device {
        &self.inner.handle
    }

    // pub fn allocate_command_buffer(&self) {
    //     let command_pool = self.command_pool();
    //     let a = command_pool.allocate_command_buffer();
    // }

    pub fn transfer_queue_family_index(&self) -> u32 {
        self.inner
            .transfer_queue
            .inner
            .queue_family_properties
            .index
    }

    pub fn graphics_queue_family_index(&self) -> u32 {
        self.inner
            .graphics_queue
            .inner
            .queue_family_properties
            .index
    }

    pub fn compute_queue_family_index(&self) -> u32 {
        self.inner.compute_queue.inner.queue_family_properties.index
    }

    pub fn transfer_queue(&self) -> &Queue {
        &self.inner.transfer_queue
    }

    pub fn graphics_queue(&self) -> &Queue {
        &self.inner.graphics_queue
    }

    pub fn compute_queue(&self) -> &Queue {
        &self.inner.compute_queue
    }

    pub fn wait_idle(&self) {
        unsafe {
            self.handle().device_wait_idle().unwrap();
        }
    }

    pub fn synchronization2_loader(&self) -> &ash::extensions::khr::Synchronization2 {
        &self.inner.synchronization2_loader
    }

    pub(crate) fn all_queue_family_indices(&self) -> &[u32] {
        &self.inner.all_queue_family_indices
    }

    pub(crate) fn debug_set_object_name(
        &self,
        name: &str,
        object_handle: u64,
        object_type: vk::ObjectType,
    ) {
        unsafe {
            self.inner
                .pdevice
                .instance
                .inner
                .debug_utils_loader
                .debug_utils_set_object_name(
                    self.handle().handle(),
                    &vk::DebugUtilsObjectNameInfoEXT::builder()
                        .object_handle(object_handle)
                        .object_type(object_type)
                        .object_name(CString::new(name).unwrap().as_ref())
                        .build(),
                )
                .unwrap();
        }
    }

    pub(crate) fn ray_tracing_pipeline_loader(&self) -> &ash::extensions::khr::RayTracingPipeline {
        &self.inner.ray_tracing_pipeline_loader
    }
}

impl Drop for DeviceRef {
    fn drop(&mut self) {
        log::debug!("dropping device");

        unsafe {
            self.handle.device_wait_idle().unwrap();
            ManuallyDrop::drop(&mut self.graphics_queue);
            ManuallyDrop::drop(&mut self.compute_queue);
            ManuallyDrop::drop(&mut self.transfer_queue);
            ManuallyDrop::drop(&mut self.command_pool);

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
    let device = pdevice.create_device();
    // device.allocate_command_buffer();
}
