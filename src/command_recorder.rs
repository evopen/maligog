use crate::{Buffer, CommandBuffer, Device};
use ash::vk;

pub struct CommandRecorder<'a> {
    pub(crate) command_buffer: &'a CommandBuffer,
}

impl<'a> CommandRecorder<'a> {
    // pub fn copy_buffer(&mut self, src: &Buffer, dst: &Buffer, region: &[vk::BufferCopy]) {
    //     unsafe {
    //         self.copy_buffer_raw(src, dst, region);
    //     }
    //     self.command_buffer.resources.push(src);
    //     self.command_buffer.resources.push(dst);
    // }

    pub(crate) unsafe fn copy_buffer_raw(
        &mut self,
        src: &Buffer,
        dst: &Buffer,
        region: &[vk::BufferCopy],
    ) {
        unsafe {
            self.device_handle().cmd_copy_buffer(
                self.command_buffer.handle,
                src.inner.handle,
                dst.inner.handle,
                region,
            );
        }
    }

    pub(crate) fn build_acceleration_structure_raw(
        &mut self,
        info: vk::AccelerationStructureBuildGeometryInfoKHR,
        build_range_infos: &[vk::AccelerationStructureBuildRangeInfoKHR],
    ) {
        unsafe {
            self.device()
                .inner
                .acceleration_structure_loader
                .cmd_build_acceleration_structures(
                    self.command_buffer.handle,
                    &[info],
                    &[build_range_infos],
                );
        }
    }

    fn device_handle(&self) -> &ash::Device {
        &self.command_buffer.device.inner.handle
    }

    fn device(&self) -> &Device {
        &self.command_buffer.device
    }
}
