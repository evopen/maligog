use std::convert::TryInto;

use maligog::vk;
use maligog::BufferView;
use maligog::Device;
use maligog::ImageView;

use maplit::btreemap;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use winit::platform::run_return::EventLoopExtRunReturn;
#[cfg(unix)]
use winit::platform::unix::EventLoopExtUnix;
#[cfg(windows)]
use winit::platform::windows::EventLoopExtWindows;

struct Engine {
    instance: maligog::Instance,
    device: maligog::Device,
    image: maligog::Image,
    swapchain: maligog::Swapchain,
    descriptor_set: maligog::DescriptorSet,
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

        let image = device.create_image(
            Some("storage image"),
            maligog::Format::R32G32B32A32_SFLOAT,
            800,
            600,
            maligog::ImageUsageFlags::STORAGE,
            maligog::MemoryLocation::GpuOnly,
        );
        image.set_layout(
            maligog::ImageLayout::UNDEFINED,
            maligog::ImageLayout::GENERAL,
        );
        let scene = maligog_gltf::Scene::from_file(
            Some("the scene"),
            &device,
            "C:/Dev/rust/silly-cat-engine/cornell-box/models/CornellBox.glb",
        );
        let descriptor_pool = device.create_descriptor_pool(
            &[
                maligog::DescriptorPoolSize::builder()
                    .ty(vk::DescriptorType::STORAGE_IMAGE)
                    .descriptor_count(1)
                    .build(),
                maligog::DescriptorPoolSize::builder()
                    .ty(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
                    .descriptor_count(1)
                    .build(),
            ],
            1,
        );
        let descriptor_set_layout = device.create_descriptor_set_layout(
            Some("temp descriptor set layout"),
            &[
                maligog::DescriptorSetLayoutBinding {
                    binding: 0,
                    descriptor_type: maligog::DescriptorType::AccelerationStructure,
                    stage_flags: maligog::ShaderStageFlags::RAYGEN_KHR,
                    descriptor_count: 1,
                },
                maligog::DescriptorSetLayoutBinding {
                    binding: 1,
                    descriptor_type: maligog::DescriptorType::StorageImage,
                    stage_flags: maligog::ShaderStageFlags::RAYGEN_KHR,
                    descriptor_count: 1,
                },
            ],
        );

        let descriptor_set = device.create_descriptor_set(
            Some("temp descriptor set"),
            &descriptor_pool,
            &descriptor_set_layout,
            btreemap! {
                0 => maligog::DescriptorUpdate::AccelerationStructure(vec![scene.tlas().clone()]),
                1 => maligog::DescriptorUpdate::Image(vec![image.create_view()]),
            },
        );

        let pipeline_layout =
            device.create_pipeline_layout(Some("pipeline layout"), &[&descriptor_set_layout], &[]);

        // let rt_pipeline = device.create_ray_tracing_pipeline(Some("rt pipeline"), pipeline_layout);

        let surface = instance.create_surface(window);
        let swapchain = device.create_swapchain(surface, maligog::PresentModeKHR::FIFO);

        let module = device.create_shader_module(simple_rt_shader::SPIRV);
        let rg_stage =
            maligog::ShaderStage::new(&module, maligog::ShaderStageFlags::RAYGEN_KHR, "main");
        let hit_stage = maligog::ShaderStage::new(
            &module,
            maligog::ShaderStageFlags::CLOSEST_HIT_KHR,
            "closest_hit",
        );
        let miss_stage =
            maligog::ShaderStage::new(&module, maligog::ShaderStageFlags::MISS_KHR, "miss");
        let tri_hg = maligog::TrianglesHitGroup::new(&hit_stage, None);
        device.create_ray_tracing_pipeline(
            Some("a rt pipeline"),
            pipeline_layout,
            &rg_stage,
            &[miss_stage],
            &[&tri_hg],
            30,
        );

        device.wait_idle();

        Self {
            instance,
            device,
            image,
            swapchain,
            descriptor_set,
        }
    }
}

#[test]
fn test_rt_pipeline() {
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
