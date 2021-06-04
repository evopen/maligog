use std::ffi::CString;
use std::sync::Arc;

use ash::vk;
use ash::vk::Handle;

use crate::Device;

pub(crate) struct TopAccelerationStructureRef {
    pub(crate) handle: vk::AccelerationStructureKHR,
    device_address: u64,
    device: Device,
}

#[derive(Clone)]
pub struct TopAccelerationStructure {
    pub(crate) inner: Arc<TopAccelerationStructureRef>,
}

impl TopAccelerationStructure {
    pub(crate) fn new(
        name: Option<&str>,
        device: &Device,
        geometries: &[&super::InstanceGeometry],
    ) {
        let vk_geometries = geometries
            .iter()
            .map(|t| t.acceleration_structure_geometry)
            .collect::<Vec<_>>();
        let instance_counts = geometries
            .iter()
            .map(|t| t.instance_count)
            .collect::<Vec<_>>();
        unsafe {
            let size_info = device
                .inner
                .acceleration_structure_loader
                .get_acceleration_structure_build_sizes(
                    vk::AccelerationStructureBuildTypeKHR::DEVICE,
                    &vk::AccelerationStructureBuildGeometryInfoKHR::builder()
                        .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
                        .ty(vk::AccelerationStructureTypeKHR::TOP_LEVEL)
                        .geometries(&vk_geometries)
                        .build(),
                    &instance_counts,
                );
            let as_buffer = device.create_buffer(
                Some(&format!(
                    "{} buffer",
                    name.unwrap_or("acceleration structure")
                )),
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
                        .ty(vk::AccelerationStructureTypeKHR::TOP_LEVEL)
                        .buffer(as_buffer.inner.handle)
                        .size(size_info.acceleration_structure_size)
                        .build(),
                    None,
                )
                .unwrap();

            if let Some(name) = name {
                device.debug_set_object_name(
                    name,
                    handle.as_raw(),
                    vk::ObjectType::ACCELERATION_STRUCTURE_KHR,
                );
            }
            let scratch_buffer = device.create_buffer(
                Some(&format!(
                    "{} scratch buffer",
                    name.unwrap_or("acceleration structure")
                )),
                size_info.build_scratch_size,
                vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                    | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
                crate::MemoryLocation::GpuOnly,
            );

            let build_geometry_info = vk::AccelerationStructureBuildGeometryInfoKHR::builder()
                .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
                .ty(vk::AccelerationStructureTypeKHR::TOP_LEVEL)
                .geometries(&vk_geometries)
                .dst_acceleration_structure(handle)
                .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
                .scratch_data(vk::DeviceOrHostAddressKHR {
                    device_address: scratch_buffer.device_address(),
                })
                .build();

            let build_range_infos = geometries
                .iter()
                .map(|t| t.build_range_info)
                .collect::<Vec<_>>();

            let mut cmd_buf = device.create_command_buffer(
                Some("build acceleration structure"),
                device.compute_queue_family_index(),
            );
            cmd_buf.encode(|recorder| {
                recorder.build_acceleration_structure_raw(build_geometry_info, &build_range_infos);
            });
            device.compute_queue().submit_blocking(&[cmd_buf]);
        }
    }
}
