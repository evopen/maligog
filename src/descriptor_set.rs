use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ffi::CString;
use std::sync::{Arc, Mutex};

use ash::vk;
use ash::vk::Handle;

use anyhow::Context;

use crate::buffer::Buffer;
use crate::descriptor_pool::DescriptorPool;
use crate::sampler::Sampler;
use crate::BufferView;
use crate::Descriptor;
use crate::DescriptorSetLayout;
use crate::Device;
use crate::ImageView;
use crate::TopAccelerationStructure;

pub struct DescriptorSetRef {
    pub(crate) handle: vk::DescriptorSet,
    device: Device,
    descriptor_pool: DescriptorPool,
    descriptor_set_layout: DescriptorSetLayout,
    resources: BTreeMap<u32, Vec<Box<dyn Descriptor>>>,
}

impl DescriptorSetRef {
    fn new(
        device: &Device,
        name: Option<&str>,
        descriptor_pool: &DescriptorPool,
        descriptor_set_layout: &DescriptorSetLayout,
    ) -> Self {
        let handle = vec![descriptor_set_layout.inner.handle];

        let mut info = vk::DescriptorSetAllocateInfo::builder()
            .set_layouts(&handle)
            .descriptor_pool(descriptor_pool.inner.handle);

        unsafe {
            let handles = if let Some(count) = descriptor_set_layout.variable_descriptor_count() {
                let mut variable_descriptor_count_alloc_info =
                    vk::DescriptorSetVariableDescriptorCountAllocateInfo::builder()
                        .descriptor_counts(&[count])
                        .build();
                let info = info
                    .push_next(&mut variable_descriptor_count_alloc_info)
                    .build();
                device.handle().allocate_descriptor_sets(&info).unwrap()
            } else {
                device.handle().allocate_descriptor_sets(&info).unwrap()
            };

            assert_eq!(handles.len(), 1);
            let handle = handles.first().unwrap().to_owned();
            if let Some(name) = name {
                device.debug_set_object_name(name, handle.as_raw(), vk::ObjectType::DESCRIPTOR_SET);
            }
            Self {
                handle,
                device: device.clone(),
                descriptor_pool: descriptor_pool.clone(),
                descriptor_set_layout: descriptor_set_layout.clone(),
                resources: BTreeMap::new(),
            }
        }
    }

    fn new_with_descriptors(
        device: &Device,
        name: Option<&str>,
        descriptor_pool: &DescriptorPool,
        descriptor_set_layout: &DescriptorSetLayout,
        update_infos: BTreeMap<u32, DescriptorUpdate>,
    ) -> Self {
        let mut descriptor_set = Self::new(device, name, descriptor_pool, descriptor_set_layout);
        descriptor_set.update(update_infos);
        descriptor_set
    }

    pub(crate) fn update(&mut self, update_infos: BTreeMap<u32, DescriptorUpdate>) {
        let device = &self.device;
        let layout_bindings = &self.descriptor_set_layout.inner.vk_bindings;

        let reserve_size = update_infos.len();
        let mut buffer_infos = Vec::with_capacity(reserve_size);
        let mut image_infos = Vec::with_capacity(reserve_size);
        let mut tlas_handles = Vec::with_capacity(reserve_size);
        let mut write_acceleration_structure = None;
        let mut writes = Vec::with_capacity(reserve_size);

        for (binding, info) in &update_infos {
            let mut write_builder = vk::WriteDescriptorSet::builder()
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
                DescriptorUpdate::Buffer(buffer_views) => {
                    self.resources.insert(
                        *binding,
                        buffer_views
                            .iter()
                            .map(|v| Box::new(v.clone()) as Box<dyn Descriptor>)
                            .collect(),
                    );
                    let index = buffer_infos.len();
                    for buffer_view in buffer_views {
                        buffer_infos.push(
                            vk::DescriptorBufferInfo::builder()
                                .buffer(buffer_view.buffer.handle())
                                .offset(buffer_view.offset)
                                .range(vk::WHOLE_SIZE)
                                .build(),
                        );
                    }
                    write_builder.buffer_info(&buffer_infos[index..]).build()
                }
                DescriptorUpdate::Image(image_views) => {
                    self.resources.insert(
                        *binding,
                        image_views
                            .iter()
                            .map(|v| Box::new(v.clone()) as Box<dyn Descriptor>)
                            .collect(),
                    );
                    let index = image_infos.len();
                    for image_view in image_views {
                        image_infos.push(
                            vk::DescriptorImageInfo::builder()
                                .image_layout(image_view.inner.image.layout())
                                .image_view(image_view.inner.handle)
                                .build(),
                        );
                    }
                    write_builder.image_info(&image_infos[index..]).build()
                }
                DescriptorUpdate::Sampler(samplers) => {
                    self.resources.insert(
                        *binding,
                        samplers
                            .iter()
                            .map(|v| Box::new(v.clone()) as Box<dyn Descriptor>)
                            .collect(),
                    );
                    let index = image_infos.len();
                    for sampler in samplers {
                        image_infos.push(
                            vk::DescriptorImageInfo::builder()
                                .sampler(sampler.inner.handle)
                                .build(),
                        );
                    }

                    write_builder.image_info(&image_infos[index..]).build()
                }
                DescriptorUpdate::AccelerationStructure(acceleration_structures) => {
                    self.resources.insert(
                        *binding,
                        acceleration_structures
                            .iter()
                            .map(|v| Box::new(v.clone()) as Box<dyn Descriptor>)
                            .collect(),
                    );
                    let index = tlas_handles.len();
                    for acc_struct in acceleration_structures {
                        tlas_handles.push(acc_struct.inner.handle);
                    }
                    write_acceleration_structure = Some(
                        vk::WriteDescriptorSetAccelerationStructureKHR::builder()
                            .acceleration_structures(&tlas_handles[index..])
                            .build(),
                    );
                    let mut write = write_builder
                        .push_next(write_acceleration_structure.as_mut().unwrap())
                        .build();
                    write.descriptor_count = acceleration_structures.len() as u32;
                    write
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

#[derive(Clone)]
pub struct DescriptorSet {
    pub(crate) inner: Arc<Mutex<DescriptorSetRef>>,
}

impl DescriptorSet {
    pub fn new(
        device: &Device,
        name: Option<&str>,
        descriptor_pool: &DescriptorPool,
        descriptor_set_layout: &DescriptorSetLayout,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(DescriptorSetRef::new(
                device,
                name,
                descriptor_pool,
                descriptor_set_layout,
            ))),
        }
    }

    pub fn update(&self, update_infos: BTreeMap<u32, DescriptorUpdate>) {
        self.inner.lock().unwrap().update(update_infos);
    }
}

impl std::fmt::Debug for DescriptorSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DescriptorSet")
            .field("handle", &self.inner.lock().unwrap().handle)
            .finish()
    }
}

pub enum DescriptorUpdate {
    Buffer(Vec<BufferView>), // buffer and offset
    Image(Vec<ImageView>),
    Sampler(Vec<Sampler>),
    AccelerationStructure(Vec<TopAccelerationStructure>),
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
        descriptor_pool: &DescriptorPool,
        descriptor_set_layout: &DescriptorSetLayout,
    ) -> DescriptorSet {
        DescriptorSet::new(self, name, descriptor_pool, descriptor_set_layout)
    }

    pub fn create_descriptor_set(
        &self,
        name: Option<&str>,
        descriptor_pool: &DescriptorPool,
        descriptor_set_layout: &DescriptorSetLayout,
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
            inner: Arc::new(Mutex::new(descriptor_set_ref)),
        }
    }
}
