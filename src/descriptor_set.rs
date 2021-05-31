use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ffi::CString;
use std::sync::Arc;

use ash::vk;
use ash::vk::Handle;

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
    ) -> Self {
        let descriptor_set = Self::new(device, name, descriptor_pool, descriptor_set_layout);
        Self {}
    }

    pub fn update(&mut self, update_infos: &[DescriptorSetUpdateInfo]) {
        let device = self.device;
        let bindings = self.descriptor_set_layout.inner.vk_bindings;

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
                    .descriptor_type(bindings.get(&info.binding).unwrap().descriptor_type);
                let mut write = match info.details {
                    DescriptorSetUpdateDetail::Buffer(buffers) => {
                        self.resources.insert(info.binding, buffer.clone());
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
                        self.resources.insert(info.binding, image_view.clone());
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
                        self.resources.insert(info.binding, sampler.clone());
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
                        self.resources.insert(info.binding, tlas.clone());
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
                .handle()
                .update_descriptor_sets(descriptor_writes.as_slice(), &[]);
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

    // pub fn update(&self, update_infos: &[DescriptorSetUpdateInfo]) {
    //     let device = self.inner.descriptor_pool.inner.device.clone();
    //     let bindings = self.inner.descriptor_set_layout.vk_bindings.clone();

    //     let mut buffer_infos = Vec::new();
    //     let mut image_infos = Vec::new();
    //     let mut tlas_handles = Vec::new();
    //     let mut write_acceleration_structure = None;

    //     let descriptor_writes = update_infos
    //         .iter()
    //         .map(|info| {
    //             let write_builder = vk::WriteDescriptorSet::builder()
    //                 .dst_set(self.handle)
    //                 .dst_binding(info.binding)
    //                 .descriptor_type(
    //                     bindings
    //                         .iter()
    //                         .filter(|binding| binding.binding == info.binding)
    //                         .map(|binding| binding.descriptor_type)
    //                         .next()
    //                         .unwrap(),
    //                 );
    //             let mut write = match info.detail.borrow() {
    //                 DescriptorSetUpdateDetail::Buffer { buffer, offset } => {
    //                     self.resources
    //                         .try_borrow_mut()
    //                         .unwrap()
    //                         .insert(info.binding, buffer.clone());
    //                     buffer_infos.push(
    //                         vk::DescriptorBufferInfo::builder()
    //                             .buffer(buffer.handle)
    //                             .offset(*offset)
    //                             .range(vk::WHOLE_SIZE)
    //                             .build(),
    //                     );

    //                     write_builder
    //                         .buffer_info(&buffer_infos.as_slice()[buffer_infos.len() - 1..])
    //                         .build()
    //                 }
    //                 DescriptorSetUpdateDetail::Image(image_view) => {
    //                     self.resources
    //                         .try_borrow_mut()
    //                         .unwrap()
    //                         .insert(info.binding, image_view.clone());
    //                     image_infos.push(
    //                         vk::DescriptorImageInfo::builder()
    //                             .image_layout(image_view.image.layout())
    //                             .image_view(image_view.handle)
    //                             .build(),
    //                     );
    //                     write_builder
    //                         .image_info(&image_infos.as_slice()[image_infos.len() - 1..])
    //                         .build()
    //                 }
    //                 DescriptorSetUpdateDetail::Sampler(sampler) => {
    //                     self.resources
    //                         .try_borrow_mut()
    //                         .unwrap()
    //                         .insert(info.binding, sampler.clone());
    //                     image_infos.push(
    //                         vk::DescriptorImageInfo::builder()
    //                             .sampler(sampler.inner.handle)
    //                             .build(),
    //                     );
    //                     write_builder
    //                         .image_info(&image_infos.as_slice()[image_infos.len() - 1..])
    //                         .build()
    //                 }
    //                 DescriptorSetUpdateDetail::AccelerationStructure(tlas) => {
    //                     self.resources
    //                         .try_borrow_mut()
    //                         .unwrap()
    //                         .insert(info.binding, tlas.clone());
    //                     tlas_handles.push(tlas.handle);
    //                     write_acceleration_structure = Some(
    //                         vk::WriteDescriptorSetAccelerationStructureKHR::builder()
    //                             .acceleration_structures(tlas_handles.as_slice())
    //                             .build(),
    //                     );
    //                     write_builder
    //                         .push_next(write_acceleration_structure.as_mut().unwrap())
    //                         .build()
    //                 }
    //             };

    //             write.descriptor_count = 1;
    //             write
    //         })
    //         .collect::<Vec<_>>();
    //     assert_eq!(descriptor_writes.len(), update_infos.len());
    //     unsafe {
    //         device
    //             .handle
    //             .update_descriptor_sets(descriptor_writes.as_slice(), &[]);
    //     }
    // }
}

impl std::fmt::Debug for DescriptorSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DescriptorSet")
            .field("handle", &self.inner.handle)
            .finish()
    }
}

pub enum DescriptorSetUpdateDetail {
    Buffer(Vec<(Buffer, u64)>), // buffer and offset
    Image(Vec<ImageView>),
    Sampler(Sampler),
    AccelerationStructure(Vec<AccelerationStructure>),
}

pub struct DescriptorSetUpdateInfo {
    pub binding: u32,
    pub details: DescriptorSetUpdateDetail,
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
    ) -> DescriptorSet {
        DescriptorSet::new(self, name, descriptor_pool, descriptor_set_layout)
    }
}
