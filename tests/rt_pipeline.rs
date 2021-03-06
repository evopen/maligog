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
    pipeline: maligog::RayTracingPipeline,
    shader_binding_tables: maligog::ShaderBindingTables,
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
            maligog::ImageUsageFlags::STORAGE | maligog::ImageUsageFlags::TRANSFER_SRC,
            maligog::MemoryLocation::GpuOnly,
        );

        let scene = maligog_gltf::Scene::from_file(
            Some("the scene"),
            &device,
            std::env::var("CORNELL_BOX_GLTF").unwrap(),
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
                    variable_count: false,
                },
                maligog::DescriptorSetLayoutBinding {
                    binding: 1,
                    descriptor_type: maligog::DescriptorType::StorageImage,
                    stage_flags: maligog::ShaderStageFlags::RAYGEN_KHR,
                    descriptor_count: 1,
                    variable_count: true,
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

        let surface = instance.create_surface(window);
        let swapchain = device.create_swapchain(surface, maligog::PresentModeKHR::FIFO);

        let result =
            spirv_builder::SpirvBuilder::new("../simple-rt-shader", "spirv-unknown-vulkan1.2")
                .capability(spirv_builder::Capability::RayTracingKHR)
                .capability(spirv_builder::Capability::ImageQuery)
                .extension("SPV_KHR_ray_tracing")
                .build()
                .unwrap();
        let module_path = result.module.unwrap_single();
        let module = std::fs::read(module_path).unwrap();

        let module = device.create_shader_module(module);
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
        let pipeline = device.create_ray_tracing_pipeline(
            Some("a rt pipeline"),
            &pipeline_layout,
            &rg_stage,
            &[&miss_stage],
            &[&tri_hg],
            30,
        );
        let shader_binding_tables = pipeline.create_shader_binding_tables(&[0]);

        device.wait_idle();

        Self {
            instance,
            device,
            image,
            swapchain,
            descriptor_set,
            shader_binding_tables,
            pipeline,
        }
    }

    pub fn render(&self) {
        let index = self.swapchain.acquire_next_image().unwrap();
        let present_img = self.swapchain.get_image(index);

        let mut cmd_buf = self.device.create_command_buffer(
            Some("main cmdbuf"),
            self.device.graphics_queue_family_index(),
        );
        cmd_buf.encode(|rec| {
            rec.bind_ray_tracing_pipeline(&self.pipeline, |rec| {
                rec.bind_descriptor_sets(vec![&self.descriptor_set], 0);
                rec.trace_ray(
                    &self.shader_binding_tables.ray_gen_table(),
                    &self.shader_binding_tables.miss_table(),
                    &self.shader_binding_tables.hit_table(),
                    &self.shader_binding_tables.callable_table(),
                    self.image.width(),
                    self.image.height(),
                    1,
                );
            });

            rec.blit_image(
                &self.image,
                maligog::ImageLayout::GENERAL,
                &present_img,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[vk::ImageBlit::builder()
                    .src_subresource(
                        vk::ImageSubresourceLayers::builder()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .layer_count(1)
                            .base_array_layer(0)
                            .mip_level(0)
                            .build(),
                    )
                    .src_offsets([
                        vk::Offset3D { x: 0, y: 0, z: 0 },
                        vk::Offset3D {
                            x: self.image.width() as i32,
                            y: self.image.height() as i32,
                            z: 1,
                        },
                    ])
                    .dst_offsets([
                        vk::Offset3D { x: 0, y: 0, z: 0 },
                        vk::Offset3D {
                            x: present_img.width() as i32,
                            y: present_img.height() as i32,
                            z: 1,
                        },
                    ])
                    .dst_subresource(
                        vk::ImageSubresourceLayers::builder()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .layer_count(1)
                            .base_array_layer(0)
                            .mip_level(0)
                            .build(),
                    )
                    .build()],
                vk::Filter::NEAREST,
            );
        });
        self.device.graphics_queue().submit_blocking(&[cmd_buf]);

        self.swapchain
            .present(index, &[&self.swapchain.image_available_semaphore()]);
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

    event_loop.run_return(|event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Poll;
        match event {
            winit::event::Event::WindowEvent { window_id, event } => {
                match event {
                    winit::event::WindowEvent::CloseRequested => {
                        *control_flow = winit::event_loop::ControlFlow::Exit;
                    }
                    _ => {}
                }
            }
            winit::event::Event::MainEventsCleared => win.request_redraw(),
            winit::event::Event::RedrawRequested(_) => {
                engine.render();
            }
            _ => {}
        }
    });
}
