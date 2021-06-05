use std::ffi::c_void;

use ash::vk;

unsafe impl Sync for InstanceGeometry {}
unsafe impl Send for InstanceGeometry {}

pub struct InstanceGeometry {
    pub(crate) acceleration_structure_geometry: vk::AccelerationStructureGeometryKHR,
    pub(crate) build_range_info: vk::AccelerationStructureBuildRangeInfoKHR,
    pub(crate) instance_count: u32,
}

impl InstanceGeometry {
    pub fn new(
        bottoms: &[&super::BottomAccelerationStructure],
        transforms: &[vk::TransformMatrixKHR],
    ) -> Self {
        let instances = bottoms
            .iter()
            .zip(transforms)
            .map(|(bottom, transform)| {
                vk::AccelerationStructureInstanceKHR {
                    transform: transform.clone(),
                    instance_custom_index_and_mask: 0 | (0xFF << 24),
                    instance_shader_binding_table_record_offset_and_flags: 0
                        | (vk::GeometryInstanceFlagsKHR::TRIANGLE_FACING_CULL_DISABLE.as_raw()
                            << 24),
                    acceleration_structure_reference: vk::AccelerationStructureReferenceKHR {
                        device_handle: bottom.inner.device_address,
                    },
                }
            })
            .collect::<Vec<_>>();

        let geometry = vk::AccelerationStructureGeometryKHR::builder()
            .geometry_type(vk::GeometryTypeKHR::INSTANCES)
            .flags(
                vk::GeometryFlagsKHR::OPAQUE
                    | vk::GeometryFlagsKHR::NO_DUPLICATE_ANY_HIT_INVOCATION,
            )
            .geometry(vk::AccelerationStructureGeometryDataKHR {
                instances: vk::AccelerationStructureGeometryInstancesDataKHR::builder()
                    .array_of_pointers(false)
                    .data(vk::DeviceOrHostAddressConstKHR {
                        host_address: instances.as_ptr() as *const c_void,
                    })
                    .build(),
            })
            .build();

        let instance_count = instances.len() as u32;
        let build_range_info = vk::AccelerationStructureBuildRangeInfoKHR::builder()
            .primitive_count(instance_count)
            .primitive_offset(0) // offset to index buffer in bytes
            .build();
        Self {
            acceleration_structure_geometry: geometry,
            instance_count,
            build_range_info,
        }
    }
}
