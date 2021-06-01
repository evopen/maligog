use crate::{
    Buffer, CommandBuffer, DescriptorSet, Device, Framebuffer, GraphicsPipeline, PipelineLayout,
    RenderPass,
};
use ash::vk;

pub struct CommandRecorder<'a> {
    pub(crate) command_buffer: &'a CommandBuffer,
    pub(crate) bind_point: Option<vk::PipelineBindPoint>,
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

    pub fn begin_render_pass<I>(
        &mut self,
        render_pass: &RenderPass,
        framebuffer: &Framebuffer,
        f: I,
    ) where
        I: FnOnce(&mut CommandRecorder),
    {
        unsafe {
            let info = vk::RenderPassBeginInfo::builder()
                .render_pass(render_pass.inner.handle)
                .framebuffer(framebuffer.inner.handle)
                .render_area(
                    vk::Rect2D::builder()
                        .extent(vk::Extent2D {
                            width: framebuffer.width(),
                            height: framebuffer.height(),
                        })
                        .build(),
                )
                .build();
            self.device().handle().cmd_begin_render_pass(
                self.command_buffer.handle,
                &info,
                vk::SubpassContents::INLINE,
            );

            f(self);

            self.device()
                .handle()
                .cmd_end_render_pass(self.command_buffer.handle);
            // self.command_buffer.resources.push(render_pass);
            // self.command_buffer.resources.push(framebuffer);
        }
    }

    pub fn bind_graphics_pipeline<I>(&mut self, pipeline: &GraphicsPipeline, f: I)
    where
        I: FnOnce(&mut CommandRecorder),
    {
        unsafe {
            self.device().handle().cmd_bind_pipeline(
                self.command_buffer.handle,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.inner.handle,
            );
            self.bind_point = Some(vk::PipelineBindPoint::GRAPHICS);
            f(self);
        }
        // self.command_buffer.resources.push(pipeline);
    }

    fn bind_descriptor_sets(
        &mut self,
        descriptor_sets: Vec<DescriptorSet>,
        layout: &PipelineLayout,
        first_set: u32,
    ) {
        unsafe {
            let descriptor_set_handles = descriptor_sets
                .iter()
                .map(|set| set.inner.handle)
                .collect::<Vec<_>>();
            self.device().handle().cmd_bind_descriptor_sets(
                self.command_buffer.handle,
                self.bind_point.unwrap(),
                layout.inner.handle,
                first_set,
                descriptor_set_handles.as_slice(),
                &[],
            );
        }

        // descriptor_sets
        //     .into_iter()
        //     .for_each(|set| self.command_buffer.resources.push(set));
    }

    fn device_handle(&self) -> &ash::Device {
        &self.command_buffer.device.inner.handle
    }

    fn device(&self) -> &Device {
        &self.command_buffer.device
    }
}
