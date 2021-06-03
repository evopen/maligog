use std::ffi::CString;
use std::sync::Arc;

use ash::vk;
use ash::vk::Handle;

use crate::CommandBuffer;
use crate::{Buffer, Device};

pub(crate) struct AccelerationStructureRef {
    pub(crate) handle: vk::AccelerationStructureKHR,
    as_buffer: Buffer,
    device_address: u64,
    device: Device,
}

#[derive(Clone)]
pub struct AccelerationStructure {
    pub(crate) inner: Arc<AccelerationStructureRef>,
}

impl AccelerationStructure {
    pub fn new(
        name: Option<&str>,
        device: &Device,
        geometries: &[vk::AccelerationStructureGeometryKHR],
        primitive_counts: &[u32],
        as_type: vk::AccelerationStructureTypeKHR,
    ) -> Self {
        assert_eq!(geometries.len(), primitive_counts.len());
        unsafe {
            let size_info = device
                .inner
                .acceleration_structure_loader
                .get_acceleration_structure_build_sizes(
                    vk::AccelerationStructureBuildTypeKHR::DEVICE,
                    &vk::AccelerationStructureBuildGeometryInfoKHR::builder()
                        .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
                        .ty(as_type)
                        .geometries(geometries)
                        .build(),
                    primitive_counts,
                );
            let as_buffer = Buffer::new(
                Some(&format!(
                    "{} buffer",
                    name.unwrap_or("acceleration structure")
                )),
                &device,
                size_info.acceleration_structure_size,
                vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                    | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
                gpu_allocator::MemoryLocation::GpuOnly,
            );

            let handle = device
                .inner
                .acceleration_structure_loader
                .create_acceleration_structure(
                    &vk::AccelerationStructureCreateInfoKHR::builder()
                        .ty(as_type)
                        .buffer(as_buffer.handle())
                        .size(size_info.acceleration_structure_size)
                        .build(),
                    None,
                )
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
                        device.handle().handle(),
                        &vk::DebugUtilsObjectNameInfoEXT::builder()
                            .object_handle(handle.as_raw())
                            .object_type(vk::ObjectType::ACCELERATION_STRUCTURE_KHR)
                            .object_name(CString::new(name).unwrap().as_ref())
                            .build(),
                    )
                    .unwrap();
            }

            let scratch_buffer = Buffer::new(
                Some(&format!(
                    "{} scratch buffer",
                    name.unwrap_or("acceleration structure")
                )),
                &device,
                size_info.build_scratch_size,
                vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                    | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
                crate::MemoryLocation::GpuOnly,
            );

            let build_geometry_info = vk::AccelerationStructureBuildGeometryInfoKHR::builder()
                .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
                .ty(as_type)
                .geometries(geometries)
                .dst_acceleration_structure(handle)
                .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
                .scratch_data(vk::DeviceOrHostAddressKHR {
                    device_address: scratch_buffer.device_address(),
                })
                .build();

            let build_range_infos = primitive_counts
                .iter()
                .map(|count| {
                    vk::AccelerationStructureBuildRangeInfoKHR::builder()
                        .first_vertex(0)
                        .primitive_offset(0)
                        .transform_offset(0)
                        .primitive_count(*count)
                        .build()
                })
                .collect::<Vec<_>>();

            let device_address = device
                .inner
                .acceleration_structure_loader
                .get_acceleration_structure_device_address(
                    &vk::AccelerationStructureDeviceAddressInfoKHR::builder()
                        .acceleration_structure(handle)
                        .build(),
                );
            let result = Self {
                inner: Arc::new(AccelerationStructureRef {
                    handle,
                    as_buffer,
                    device_address,
                    device: device.clone(),
                }),
            };

            let mut command_buffer =
                device.create_command_buffer(device.graphics_queue_family_index());
            command_buffer.encode(|recorder| {
                recorder.build_acceleration_structure_raw(
                    build_geometry_info,
                    build_range_infos.as_ref(),
                )
            });

            device.graphics_queue().submit_blocking(&[command_buffer]);

            result
        }
    }

    pub fn device_address(&self) -> u64 {
        self.inner.device_address
    }
}

impl Drop for AccelerationStructureRef {
    fn drop(&mut self) {
        unsafe {
            self.device
                .inner
                .acceleration_structure_loader
                .destroy_acceleration_structure(self.handle, None);
        }
    }
}
