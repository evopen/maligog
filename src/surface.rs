use std::sync::Arc;

use ash::vk;

use crate::instance::Instance;

pub struct SurfaceRef {
    pub(crate) handle: vk::SurfaceKHR,
    instance: Instance,
    required_extensions: Vec<String>,
}

pub struct Surface {
    pub(crate) inner: Arc<SurfaceRef>,
}

impl Surface {
    pub fn new(instance: Instance, window: &dyn raw_window_handle::HasRawWindowHandle) -> Self {
        let handle = unsafe {
            ash_window::create_surface(
                &instance.inner.entry.handle,
                &instance.inner.handle,
                window,
                None,
            )
            .unwrap()
        };

        let required_extensions = ash_window::enumerate_required_extensions(window)
            .unwrap()
            .iter()
            .map(|s| s.to_str().unwrap().to_string())
            .collect::<Vec<_>>();

        Self {
            inner: Arc::new(SurfaceRef {
                handle,
                instance,
                required_extensions,
            }),
        }
    }
}

impl Drop for SurfaceRef {
    fn drop(&mut self) {
        unsafe {
            self.instance
                .inner
                .surface_loader
                .as_ref()
                .unwrap()
                .destroy_surface(self.handle, None);
        }
    }
}
