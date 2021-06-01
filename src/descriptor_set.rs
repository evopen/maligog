use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ffi::CString;
use std::sync::Arc;

use ash::vk;
use ash::vk::Handle;

use anyhow::Context;

use crate::buffer::Buffer;
use crate::descriptor_pool::DescriptorPool;
use crate::sampler::Sampler;
use crate::AccelerationStructure;
use crate::Descriptor;
use crate::DescriptorSetLayout;
use crate::Device;
use crate::ImageView;

pub struct DescriptorSetRef {
    handle: vk::DescriptorSet,
    device: Device,
    descriptor_pool: DescriptorPool,
    descriptor_set_layout: DescriptorSetLayout,
    resources: BTreeMap<u32, Vec<Arc<dyn Descriptor>>>,
}

impl DescriptorSetRef {
    fn new(
        device: &Device,
        name: Option<&str>,
        descriptor_pool: DescriptorPool,
        descriptor_set_layout: DescriptorSetLayout,
    ) -> Self {
        let info = vk::DescriptorSetAllocateInfo::builder()
            .set_layouts(&[descriptor_set_layout.inner.handle])
            .descriptor_pool(descriptor_pool.inner.handle)
            .build();
        unsafe {
            let handles = device.handle().allocate_descriptor_sets(&info).unwrap();
            assert_eq!(handles.len(), 1);
            let handle = handles.first().unwrap().to_owned();
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
                        device.handle().handle(),
                        &vk::DebugUtilsObjectNameInfoEXT::builder()
                            .object_handle(handle.as_raw())
                            .object_type(vk::ObjectType::DESCRIPTOR_SET)
                            .object_name(CString::new(name).unwrap().as_ref())
                            .build(),
                    )
                    .unwrap();
            }
            Self {
                handle,
                device: device.clone(),
                descriptor_pool,
                descriptor_set_layout,
                resources: BTreeMap::new(),
            }
        }
    }

    fn new_with_descriptors(
        device: &Device,
        name: Option<&str>,
        descriptor_pool: DescriptorPool,
        descriptor_set_layout: DescriptorSetLayout,
        update_infos: BTreeMap<u32, DescriptorUpdate>,
    ) -> Self {
        let mut descriptor_set = Self::new(device, name, descriptor_pool, descriptor_set_layout);
        descriptor_set.update(update_infos);
        descriptor_set
    }

    pub(crate) fn update(&mut self, update_infos: BTreeMap<u32, DescriptorUpdate>) {
        let device = &self.device;
        let layout_bindings = &self.descriptor_set_layout.inner.vk_bindings;

        let mut buffer_infos = Vec::new();
        let mut image_infos = Vec::new();
        let mut tlas_handles = Vec::new();
        let mut write_acceleration_structure = None;
        let mut writes = Vec::new();

        for (binding, info) in &update_infos {
            let write_builder = vk::WriteDescriptorSet::builder()
                .dst_set(self.handle)
                .dst_binding(*binding)
                .descriptor_type(
                    layout_bindings
                        .get(binding)
                        .context(format!("layout does not contains binding {}", binding))
                        .unwrap()
                        .descriptor_type,
                );

            let write = match info {
                DescriptorUpdate::Buffer(buffers) => {
                    for (buffer, offset) in buffers {
                        buffer_infos.push(
                            vk::DescriptorBufferInfo::builder()
                                .buffer(buffer.inner.handle)
                                .offset(*offset)
                                .range(vk::WHOLE_SIZE)
                                .build(),
                        );
                    }
                    write_builder.buffer_info(&buffer_infos).build()
                }
                DescriptorUpdate::Image(image_views) => {
                    for image_view in image_views {
                        image_infos.push(
                            vk::DescriptorImageInfo::builder()
                                .image_layout(image_view.inner.image.layout())
                                .image_view(image_view.inner.handle)
                                .build(),
                        );
                    }
                    write_builder.image_info(&image_infos).build()
                }
                DescriptorUpdate::Sampler(sampler) => {
                    image_infos.push(
                        vk::DescriptorImageInfo::builder()
                            .sampler(sampler.inner.handle)
                            .build(),
                    );
                    write_builder.image_info(&image_infos).build()
                }
                DescriptorUpdate::AccelerationStructure(acceleration_structures) => {
                    for acc_struct in acceleration_structures {
                        tlas_handles.push(acc_struct.inner.handle);
                    }
                    write_acceleration_structure = Some(
                        vk::WriteDescriptorSetAccelerationStructureKHR::builder()
                            .acceleration_structures(tlas_handles.as_slice())
                            .build(),
                    );
                    write_builder
                        .push_next(write_acceleration_structure.as_mut().unwrap())
                        .build()
                }
            };
            writes.push(write);
        }

        unsafe {
            device
                .handle()
                .update_descriptor_sets(writes.as_slice(), &[]);
        }
    }
}

pub struct DescriptorSet {
    inner: Arc<DescriptorSetRef>,
}

impl DescriptorSet {
    pub fn new(
        device: &Device,
        name: Option<&str>,
        descriptor_pool: DescriptorPool,
        descriptor_set_layout: DescriptorSetLayout,
    ) -> Self {
        Self {
            inner: Arc::new(DescriptorSetRef::new(
                device,
                name,
                descriptor_pool,
                descriptor_set_layout,
            )),
        }
    }
}

impl std::fmt::Debug for DescriptorSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DescriptorSet")
            .field("handle", &self.inner.handle)
            .finish()
    }
}

pub enum DescriptorUpdate {
    Buffer(Vec<(Buffer, u64)>), // buffer and offset
    Image(Vec<ImageView>),
    Sampler(Sampler),
    AccelerationStructure(Vec<AccelerationStructure>),
}

impl Drop for DescriptorSetRef {
    fn drop(&mut self) {
        unsafe {
            self.descriptor_pool
                .inner
                .device
                .inner
                .handle
                .free_descriptor_sets(self.descriptor_pool.inner.handle, &[self.handle])
                .unwrap();
        }
    }
}

impl Device {
    pub fn allocate_descriptor_set(
        &self,
        name: Option<&str>,
        descriptor_pool: DescriptorPool,
        descriptor_set_layout: DescriptorSetLayout,
    ) -> DescriptorSet {
        DescriptorSet::new(self, name, descriptor_pool, descriptor_set_layout)
    }

    pub fn create_descriptor_set(
        &self,
        name: Option<&str>,
        descriptor_pool: DescriptorPool,
        descriptor_set_layout: DescriptorSetLayout,
        descriptor_infos: BTreeMap<u32, DescriptorUpdate>,
    ) -> DescriptorSet {
        let descriptor_set_ref = DescriptorSetRef::new_with_descriptors(
            self,
            name,
            descriptor_pool,
            descriptor_set_layout,
            descriptor_infos,
        );
        DescriptorSet {
            inner: Arc::new(descriptor_set_ref),
        }
    }
}
