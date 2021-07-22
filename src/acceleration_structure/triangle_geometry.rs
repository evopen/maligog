use std::convert::TryInto;
use std::ffi::c_void;

use ash::vk;

unsafe impl Sync for TriangleGeometry {}
unsafe impl Send for TriangleGeometry {}

#[derive(Clone)]
pub struct TriangleGeometry {
    pub(crate) acceleration_structure_geometry: vk::AccelerationStructureGeometryKHR,
    index_buffer_view: crate::IndexBufferView,
    vertex_buffer_view: crate::VertexBufferView,
    transform_buffer_view: Option<crate::BufferView>,
    pub(crate) vertex_count: u32,
    pub(crate) triangle_count: u32,
    pub(crate) build_range_info: vk::AccelerationStructureBuildRangeInfoKHR,
}

impl super::Geometry for TriangleGeometry {
    fn build_range_info(&self) -> vk::AccelerationStructureBuildRangeInfoKHR {
        self.build_range_info
    }

    fn geometry(&self) -> vk::AccelerationStructureGeometryKHR {
        self.acceleration_structure_geometry
    }

    fn primitives_count(&self) -> u32 {
        self.triangle_count
    }
}

impl TriangleGeometry {
    pub fn new(
        index_buffer_view: &crate::IndexBufferView,
        vertex_buffer_view: &crate::VertexBufferView,
        transform_buffer_view: Option<&crate::BufferView>,
    ) -> Self {
        let mut triangles_data = vk::AccelerationStructureGeometryTrianglesDataKHR::builder()
            .index_type(index_buffer_view.index_type)
            .index_data(vk::DeviceOrHostAddressConstKHR {
                device_address: index_buffer_view.buffer_view.buffer.device_address()
                    + index_buffer_view.buffer_view.offset,
            })
            .vertex_data(vk::DeviceOrHostAddressConstKHR {
                device_address: vertex_buffer_view.buffer_view.buffer.device_address()
                    + vertex_buffer_view.buffer_view.offset,
            })
            .vertex_format(vertex_buffer_view.format)
            .vertex_stride(vertex_buffer_view.stride)
            .max_vertex(vertex_buffer_view.count)
            .build();
        let mut vk_transform: Option<[f32; 12]> = None;
        if let Some(view) = transform_buffer_view {
            triangles_data.transform_data = vk::DeviceOrHostAddressConstKHR {
                device_address: view.buffer.device_address() + view.offset,
            }
        }
        let geometry = vk::AccelerationStructureGeometryKHR::builder()
            .geometry_type(vk::GeometryTypeKHR::TRIANGLES)
            .flags(
                vk::GeometryFlagsKHR::OPAQUE
                    | vk::GeometryFlagsKHR::NO_DUPLICATE_ANY_HIT_INVOCATION,
            )
            .geometry(vk::AccelerationStructureGeometryDataKHR {
                triangles: triangles_data,
            })
            .build();

        assert!(index_buffer_view.count % 3 == 0);
        let triangle_count = index_buffer_view.count / 3;
        let vertex_count = vertex_buffer_view.count;
        let build_range_info = vk::AccelerationStructureBuildRangeInfoKHR::builder()
            .primitive_count(triangle_count)
            .primitive_offset(0) // offset to index buffer in bytes
            .first_vertex(0)
            .transform_offset(0)
            .build();
        Self {
            acceleration_structure_geometry: geometry,
            index_buffer_view: index_buffer_view.clone(),
            vertex_buffer_view: vertex_buffer_view.clone(),
            transform_buffer_view: transform_buffer_view.map(|v| v.to_owned()),
            vertex_count,
            triangle_count,
            build_range_info,
        }
    }

    pub fn index_buffer_view(&self) -> &crate::IndexBufferView {
        &self.index_buffer_view
    }

    pub fn vertex_buffer_view(&self) -> &crate::VertexBufferView {
        &self.vertex_buffer_view
    }
}
