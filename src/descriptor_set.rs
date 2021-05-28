use std::sync::Arc;

use ash::vk;

use crate::buffer::Buffer;
use crate::descriptor::DescriptorSetLayout;
use crate::descriptor_pool::DescriptorPool;
use crate::sampler::Sampler;

pub struct DescriptorSetRef {
    handle: vk::DescriptorSet,
    descriptor_pool: DescriptorPool,
    descriptor_set_layout: DescriptorSetLayout,
    resources: RefCell<BTreeMap<u32, Arc<dyn Resource>>>,
}

pub struct DescriptorSet {
    inner: Arc<DescriptorSetRef>,
}

impl DescriptorSet {
    pub fn new(
        name: Option<&str>,
        descriptor_pool: Arc<DescriptorPool>,
        descriptor_set_layout: Arc<DescriptorSetLayout>,
    ) -> Self {
        let device = &descriptor_pool.device;
        let info = vk::DescriptorSetAllocateInfo::builder()
            .set_layouts(&[descriptor_set_layout.handle])
            .descriptor_pool(descriptor_pool.handle)
            .build();

        unsafe {
            let handles = device.handle.allocate_descriptor_sets(&info).unwrap();
            assert_eq!(handles.len(), 1);
            let handle = handles.first().unwrap().to_owned();
            if let Some(name) = name {
                device
                    .pdevice
                    .instance
                    .debug_utils_loader
                    .debug_utils_set_object_name(
                        device.handle.handle(),
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
                descriptor_pool,
                descriptor_set_layout,
                resources: RefCell::new(BTreeMap::new()),
            }
        }
    }

    pub fn update(&self, update_infos: &[DescriptorSetUpdateInfo]) {
        let device = self.inner.descriptor_pool.inner.device.clone();
        let bindings = self.descriptor_set_layout.vk_bindings.clone();

        let mut buffer_infos = Vec::new();
        let mut image_infos = Vec::new();
        let mut tlas_handles = Vec::new();
        let mut write_acceleration_structure = None;

        let descriptor_writes = update_infos
            .iter()
            .map(|info| {
                let write_builder = vk::WriteDescriptorSet::builder()
                    .dst_set(self.handle)
                    .dst_binding(info.binding)
                    .descriptor_type(
                        bindings
                            .iter()
                            .filter(|binding| binding.binding == info.binding)
                            .map(|binding| binding.descriptor_type)
                            .next()
                            .unwrap(),
                    );
                let mut write = match info.detail.borrow() {
                    DescriptorSetUpdateDetail::Buffer { buffer, offset } => {
                        self.resources
                            .try_borrow_mut()
                            .unwrap()
                            .insert(info.binding, buffer.clone());
                        buffer_infos.push(
                            vk::DescriptorBufferInfo::builder()
                                .buffer(buffer.handle)
                                .offset(*offset)
                                .range(vk::WHOLE_SIZE)
                                .build(),
                        );

                        write_builder
                            .buffer_info(&buffer_infos.as_slice()[buffer_infos.len() - 1..])
                            .build()
                    }
                    DescriptorSetUpdateDetail::Image(image_view) => {
                        self.resources
                            .try_borrow_mut()
                            .unwrap()
                            .insert(info.binding, image_view.clone());
                        image_infos.push(
                            vk::DescriptorImageInfo::builder()
                                .image_layout(image_view.image.layout())
                                .image_view(image_view.handle)
                                .build(),
                        );
                        write_builder
                            .image_info(&image_infos.as_slice()[image_infos.len() - 1..])
                            .build()
                    }
                    DescriptorSetUpdateDetail::Sampler(sampler) => {
                        self.resources
                            .try_borrow_mut()
                            .unwrap()
                            .insert(info.binding, sampler.clone());
                        image_infos.push(
                            vk::DescriptorImageInfo::builder()
                                .sampler(sampler.inner.handle)
                                .build(),
                        );
                        write_builder
                            .image_info(&image_infos.as_slice()[image_infos.len() - 1..])
                            .build()
                    }
                    DescriptorSetUpdateDetail::AccelerationStructure(tlas) => {
                        self.resources
                            .try_borrow_mut()
                            .unwrap()
                            .insert(info.binding, tlas.clone());
                        tlas_handles.push(tlas.handle);
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

                write.descriptor_count = 1;
                write
            })
            .collect::<Vec<_>>();
        assert_eq!(descriptor_writes.len(), update_infos.len());
        unsafe {
            device
                .handle
                .update_descriptor_sets(descriptor_writes.as_slice(), &[]);
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

pub enum DescriptorSetUpdateDetail {
    Buffer { buffer: Buffer, offset: u64 },
    Image(Arc<ImageView>),
    Sampler(Sampler),
    AccelerationStructure(Arc<AccelerationStructure>),
}

pub struct DescriptorSetUpdateInfo {
    pub binding: u32,
    pub detail: DescriptorSetUpdateDetail,
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
