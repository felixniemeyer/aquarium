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
		},
        blend::{
            AttachmentBlend, 
            BlendFactor, 
            BlendOp, 
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
        RenderPassAbstract,
    },

    image::{
        SwapchainImage, 
        Dimensions, 
        ImmutableImage, 
        StorageImage,
        ImageUsage,
        AttachmentImage,
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

use cgmath::{
    prelude::*,
	Matrix4, 
	Point3, 
	Vector3,
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
	const PARTICLE_COUNT: usize = 512; //1024 - vielleicht 256 pro spezies?  

	let mut rng = thread_rng();

    let fish_colors = match image::open("./fish/0001-colors.png") {
        Ok(image) => image, 
        Err(err) => panic!("{:?}", err)
    };

    let fish_normals = match image::open("./fish/0001-normals.png") {
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
            },
            depth: {
                load: Clear, 
                store: DontCare, 
                format: Format::D16Unorm, 
                samples: 1, 
            }
        },
        pass: {
            color: [color],
            depth_stencil: {depth}
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

    #[allow(dead_code)] // Used to force recompilation of shader change
    const S_SKY_FS: &str = include_str!("./shader/sky.fs.glsl");
    mod sky_fs { 
        vulkano_shaders::shader!{
            ty: "fragment", 
            path: "./src/shader/sky.fs.glsl"
        }
    }

    let fish_vs = fish_vs::Shader::load(device.clone()).unwrap(); 
    let fish_gs = fish_gs::Shader::load(device.clone()).unwrap(); 
    let fish_fs = fish_fs::Shader::load(device.clone()).unwrap(); 

    let flux_cp = flux_cp::Shader::load(device.clone()).unwrap(); 
	let particle_cp = particle_cp::Shader::load(device.clone()).unwrap(); 

    let general_2d_vs = general_2d_vs::Shader::load(device.clone()).unwrap(); 
    let debug_draw_flux_fs = debug_draw_flux_fs::Shader::load(device.clone()).unwrap(); 
    let sky_fs = sky_fs::Shader::load(device.clone()).unwrap();


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
        Format::R16G16B16A16Snorm,
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
        .depth_stencil_simple_depth()
        .blend_pass_through()
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap()
    ); 


    let sky_pipeline = Arc::new(GraphicsPipeline::start()
        .vertex_input(SingleBufferDefinition::<VertexTwoDTex>::new())
        .vertex_shader(general_2d_vs.main_entry_point(), ())
        .triangle_strip()
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(sky_fs.main_entry_point(), ())
        .blend_collective(AttachmentBlend {
            enabled: true, 
            color_op: BlendOp::Add,
            color_source: BlendFactor::OneMinusDstAlpha, 
            color_destination: BlendFactor::DstAlpha,
            alpha_op: BlendOp::Max, 
            alpha_source: BlendFactor::One, 
            alpha_destination: BlendFactor::One,
            mask_red: true, 
            mask_green: true, 
            mask_blue: true, 
            mask_alpha: true
        })
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap()
    );

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
    let vertex_data: [Vertex; PARTICLE_COUNT] = [
		Vertex { 
			position: [0., 0., 0., 1.0],
			tail: [1.0, 0., 0., 1.0]
		}; PARTICLE_COUNT 
	];
	// TODO: use DeviceLocalBuffer
    let fish_vertex_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(), 
        BufferUsage::all(), 
        false, 
        vertex_data.iter().cloned()
    ).unwrap();

    let (fish_colors_texture, fish_colors_future) = {
        let img_dim = fish_colors.dimensions();
        match ImmutableImage::from_iter(
            fish_colors.as_rgba8().unwrap().pixels().map(|rgba| {
                let bytes : [u8; 4] = [rgba[0], rgba[1], rgba[2], rgba[3]]; 
                bytes
            }),
            Dimensions::Dim2d { width: img_dim.0, height: img_dim.1 },
            Format::R8G8B8A8Unorm, 
            queue.clone()
        ) {
            Ok(i) => i, 
            Err(err) => panic!("{:?}", err)
        }
    };

    let (fish_normals_texture, fish_normals_future) = {
        let img_dim = fish_normals.dimensions();
        match ImmutableImage::from_iter(
            fish_normals.as_rgb8().unwrap().pixels().map(|rgb| {
                let bytes : [u8; 4] = [rgb[0], rgb[1], rgb[2], 255]; 
                bytes
            }),
            Dimensions::Dim2d { width: img_dim.0, height: img_dim.1 },
            Format::R8G8B8A8Unorm, 
            queue.clone()
        ) {
            Ok(i) => i, 
            Err(err) => panic!("{:?}", err)
        }
    };


    let fish_skin_sampler = Sampler::new(device.clone(), Filter::Linear, Filter::Linear,
        MipmapMode::Nearest, SamplerAddressMode::Repeat, SamplerAddressMode::Repeat, 
        SamplerAddressMode::Repeat, 0.0, 1.0, 0.0, 0.0).unwrap(); 
	let fish_pipeline_layout = fish_pipeline.layout().descriptor_set_layout(0).unwrap();
    let fish_desc_set = Arc::new(PersistentDescriptorSet::start(fish_pipeline_layout.clone())
        .add_sampled_image(fish_colors_texture.clone(), fish_skin_sampler.clone()).unwrap()
        .add_sampled_image(fish_normals_texture.clone(), fish_skin_sampler.clone()).unwrap()
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
        size: f32,
        offset: [f32; 3],
        padding_0: f32,
        drift: [f32;3],
        padding_1: f32,
	}
	let mut particle_data: [Particle; PARTICLE_COUNT] = {
		unsafe { MaybeUninit::uninit().assume_init() }
	};
	// TODO: use DeviceLocalBuffer
	for i in 0..particle_data.len() {
		particle_data[i].position 	= random_point_in_sphere(&mut rng); 
		particle_data[i].offset 	= random_point_in_sphere(&mut rng); 
        particle_data[i].drift 		= random_point_in_sphere(&mut rng);
        particle_data[i].size = rng.gen_range(0.07, 0.13); 
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

    /////////
    // sky //
    /////////
	
    let sky_vb = {
		let x = -1.0;
		let y = -1.0; 
		let s = 2.0;
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
    
    let mut framebuffers = window_size_dependent_setup(device.clone(),
         &images,
         render_pass.clone(),
         &mut dynamic_state
    ); 

    let mut recreate_swapchain = false; 

 	let mut previous_frame_end = Some(
        Box::new(sync::now(device.clone())
        .join(fish_colors_future.join(fish_normals_future))) as Box<dyn GpuFuture>); 

    let t0 = time::SystemTime::now(); 
    let mut now = t0; 
    let mut then = t0;

	let mut view: Matrix4<f32> = Matrix4::<f32>::from([[0.0;4];4]);
    let s = 0.007776; 
	let perspective: Matrix4<f32> = cgmath::frustum(-s, s, -s, s, 0.01, 10.0);

	let mut view_perspective: Matrix4<f32> = Matrix4::<f32>::from([[0.0;4];4]); 


    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent { event: WindowEvent::Resized(_), .. } => {
				recreate_swapchain = true; 
				
			}
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
                    framebuffers = window_size_dependent_setup(device.clone(),
						 &new_images,
						 render_pass.clone(),
						 &mut dynamic_state
                    ); 
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

                let angle = cgmath::Deg(time * 1.0);
                let updown = cgmath::Deg(time * 4.0).sin();
                let r = cgmath::Deg(time * 2.0).sin() * 0.2 + 1.2;
                let camera = Point3::new(
                    angle.sin() * r, 
                    updown * 0.5, 
                    angle.cos() * r
                );
                let center = Point3::new(0.0, 0.0, 0.0);
                let up = Vector3::new(0.0, 1.0, 0.0);

                view = Matrix4::look_at(camera, center, up);

                let straight = (center - camera).normalize(); 
                let right = straight.cross(up).normalize(); 
                let bottom = straight.cross(right).normalize(); 

                view_perspective = perspective * view; 

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
					viewPerspective: view_perspective.into(),
                    cameraPos: camera.into(),
                    time,
                    dtime,
                };

                let sky_push_constants = sky_fs::ty::PCData {
                    straight: straight.into(), 
                    right: right.into(), 
                    bottom: bottom.into(),
                    dummy: 0.0, 
                    dummy2: 0.0,
                };

                let clear_values = vec!([0.03, 0.13, 0.3, 0.0].into(), 1f32.into()); 
                let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(
                    device.clone(), 
                    queue.family()
                )
                    .unwrap()
					.dispatch(
						[PARTICLE_COUNT as u32, 1, 1], 
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
                        fish_push_constants
                    ).unwrap()
                    .draw(
                        sky_pipeline.clone(),
                        &dynamic_state, 
                        sky_vb.clone(),
                        (), 
                        sky_push_constants,
                    ).unwrap()
                    //.draw(
                    //    debug_draw_flux_pipeline.clone(), 
                    //    &dynamic_state, 
                    //    debug_draw_flux_vertex_buffer.clone(), 
                    //    debug_draw_flux_desc_set.clone(), 
                    //    ()
                    //).unwrap()
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

fn random_point_in_sphere<T: Rng>(rng: &mut T) -> [f32; 3] {
	let mut gen = || [
		rng.gen_range(-1.0,1.0),
		rng.gen_range(-1.0,1.0),
		rng.gen_range(-1.0,1.0),
	];
	let mut r:[f32;3] = gen();
	while (r[0].powf(2.0) + r[1].powf(2.0)).powf(2.0) + r[2].powf(2.0) > 1.0 {
		r = gen();
	}
	return r
}

fn window_size_dependent_setup(
    device: Arc<Device>, 
    images: &[Arc<SwapchainImage<Window>>], 
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>, 
    dynamic_state: &mut DynamicState
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
    let dimensions = images[0].dimensions(); 

    let depth_buffer = 
        AttachmentImage::transient(device.clone(), dimensions, Format::D16Unorm).unwrap(); 

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
                .add(depth_buffer.clone()).unwrap()
                .build().unwrap()
        ) as Arc<dyn FramebufferAbstract + Send + Sync>
    }).collect::<Vec<_>>()
}

