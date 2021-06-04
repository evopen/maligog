use std::array::IntoIter;
use std::collections::BTreeMap;
use std::iter::FromIterator;
use std::time::Duration;

use maligog::vk;
use maligog::BufferView;

use maplit::btreemap;
use winit::platform::run_return::EventLoopExtRunReturn;
#[cfg(unix)]
use winit::platform::unix::EventLoopExtUnix;
#[cfg(windows)]
use winit::platform::windows::EventLoopExtWindows;

struct Engine {
    instance: maligog::Instance,
    device: maligog::Device,
    sampler: maligog::Sampler,
    buffer1: maligog::Buffer,
    buffer2: maligog::Buffer,
    image: maligog::Image,
    swapchain: maligog::Swapchain,
    image_view: maligog::ImageView,
    descriptor_set: maligog::DescriptorSet,
    blases: Vec<maligog::BottomAccelerationStructure>,
}

impl Engine {
    pub fn new(window: &dyn raw_window_handle::HasRawWindowHandle) -> Self {
        let entry = maligog::Entry::new().unwrap();
        let mut required_extensions = maligog::Surface::required_extensions();
        required_extensions.push(maligog::name::instance::Extension::ExtDebugUtils);
        let instance = entry.create_instance(&[], &&required_extensions);
        let pdevice = instance
            .enumerate_physical_device()
            .first()
            .unwrap()
            .to_owned();
        let device = pdevice.create_device();
        let buffer1 = device.create_buffer(
            None,
            123,
            maligog::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR,
            maligog::MemoryLocation::GpuOnly,
        );
        dbg!(&buffer1);
        let buffer2 = device.create_buffer(
            Some("hello"),
            1,
            maligog::BufferUsageFlags::TRANSFER_DST | maligog::BufferUsageFlags::STORAGE_BUFFER,
            maligog::MemoryLocation::GpuToCpu,
        );
        dbg!(&buffer2);

        let buffer3 = device.create_buffer_init(
            Some("hastalavista"),
            &[1, 2, 3, 1, 51, 56, 4, 23, 1, 3],
            maligog::BufferUsageFlags::STORAGE_BUFFER,
            maligog::MemoryLocation::CpuToGpu,
        );
        dbg!(&buffer3);

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
                descriptor_count: 1,
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
        let image = device.create_image(
            Some("this is an image"),
            vk::Format::B8G8R8A8_UNORM,
            200,
            200,
            maligog::ImageUsageFlags::STORAGE,
            maligog::MemoryLocation::GpuOnly,
        );
        let image1 = device.create_image_init(
            Some("initialized image"),
            vk::Format::R8_UINT,
            1,
            1,
            vk::ImageUsageFlags::empty(),
            maligog::MemoryLocation::GpuOnly,
            &[123],
        );
        image1.set_layout(
            maligog::ImageLayout::UNDEFINED,
            maligog::ImageLayout::GENERAL,
        );
        let image_view = image.create_view();
        let surface = instance.create_surface(window);
        let swapchain = device.create_swapchain(surface, maligog::PresentModeKHR::FIFO);
        let descriptor_set_layout = device.create_descriptor_set_layout(
            Some("temp descriptor set layout"),
            &[maligog::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: maligog::DescriptorType::StorageBuffer,
                stage_flags: maligog::ShaderStageFlags::ALL_GRAPHICS,
                descriptor_count: 2,
            }],
        );

        let descriptor_set = device.create_descriptor_set(
            Some("temp descriptor set"),
            &descriptor_pool,
            &descriptor_set_layout,
            btreemap! {
                0 => maligog::DescriptorUpdate::Buffer(vec![BufferView{buffer: buffer3.clone(), offset: 0},
                                                            BufferView{buffer: buffer2.clone(), offset: 0}]),
            },
        );

        let mut blases = Vec::new();
        let mut buffers = Vec::new();

        if let Ok(p) = std::env::var("GLTF_SAMPLE_PATH") {
            log::info!("testing acceleration structure");
            let (doc, gltf_buffers, _) =
                gltf::import(std::path::PathBuf::from(p).join("2.0/Box/glTF/Box.gltf")).unwrap();
            gltf_buffers.iter().for_each(|data| {
                buffers.push(device.create_buffer_init(
                    Some("gltf buffer"),
                    data.as_ref(),
                    maligog::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR
                        | maligog::BufferUsageFlags::STORAGE_BUFFER,
                    maligog::MemoryLocation::GpuOnly,
                ))
            });
            for mesh in doc.meshes() {
                let geometries: Vec<maligog::TriangleGeometry> = mesh
                    .primitives()
                    .map(|p| {
                        let index_accessor = p.indices().unwrap();
                        let (_, vertex_accessor) = p
                            .attributes()
                            .find(|(semantic, _)| semantic.eq(&gltf::Semantic::Positions))
                            .unwrap();
                        let index_buffer_view = maligog::IndexBufferView {
                            buffer_view: maligog::BufferView {
                                buffer: buffers[index_accessor.view().unwrap().buffer().index()]
                                    .clone(),
                                offset: (index_accessor.offset()
                                    + index_accessor.view().unwrap().offset())
                                    as u64,
                            },
                            index_type: match index_accessor.data_type() {
                                gltf::accessor::DataType::U16 => vk::IndexType::UINT16,
                                gltf::accessor::DataType::U32 => vk::IndexType::UINT32,
                                _ => {
                                    unimplemented!()
                                }
                            },
                            count: index_accessor.count() as u32,
                        };
                        let vertex_buffer_view = maligog::VertexBufferView {
                            buffer_view: maligog::BufferView {
                                buffer: buffers[vertex_accessor.view().unwrap().buffer().index()]
                                    .clone(),
                                offset: (vertex_accessor.offset()
                                    + vertex_accessor.view().unwrap().offset())
                                    as u64,
                            },
                            format: match vertex_accessor.data_type() {
                                gltf::accessor::DataType::U32 => maligog::Format::R32G32B32_UINT,
                                gltf::accessor::DataType::F32 => maligog::Format::R32G32B32_SFLOAT,
                                _ => {
                                    unimplemented!()
                                }
                            },
                            stride: match vertex_accessor.dimensions() {
                                gltf::accessor::Dimensions::Vec3 => {
                                    std::mem::size_of::<f32>() as u64 * 3
                                }
                                _ => {
                                    unimplemented!()
                                }
                            },
                            count: vertex_accessor.count() as u32,
                        };
                        maligog::TriangleGeometry::new(
                            &&index_buffer_view,
                            &&vertex_buffer_view,
                            None,
                        )
                    })
                    .collect();

                blases.push(
                    device.create_bottom_level_acceleration_structure(mesh.name(), &geometries),
                );
                log::warn!("here");
            }
            for blas in &blases {
                dbg!(&blas.name());
            }
        }

        Self {
            instance,
            device,
            sampler,
            buffer1,
            buffer2,
            image,
            swapchain,
            image_view,
            descriptor_set,
            blases,
        }
    }
}

#[test]
fn test_general() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .try_init()
        .ok();
    dotenv::dotenv().ok();

    let mut event_loop = winit::event_loop::EventLoop::<()>::new_any_thread();
    let win = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();
    let mut engine = Engine::new(&win);

    let mut frame_counter: u64 = 0;
    event_loop.run_return(|event, _, control_flow| {
        if frame_counter > 5 {
            *control_flow = winit::event_loop::ControlFlow::Exit;
        } else {
            *control_flow = winit::event_loop::ControlFlow::Poll;
        }
        frame_counter += 1;
        let index = engine.swapchain.acquire_next_image().unwrap();
        engine.swapchain.get_image(index).set_layout(
            maligog::ImageLayout::UNDEFINED,
            maligog::ImageLayout::PRESENT_SRC_KHR,
        );
        engine
            .swapchain
            .present(index, &[&engine.swapchain.image_available_semaphore()]);
    });
    engine.device.wait_idle();
}
