mod aabb_geometry;
mod triangle_geometry;

mod blas_instance;
mod bottom;
mod instance_geometry;
mod top;

pub use aabb_geometry::AABBGeometry;
pub use blas_instance::BLASInstance;
pub use bottom::BottomAccelerationStructure;
pub use instance_geometry::InstanceGeometry;
pub use top::TopAccelerationStructure;
pub use triangle_geometry::TriangleGeometry;

use std::ffi::CString;
use std::sync::Arc;

use ash::vk;
use ash::vk::Handle;

pub trait Geometry {
    fn build_range_info(&self) -> vk::AccelerationStructureBuildRangeInfoKHR;
    fn geometry(&self) -> vk::AccelerationStructureGeometryKHR;
    fn primitives_count(&self) -> u32;
}
