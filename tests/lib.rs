use maligog::vk;

#[cfg(unix)]
use winit::platform::unix::EventLoopExtUnix;
#[cfg(windows)]
use winit::platform::windows::EventLoopExtWindows;

#[test]
fn test_general() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .try_init()
        .ok();
    let entry = maligog::Entry::new().unwrap();
    let mut required_extensions = maligog::Surface::required_extensions();
    required_extensions.push(maligog::name::instance::Extension::ExtDebugUtils);
    let instance = entry.create_instance(&[], &&required_extensions);
    let pdevice = instance
        .enumerate_physical_device()
        .first()
        .unwrap()
        .to_owned();
    let device = pdevice.create_device(&[(pdevice.queue_families().first().unwrap(), &[1.0])]);
    let _buffer1 = device.create_buffer(
        None,
        123,
        maligog::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR,
        maligog::MemoryLocation::GpuOnly,
    );
    dbg!(&_buffer1);
    let _buffer2 = device.create_buffer(
        Some("hello"),
        1,
        maligog::BufferUsageFlags::TRANSFER_DST | maligog::BufferUsageFlags::UNIFORM_BUFFER,
        maligog::MemoryLocation::GpuToCpu,
    );
    dbg!(&_buffer2);

    let _buffer3 = device.create_buffer_init(
        Some("hastalavista"),
        &[1, 2, 3, 1, 51, 56, 4, 23, 1, 3],
        maligog::BufferUsageFlags::STORAGE_BUFFER,
        maligog::MemoryLocation::CpuToGpu,
    );
    dbg!(&_buffer3);

    let _buffer4 = device.create_buffer_init(
        Some("gpu only"),
        &[1, 2, 3, 1, 51, 56, 4, 23, 1, 3, 65, 2, 2, 2, 3],
        maligog::BufferUsageFlags::UNIFORM_BUFFER,
        maligog::MemoryLocation::GpuOnly,
    );
    dbg!(&_buffer4);

    let set_layout = device.create_descriptor_set_layout(
        Some("descriptor_set_layout"),
        &[maligog::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: maligog::DescriptorType::UniformBuffer,
            stage_flags: maligog::ShaderStageFlags::VERTEX,
        }],
    );

    let pipeline_layout =
        device.create_pipeline_layout(Some("pipeline layout"), &[&set_layout], &[]);
    let descriptor_pool = device.create_descriptor_pool(
        &[maligog::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(2)
            .build()],
        1,
    );
    let sampler = device.create_sampler(Some("sampler"));
    let event_loop = winit::event_loop::EventLoop::<()>::new_any_thread();
    let win = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();
    let surface = instance.create_surface(&win);
    let swapchain = device.create_swapchain(surface, maligog::PresentModeKHR::IMMEDIATE);
}
