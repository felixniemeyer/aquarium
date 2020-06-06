use std::time; 
use std::mem::MaybeUninit;

use vulkano::{
    instance::{
        Instance,
        PhysicalDevice,
    },

    pipeline::{
        GraphicsPipeline, 
        viewport::Viewport, 
        ComputePipeline,
		vertex::{
			SingleBufferDefinition
		}
    },

    device::{
        Device,
        Features,
    },

    framebuffer::{
        Framebuffer, 
        FramebufferAbstract, 
        Subpass, 
        RenderPassAbstract
    },

    image::{
        SwapchainImage, 
        Dimensions, 
        ImmutableImage, 
        StorageImage,
        ImageUsage,
    },

    sampler::{
        Sampler, 
        SamplerAddressMode, 
        Filter, 
        MipmapMode
    },

    buffer::{
        BufferUsage,
        CpuAccessibleBuffer,
    },

    command_buffer::{
        AutoCommandBufferBuilder, 
        DynamicState
    },

    descriptor::{
        descriptor_set::PersistentDescriptorSet,
        PipelineLayoutAbstract
    },

    format::Format, 

    swapchain,
    swapchain::{
        ColorSpace,
        FullscreenExclusive,
        AcquireError, 
        Swapchain, 
        SurfaceTransform, 
        PresentMode, 
        SwapchainCreationError
    },

    sync, 
    sync::{
        GpuFuture, 
        FlushError
    },
};

use std::sync::Arc;

use rand::{
    thread_rng, 
    Rng
};

use image::{
    GenericImageView, 
};

use vulkano_win::VkSurfaceBuild; 
use winit::{
    event_loop::{
        ControlFlow, 
        EventLoop, 
    },
    window::{
        Window, 
        WindowBuilder, 
    },
    event::{
        Event, 
        WindowEvent,
        KeyboardInput, 
        VirtualKeyCode,
        ElementState,
    }
};

//#[derive(Copy, Clone, Debug)]
//struct Particle {
//    pos: [f32; 2],
//    speed: [f32; 2],
//    tail: [f32; 2],
//    prev_pos: [f32; 2],
//    prev_tail: [f32; 2],
//}

// Vertex types
#[derive(Default, Debug, Clone, Copy)]
struct Vertex { 
    position: [f32; 4], // note: to be able to bind it as a buffer for compute shader access, use 4 or 2 array sizes - never 3
    tail: [f32; 4],
}
vulkano::impl_vertex!(Vertex, position, tail); 

#[derive(Default, Debug, Clone, Copy)]
struct VertexTwoDTex {
	position: [f32; 2],
	uv: [f32; 2],
}
vulkano::impl_vertex!(VertexTwoDTex, position, uv); 

fn main() {
    const FLUX_RES: u32 = 32; 
	const PARTICLE_COUNT: u32 = 512; //TODO: for some reason only half of it gets updated by particle_cp 

    let img = match image::open("./fish/skin-0001.png") {
        Ok(image) => image, 
        Err(err) => panic!("{:?}", err)
    };

    let instance = {
        let inst_exts = vulkano_win::required_extensions(); 
        Instance::new(None, &inst_exts, None).expect("failed to create instance")
    };

    for p in PhysicalDevice::enumerate(&instance) {
        print!("{}", p.name());
        println!(", driver version: {}", p.driver_version());
    }

    let physical = PhysicalDevice::enumerate(&instance)
        .next()
        .expect("no device available");

    let queue_family = physical
        .queue_families()
        .find(|&q| q.supports_graphics() && q.supports_compute())
        .expect("couldn't find a queue that supports compute and graphics");

    let (device, mut queues) = {
        let device_extensions = vulkano::device::DeviceExtensions {
            khr_swapchain: true, 
            khr_storage_buffer_storage_class: true, 
            .. vulkano::device::DeviceExtensions::none()
        };

        // disadvantage of specifying to much (all supported) features? physical.supported_features()
        let device_features = Features {
            geometry_shader: true, 
            .. Features::none()
        };

        Device::new(
            physical,
            &device_features,
            &device_extensions,
            [(queue_family, 0.5)].iter().cloned(),
        )
        .expect("failed to create device")
    };

    let queue = queues.next().unwrap();

    // let particles = init_particles_buffer();
    // let particles_buffer =
    //     CpuAccessibleBuffer::from_data(device.clone(), BufferUsage::all(), particles)
    //         .expect("failed to create buffer");


    // let shader = cs::Shader::load(device.clone()).expect("failed to create shader module");
    // let compute_pipeline = Arc::new(
    //     ComputePipeline::new(device.clone(), &shader.main_entry_point(), &())
    //         .expect("failed to create compute pipeline"),
    // );

    // let set = Arc::new(
    //     PersistentDescriptorSet::start(compute_pipeline.clone(), 0)
    //         .add_buffer(particles_buffer.clone())
    //         .unwrap()
    //         .build()
    //         .unwrap(),
    // );

    // let command_buffer = AutoCommandBufferBuilder::new(device.clone(), queue.family())
    //     .unwrap()
    //     .dispatch([PARTICLE_COUNT as u32 / 32, 1, 1], compute_pipeline.clone(), set.clone(), ())
    //     .unwrap()
    //     .build()
    //     .unwrap();

    let event_loop = EventLoop::new(); 
    let surface = WindowBuilder::new().build_vk_surface(&event_loop, instance.clone()).unwrap();

    let (mut swapchain, images) = {
        let caps = surface.capabilities(physical) 
            .expect("failed to get surface capabilities"); 
        let usage = caps.supported_usage_flags; 
        let alpha = caps.supported_composite_alpha.iter().next().unwrap(); 
        let format = caps.supported_formats[0].0;
        println!("Supported surface formats:");
        caps.supported_formats.iter().for_each(|sth| println!("{:?}", sth)); 

        let dim: [u32; 2] = surface.window().inner_size().into();

        Swapchain::new(
            device.clone(), 
            surface.clone(), 
            caps.min_image_count, format, dim, 1, usage, &queue, 
            SurfaceTransform::Identity, alpha, PresentMode::Fifo, FullscreenExclusive::Default, false, ColorSpace::SrgbNonLinear)
        .expect("failed to create swapchain")
    };

    let render_pass = Arc::new(vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: swapchain.format(),
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
    ).unwrap());

	/////////////
	// shaders //
	/////////////
    #[allow(dead_code)] // Used to force recompilation of shader change
    const SFISH0: &str = include_str!("./shader/fish.vs.glsl");
    mod fish_vs { 
        vulkano_shaders::shader!{
            ty: "vertex", 
            path: "./src/shader/fish.vs.glsl"
        }
    }

    #[allow(dead_code)] // Used to force recompilation of shader change
    const S_FISH1: &str = include_str!("./shader/fish.fs.glsl");
    mod fish_fs { 
        vulkano_shaders::shader!{
            ty: "fragment", 
            path: "./src/shader/fish.fs.glsl"
        }
    }

    #[allow(dead_code)] // Used to force recompilation of shader change
    const S_FISH2: &str = include_str!("./shader/fish.gs.glsl");
    mod fish_gs { 
        vulkano_shaders::shader!{
            ty: "geometry", 
            path: "./src/shader/fish.gs.glsl"
        }
    }

    #[allow(dead_code)] // Used to force recompilation of shader change
    const C_FLUX: &str = include_str!("./shader/flux.cp.glsl");
    mod flux_cp { 
        vulkano_shaders::shader!{
            ty: "compute", 
            path: "./src/shader/flux.cp.glsl"
        }
    }

    #[allow(dead_code)] // Used to force recompilation of shader change
    const C_PARTICLE: &str = include_str!("./shader/particle.cp.glsl");
	mod particle_cp {
		vulkano_shaders::shader! {
			ty: "compute",
			path: "./src/shader/particle.cp.glsl"
		}
	}

    #[allow(dead_code)] // Used to force recompilation of shader change
    const S_G_2D: &str = include_str!("./shader/general_2d.vs.glsl");
    mod general_2d_vs { 
        vulkano_shaders::shader!{
            ty: "vertex", 
            path: "./src/shader/general_2d.vs.glsl"
        }
    }

    #[allow(dead_code)] // Used to force recompilation of shader change
    const S_DDN: &str = include_str!("./shader/debug_draw_flux.fs.glsl");
    mod debug_draw_flux_fs { 
        vulkano_shaders::shader!{
            ty: "fragment", 
            path: "./src/shader/debug_draw_flux.fs.glsl"
        }
    }

    let fish_vs = fish_vs::Shader::load(device.clone()).unwrap(); 
    let fish_gs = fish_gs::Shader::load(device.clone()).unwrap(); 
    let fish_fs = fish_fs::Shader::load(device.clone()).unwrap(); 

    let flux_cp = flux_cp::Shader::load(device.clone()).unwrap(); 
	let particle_cp = particle_cp::Shader::load(device.clone()).unwrap(); 

    let general_2d_vs = general_2d_vs::Shader::load(device.clone()).unwrap(); 
    let debug_draw_flux_fs = debug_draw_flux_fs::Shader::load(device.clone()).unwrap(); 


    //////////////
    // flux img //
    //////////////
	
    let mut image_usage = ImageUsage::none();
    image_usage.storage = true; 
	image_usage.sampled = true;
    let flux = match StorageImage::with_usage(
        device.clone(),
        Dimensions::Dim3d { 
            width: FLUX_RES,    
            height: FLUX_RES, 
            depth: FLUX_RES 
        },
        Format::R8G8B8A8Snorm,
		image_usage,
        Some(queue.family())
    ) {
        Ok(i) => i,
        Err(err) => panic!("{:?}", err)
    };

	/////////////// 
	// pipelines // 
	///////////////

    let fish_pipeline = Arc::new(GraphicsPipeline::start()
        .vertex_input(SingleBufferDefinition::<Vertex>::new())
        .vertex_shader(fish_vs.main_entry_point(), ())
        .geometry_shader(fish_gs.main_entry_point(), ())
        .point_list()
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fish_fs.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap()
    ); 

    let fish_skin_sampler = Sampler::new(device.clone(), Filter::Linear, Filter::Linear, 
        MipmapMode::Nearest, SamplerAddressMode::Repeat, SamplerAddressMode::Repeat, 
        SamplerAddressMode::Repeat, 0.0, 1.0, 0.0, 0.0).unwrap(); 


    let debug_draw_flux_pipeline = Arc::new(GraphicsPipeline::start()
        .vertex_input(SingleBufferDefinition::<VertexTwoDTex>::new())
        .vertex_shader(general_2d_vs.main_entry_point(), ())
        .triangle_strip()
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(debug_draw_flux_fs.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap()
    );


    let flux_compute_pipeline = Arc::new(
        ComputePipeline::new(device.clone(), &flux_cp.main_entry_point(), &()).unwrap()
    );

	let particle_compute_pipeline = Arc::new(
		ComputePipeline::new(device.clone(), &particle_cp.main_entry_point(), &()).unwrap()
	); 

    ///////////////
    // fish draw //
    ///////////////
    let mut vertex_data: [Vertex; PARTICLE_COUNT as usize] = {
		unsafe { MaybeUninit::uninit().assume_init() }
	};
    let mut rng = thread_rng();
    for i in 0..vertex_data.len() {
        vertex_data[i].position = [
            rng.gen_range(-1.0,1.0),
            rng.gen_range(-1.0,1.0),
            rng.gen_range(-1.0,1.0),
            1.0,
        ];
        vertex_data[i].tail = [ 1.0, 0.0, 0.0, 1.0 ];
    }
	// TODO: use DeviceLocalBuffer
    let fish_vertex_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(), 
        BufferUsage::all(), 
        false, 
        vertex_data.iter().cloned()
    ).unwrap();

    let img_dim = img.dimensions();
    let (autumn_texture, autumn_texture_future) = match ImmutableImage::from_iter(
        img.as_rgba8().unwrap().pixels().map(|rgba| {
            let bytes : [u8; 4] = [rgba[0], rgba[1], rgba[2], rgba[3]]; 
            bytes
        }),
        Dimensions::Dim2d { width: img_dim.0, height: img_dim.1 },
        Format::R8G8B8A8Unorm, 
        queue.clone()
    ) {
        Ok(i) => i, 
        Err(err) => panic!("{:?}", err)
    };

	let fish_pipeline_layout = fish_pipeline.layout().descriptor_set_layout(0).unwrap(); 
    let fish_desc_set = Arc::new(PersistentDescriptorSet::start(fish_pipeline_layout.clone())
        .add_sampled_image(autumn_texture.clone(), fish_skin_sampler.clone()).unwrap()
        .build().unwrap()
    );

	//////////////
	// flux gen //
	//////////////
	let flux_compute_pipeline_layout = flux_compute_pipeline.layout().descriptor_set_layout(0).unwrap(); 
    let flux_compute_descr_set = Arc::new(
        PersistentDescriptorSet::start(flux_compute_pipeline_layout.clone())
            .add_image(flux.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

	/////////////////////
	// particle update //
	/////////////////////
	#[derive(Default, Debug, Clone, Copy)]
	struct Particle {
		position: [f32; 3],
        velocity: [f32; 3],
        drift: [f32;3],
	}
	let mut particle_data: [Particle; PARTICLE_COUNT as usize] = {
		unsafe { MaybeUninit::uninit().assume_init() }
	};
	// TODO: use DeviceLocalBuffer
	for i in 0..particle_data.len() {
		particle_data[i].position = [
            vertex_data[i].position[0],
            vertex_data[i].position[1],
            vertex_data[i].position[2],
        ];
        particle_data[i].velocity = [0.0, 0.0, 0.0];
        particle_data[i].drift = [
            -vertex_data[i].position[0],
            -vertex_data[i].position[1],
            -vertex_data[i].position[2],
        ];
	}
	let fish_particle_buffer = CpuAccessibleBuffer::from_iter(
		device.clone(),
		BufferUsage::all(), 
		false,
		particle_data.iter().cloned()
	).unwrap();

	let flux_sampler = Sampler::new(
		device.clone(), 
		Filter::Linear, Filter::Linear, 
		MipmapMode::Nearest, 
		SamplerAddressMode::ClampToEdge,
		SamplerAddressMode::ClampToEdge,
		SamplerAddressMode::ClampToEdge,
		0.0, 1.0, 0.0, 0.0
	).unwrap();

	let particle_compute_pipeline_layout = particle_compute_pipeline.layout().descriptor_set_layout(0).unwrap(); 
	let particle_compute_descr_set = Arc::new(
		PersistentDescriptorSet::start(particle_compute_pipeline_layout.clone())
			.add_sampled_image(flux.clone(), flux_sampler.clone()).unwrap()
    		.add_buffer(fish_particle_buffer.clone()).unwrap()
    		.add_buffer(fish_vertex_buffer.clone()).unwrap()
			.build().unwrap()
	);

    /////////////////////
    // debug draw flux //
    /////////////////////
	
    let debug_draw_flux_vertex_buffer = {
		let x = -0.9;
		let y = -0.9; 
		let s = 0.3;
		CpuAccessibleBuffer::from_iter(
			device.clone(), 
			BufferUsage::all(), 
			false, 
			[
				VertexTwoDTex { position: [x, y], uv: [0.0, 0.0] },
				VertexTwoDTex { position: [x, y+s], uv: [0.0, 1.0] },
				VertexTwoDTex { position: [x+s, y], uv: [1.0, 0.0] },
				VertexTwoDTex { position: [x+s, y+s], uv: [1.0, 1.0] },
			].iter().cloned()
		).unwrap()
	};

	let layout = debug_draw_flux_pipeline.layout().descriptor_set_layout(0).unwrap().clone();
    let debug_draw_flux_desc_set = Arc::new(PersistentDescriptorSet::start(layout)
        .add_sampled_image(flux.clone(), flux_sampler.clone()).unwrap()
        .build().unwrap()
    );


	//////////////////////
	//////////////////////
	//////////////////////
    let mut dynamic_state = DynamicState { 
        line_width: None, 
        viewports: None, 
        scissors: None, 
        compare_mask: None, 
        write_mask: None, 
        reference: None 
    }; 
    
    let mut framebuffers = window_size_dependent_setup(&images, render_pass.clone(), &mut dynamic_state); 

    let mut recreate_swapchain = false; 

 	let mut previous_frame_end = Some(Box::new(sync::now(device.clone()).join(autumn_texture_future)) as Box<dyn GpuFuture>); 

    let t0 = time::SystemTime::now(); 
    let mut now = t0; 
    let mut then = t0;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent { event: WindowEvent::Resized(_), .. } => recreate_swapchain = true,
            Event::WindowEvent { 
                event: WindowEvent::KeyboardInput {  
                    input: KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Q),
                        state: ElementState::Pressed, ..
                    }, ..
                }, ..

            } => *control_flow = ControlFlow::Exit, 
            Event::RedrawEventsCleared => {
                previous_frame_end.as_mut().unwrap().cleanup_finished(); 

                if recreate_swapchain {
                    let dim: [u32; 2] = surface.window().inner_size().into();
                    let (new_swapchain, new_images) = match swapchain.recreate_with_dimensions(dim) {
                        Ok(r) => r, 
                        Err(SwapchainCreationError::UnsupportedDimensions) => return, 
                        Err(err) => panic!("failed to recreate swapchain {:?}", err)
                    }; 

                    swapchain = new_swapchain; 
                    framebuffers = window_size_dependent_setup(&new_images, render_pass.clone(), &mut dynamic_state); 
                    recreate_swapchain = false; 
                }

                let (image_num, suboptimal, acquire_future) = match swapchain::acquire_next_image(swapchain.clone(), None){
                    Ok(r) => r, 
                    Err(AcquireError::OutOfDate) => {
                        recreate_swapchain = true; 
                        return; 
                    }, 
                    Err(err) => panic!("{:?}", err)
                }; 

                if suboptimal {
                    recreate_swapchain = true; 
                }

                then = now; 
                now = time::SystemTime::now();

                let time = (now.duration_since(t0).unwrap().as_millis() % (1000 * 60 * 60 * 24 * 365)) as f32 * 0.001;
                let dtime = now.duration_since(then).unwrap().as_millis() as f32 * 0.001;

                let flux_compute_push_constants = flux_cp::ty::PushConstantData {
                    time, 
                    dtime
                };

                let particle_compute_push_constants = particle_cp::ty::PushConstantData {
                    time, 
                    dtime,
                    friction_95: 0.8f32.powf(dtime)
                };

                let fish_push_constants = fish_gs::ty::PushConstantData {
                    time,
                    dtime
                };

                let clear_values = vec!([0.03, 0.13, 0.3, 1.0].into()); 
                let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(
                    device.clone(), 
                    queue.family()
                )
                    .unwrap()
					.dispatch(
						[PARTICLE_COUNT, 1, 1], 
						particle_compute_pipeline.clone(), 
						particle_compute_descr_set.clone(), 
						particle_compute_push_constants
					)
                    .unwrap()
					.dispatch(
						[FLUX_RES, FLUX_RES, FLUX_RES], 
						flux_compute_pipeline.clone(), 
						flux_compute_descr_set.clone(), 
						flux_compute_push_constants
					)
                    .unwrap()
                    .begin_render_pass(framebuffers[image_num].clone(), false, clear_values)
					.unwrap()
                    .draw(
                        fish_pipeline.clone(), 
                        &dynamic_state, 
                        fish_vertex_buffer.clone(), 
                        fish_desc_set.clone(), 
                        fish_push_constants)
                    .unwrap()
                    .draw(
                        debug_draw_flux_pipeline.clone(), 
                        &dynamic_state, 
                        debug_draw_flux_vertex_buffer.clone(), 
                        debug_draw_flux_desc_set.clone(), 
                        ()
                    )
                    .unwrap()
                    .end_render_pass()
                    .unwrap()
                    .build()
                    .unwrap();

                let future = previous_frame_end.take().unwrap()
                    .join(acquire_future)
                    .then_execute(queue.clone(), command_buffer).unwrap()
                    .then_swapchain_present(queue.clone(), swapchain.clone(), image_num) 
                    .then_signal_fence_and_flush(); 

                match future {
                    Ok(future) => {
                        future.wait(None).unwrap(); 
                        previous_frame_end = Some(Box::new(future) as Box<_>);
                    }
                    Err(FlushError::OutOfDate) => {
                        recreate_swapchain = true; 
                        previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<_>);
                    }
                    Err(e) => {
                        println!("{:?}", e);
                        previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<_>); 
                    }
                }
            },
            _ => ()
        }
    });
}

// fn init_particles_buffer() -> [Particle; PARTICLE_COUNT] {
//     let mut rng = thread_rng();
//     let mut particles = [Particle {
//         pos: [0.0, 0.0],
//         tail: [0.0, 0.0],
//         speed: [0.0, 0.0],
//         prev_pos: [0.0, 0.0],
//         prev_tail: [0.0, 0.0],
//     }; PARTICLE_COUNT];
//     for i in 0..particles.len() {
//         particles[i].pos = [rng.gen_range(-1.0, 1.0), rng.gen_range(-1.0, 1.0)];
//         particles[i].tail = particles[i].pos.clone();
//         particles[i].speed = [rng.gen_range(-0.1, 0.1), rng.gen_range(-0.1, 0.1)];
//     }
//     return particles;
// }

fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage<Window>>], 
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>, 
    dynamic_state: &mut DynamicState
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
    let dimensions = images[0].dimensions(); 

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32], 
        depth_range: 0.0 .. 1.0, 
    }; 

    dynamic_state.viewports = Some(vec!(viewport)); 

    images.iter().map(|image| {
        Arc::new(
            Framebuffer::start(render_pass.clone())
                .add(image.clone()).unwrap()
                .build().unwrap()
        ) as Arc<dyn FramebufferAbstract + Send + Sync>
    }).collect::<Vec<_>>()
}

