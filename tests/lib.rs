#[test]
fn test_general() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .try_init()
        .ok();
    let entry = maligog::Entry::new().unwrap();
    let instance = entry.create_instance(&[], &[maligog::name::instance::Extension::ExtDebugUtils]);
    let pdevice = instance
        .enumerate_physical_device()
        .first()
        .unwrap()
        .to_owned();
    let (device, _queue_families) =
        pdevice.create_device(&[(pdevice.queue_families().first().unwrap(), &[1.0])]);
    let _buffer1 = device.create_buffer(
        None,
        123,
        maligog::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR,
        maligog::MemoryLocation::GpuOnly,
    );
    let _buffer2 = device.create_buffer(
        Some("hello"),
        1,
        maligog::BufferUsageFlags::TRANSFER_DST | maligog::BufferUsageFlags::UNIFORM_BUFFER,
        maligog::MemoryLocation::GpuToCpu,
    );
    let _buffer3 = device.create_buffer_init(
        Some("hastalavista"),
        &[1, 2, 3, 1, 51, 56, 4, 23, 1, 3],
        maligog::BufferUsageFlags::STORAGE_BUFFER,
        maligog::MemoryLocation::CpuToGpu,
    );
    let _buffer4 = device.create_buffer_init(
        Some("gpu only"),
        &[1, 2, 3, 1, 51, 56, 4, 23, 1, 3, 65, 2, 2, 2, 3],
        maligog::BufferUsageFlags::UNIFORM_BUFFER,
        maligog::MemoryLocation::GpuOnly,
    );
}
