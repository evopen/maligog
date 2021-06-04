mod bottom;
mod instance_geometry;
mod top;
mod triangle_geometry;

pub use bottom::BottomAccelerationStructure;
pub use instance_geometry::InstanceGeometry;
pub use top::TopAccelerationStructure;
pub use triangle_geometry::TriangleGeometry;

use std::ffi::CString;
use std::sync::Arc;

use ash::vk;
use ash::vk::Handle;
