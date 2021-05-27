use crate::queue::Queue;

#[derive(Debug, Clone)]
pub struct QueueFamilyProperties {
    pub(crate) index: u32,
    pub(crate) support_graphics: bool,
    pub(crate) support_compute: bool,
    pub(crate) support_transfer: bool,
    pub(crate) count: u32,
}

impl QueueFamilyProperties {
    pub fn support_graphics(&self) -> bool {
        self.support_graphics
    }

    pub fn support_compute(&self) -> bool {
        self.support_compute
    }
}

#[derive(Clone)]
pub struct QueueFamily {
    pub(crate) property: QueueFamilyProperties,
    pub queues: Vec<Queue>,
}

impl QueueFamily {
    pub fn support_graphics(&self) -> bool {
        self.property.support_graphics
    }

    pub fn support_compute(&self) -> bool {
        self.property.support_compute
    }
}
