use std::ffi::CString;
use std::process::Command;
use std::sync::Arc;

use ash::vk;
use ash::vk::Handle;

use crate::command_pool::CommandPool;
use crate::command_recorder::CommandRecorder;
use crate::device::Device;

pub trait CommandBufferResource {}

impl CommandBufferResource for crate::Framebuffer {}
impl CommandBufferResource for crate::RenderPass {}

pub struct CommandBuffer {
    pub(crate) device: Device,
    pub(crate) handle: vk::CommandBuffer,
    pub(crate) resources: Vec<Box<dyn CommandBufferResource>>,
}

impl CommandBuffer {
    pub(crate) fn new(name: Option<&str>, device: &Device, command_pool: &CommandPool) -> Self {
        unsafe {
            let handle = device
                .inner
                .handle
                .allocate_command_buffers(
                    &vk::CommandBufferAllocateInfo::builder()
                        .command_pool(command_pool.inner.handle)
                        .command_buffer_count(1)
                        .level(vk::CommandBufferLevel::PRIMARY)
                        .build(),
                )
                .unwrap()
                .first()
                .unwrap()
                .to_owned();
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
                        device.inner.handle.handle(),
                        &vk::DebugUtilsObjectNameInfoEXT::builder()
                            .object_handle(handle.as_raw())
                            .object_type(vk::ObjectType::COMMAND_BUFFER)
                            .object_name(CString::new(name).unwrap().as_ref())
                            .build(),
                    )
                    .unwrap();
            }

            Self {
                handle,
                device: device.clone(),
                resources: Vec::new(),
            }
        }
    }

    // pub fn begin(&self) {
    //     unsafe {
    //         self.device
    //             .inner
    //             .handle
    //             .begin_command_buffer(self.handle, &vk::CommandBufferBeginInfo::default())
    //             .unwrap();
    //     }
    // }

    // pub fn end(&self) {
    //     unsafe {
    //         self.device
    //             .inner
    //             .handle
    //             .end_command_buffer(self.handle)
    //             .unwrap();
    //     }
    // }

    pub fn encode<F>(&mut self, func: F)
    where
        F: FnOnce(&mut CommandRecorder),
    {
        unsafe {
            let device = self.device.inner.handle.clone();
            device
                .begin_command_buffer(self.handle, &vk::CommandBufferBeginInfo::default())
                .unwrap();

            let mut recorder = CommandRecorder {
                command_buffer: self,
                bind_point: None,
                pipeline_layout: None,
            };
            func(&mut recorder);
            device.end_command_buffer(self.handle).unwrap();
        }
    }
}

impl Device {
    pub fn create_command_buffer(
        &self,
        name: Option<&str>,
        queue_family_index: u32,
    ) -> CommandBuffer {
        CommandBuffer::new(name, self, &self.command_pool(queue_family_index))
    }
}

#[test]
fn test_command_buffer() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .try_init()
        .ok();
    use crate::entry::Entry;

    let entry = Entry::new().unwrap();
    let instance = entry.create_instance(&[], &[]);
    let _pdevices = instance.enumerate_physical_device();

    let pdevice = instance
        .enumerate_physical_device()
        .into_iter()
        .find(|p| {
            p.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
                && p.queue_families
                    .iter()
                    .any(|f| f.support_compute() && f.support_graphics())
        })
        .unwrap();
    let pdevice = pdevice;
    let queue_family = pdevice
        .queue_families()
        .iter()
        .find(|f| f.support_graphics() && f.support_compute())
        .unwrap();
    let device = pdevice.create_device(&[(&queue_family, &[1.0])]);
    // device.allocate_command_buffer();
}
