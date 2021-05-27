use std::sync::Arc;

use ash::vk;

use crate::Device;

pub(crate) struct ShaderModuleRef {
    handle: vk::ShaderModule,
    device: Device,
}

pub struct ShaderModule {
    inner: Arc<ShaderModuleRef>,
}

#[repr(C, align(32))]
struct AlignedSpirv {
    pub code: Vec<u8>,
}

impl ShaderModule {
    pub(crate) fn new<P>(device: Device, spv: P) -> Self
    where
        P: AsRef<[u8]>,
    {
        let aligned = AlignedSpirv {
            code: spv.as_ref().to_vec(),
        };
        let info = vk::ShaderModuleCreateInfo::builder()
            .code(bytemuck::cast_slice(aligned.code.as_slice()))
            .build();
        unsafe {
            let handle = device
                .inner
                .handle
                .create_shader_module(&info, None)
                .unwrap();
            Self {
                inner: Arc::new(ShaderModuleRef { handle, device }),
            }
        }
    }
}

impl Device {
    pub fn create_shader_module<P>(&self, spv: P) -> ShaderModule
    where
        P: AsRef<[u8]>,
    {
        ShaderModule::new(self.clone(), spv)
    }
}

impl Drop for ShaderModuleRef {
    fn drop(&mut self) {
        unsafe {
            self.device
                .inner
                .handle
                .destroy_shader_module(self.handle, None);
        }
    }
}
