use crate::command_buffer::CommandBufferResource;
use crate::{
    Buffer, CommandBuffer, DescriptorSet, Device, Framebuffer, GraphicsPipeline, Image,
    PipelineLayout, RenderPass,
};
use ash::vk;

pub struct CommandRecorder<'a> {
    pub(crate) command_buffer: &'a mut CommandBuffer,
    pub(crate) bind_point: Option<vk::PipelineBindPoint>,
    pub(crate) pipeline_layout: Option<PipelineLayout>,
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
        self.command_buffer
            .resources
            .push(Box::new(render_pass.clone()));
        self.command_buffer
            .resources
            .push(Box::new(framebuffer.clone()));

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
            self.pipeline_layout = Some(pipeline.inner.layout.clone());
            f(self);
        }
        // self.command_buffer.resources.push(pipeline);
    }

    pub fn bind_descriptor_sets(&mut self, descriptor_sets: Vec<&DescriptorSet>, first_set: u32) {
        unsafe {
            let descriptor_set_handles = descriptor_sets
                .iter()
                .map(|set| set.inner.handle)
                .collect::<Vec<_>>();
            self.device().handle().cmd_bind_descriptor_sets(
                self.command_buffer.handle,
                self.bind_point.unwrap(),
                self.pipeline_layout.as_ref().unwrap().inner.handle,
                first_set,
                descriptor_set_handles.as_slice(),
                &[],
            );
        }

        // descriptor_sets
        //     .into_iter()
        //     .for_each(|set| self.command_buffer.resources.push(set));
    }

    pub fn set_scissor(&self, scissors: &[vk::Rect2D]) {
        unsafe {
            self.device()
                .handle()
                .cmd_set_scissor(self.command_buffer.handle, 0, scissors);
        }
    }

    pub fn set_viewport(&self, viewport: vk::Viewport) {
        unsafe {
            self.device()
                .handle()
                .cmd_set_viewport(self.command_buffer.handle, 0, &[viewport]);
        }
    }

    pub fn bind_index_buffer(&mut self, buffer: Buffer, offset: u64, index_type: vk::IndexType) {
        unsafe {
            self.device().handle().cmd_bind_index_buffer(
                self.command_buffer.handle,
                buffer.inner.handle,
                offset,
                index_type,
            );
        }
        // self.command_buffer.resources.push(buffer);
    }

    pub fn bind_vertex_buffer(&mut self, buffers: Vec<Buffer>, offsets: &[u64]) {
        let buffer_handles = buffers.iter().map(|b| b.inner.handle).collect::<Vec<_>>();
        unsafe {
            self.device().handle().cmd_bind_vertex_buffers(
                self.command_buffer.handle,
                0,
                buffer_handles.as_slice(),
                offsets,
            );
        }
        // buffers
        //     .into_iter()
        //     .for_each(|b| self.command_buffer.resources.push(b));
    }

    pub fn draw_indexed(&self, index_count: u32, instance_count: u32) {
        unsafe {
            self.device().handle().cmd_draw_indexed(
                self.command_buffer.handle,
                index_count,
                instance_count,
                0,
                0,
                0,
            );
        }
    }

    pub(crate) unsafe fn copy_buffer_to_image_raw(
        &mut self,
        src: &Buffer,
        dst: &Image,
        regions: &[vk::BufferImageCopy],
    ) {
        self.device().handle().cmd_copy_buffer_to_image(
            self.command_buffer.handle,
            src.inner.handle,
            dst.inner.handle,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            regions,
        );
    }

    pub(crate) unsafe fn copy_image_raw(
        &mut self,
        src: vk::Image,
        dst: vk::Image,
        regions: &[vk::ImageCopy],
    ) {
        self.device().handle().cmd_copy_image(
            self.command_buffer.handle,
            src,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            dst,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            regions,
        );
    }

    pub(crate) fn pipeline_barrier(&mut self, dependency_info: &vk::DependencyInfoKHR) {
        unsafe {
            self.device()
                .synchronization2_loader()
                .cmd_pipeline_barrier2(self.command_buffer.handle, dependency_info);
        }
    }

    pub fn clear_attachments(
        &mut self,
        attachments: &[vk::ClearAttachment],
        rects: &[vk::ClearRect],
    ) {
        unsafe {
            self.device().handle().cmd_clear_attachments(
                self.command_buffer.handle,
                attachments,
                rects,
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
