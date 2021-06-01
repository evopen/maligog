use std::sync::Arc;

use ash::vk;

use crate::Device;

pub(crate) struct DescriptorPoolRef {
    pub(crate) handle: vk::DescriptorPool,
    pub(crate) device: Device,
}

#[derive(Clone)]
pub struct DescriptorPool {
    pub(crate) inner: Arc<DescriptorPoolRef>,
}

impl DescriptorPool {
    pub fn new(
        device: &Device,
        descriptor_pool_size: &[vk::DescriptorPoolSize],
        max_sets: u32,
    ) -> Self {
        unsafe {
            let info = vk::DescriptorPoolCreateInfo::builder()
                .pool_sizes(descriptor_pool_size)
                .max_sets(max_sets)
                .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET)
                .build();
            let handle = device
                .inner
                .handle
                .create_descriptor_pool(&info, None)
                .unwrap();
            Self {
                inner: Arc::new(DescriptorPoolRef {
                    handle,
                    device: device.clone(),
                }),
            }
        }
    }
}

impl Drop for DescriptorPoolRef {
    fn drop(&mut self) {
        unsafe {
            self.device
                .inner
                .handle
                .destroy_descriptor_pool(self.handle, None);
        }
    }
}

impl Device {
    pub fn create_descriptor_pool(
        &self,
        descriptor_pool_size: &[vk::DescriptorPoolSize],
        max_sets: u32,
    ) -> DescriptorPool {
        DescriptorPool::new(&self, descriptor_pool_size, max_sets)
    }
}
