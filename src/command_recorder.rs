use crate::{Buffer, CommandBuffer};
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

    fn device_handle(&self) -> &ash::Device {
        &self.command_buffer.device.inner.handle
    }
}
