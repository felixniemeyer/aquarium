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
//        ComputePipeline,
    },

    device::{
        Device,
        Features,
        RawDeviceExtensions,
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

    descriptor::descriptor_set::PersistentDescriptorSet,
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
        WindowEvent
    }
};


mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "./src/shader/particle_update.cp.glsl"
    }
}

#[derive(Copy, Clone, Debug)]
struct Particle {
    pos: [f32; 2],
    speed: [f32; 2],
    tail: [f32; 2],
    prev_pos: [f32; 2],
    prev_tail: [f32; 2],
}

// struct ImageDataIterator<P> {
//     originalIterator: image::buffer::Pie, 
//     length: usize
// }
// 
// impl<T> ImageDataIterator<T> { 
//     fn from_dynamic_image(img: &DynamicImage) -> ImageDataIterator<T> {
//         let dimensions = img.dimensions();
//         
//         ImageDataIterator { 
//             originalIterator: img.to_rgba().pixels(), 
//             length: ( dimensions.0 * dimensions.1 ) as usize
//         }
//     }
// }
// 
// impl<'a, T> Iterator for ImageDataIterator<'a, T> {
//     type Item = [u8; 4]; 
//     fn next(&mut self) -> Option<[u8; 4]> {
//         return match self.originalIterator.next() {
//             Some(pixel) => {
//                 let rgba = pixel.2;
//                 let data: [u8; 4] = [ rgba[0], rgba[1], rgba[2], rgba[3] ]; 
//                 return Some(data);
//             }, 
//             None => None
//         }
//     }
// }
// 
// impl<'a, T> ExactSizeIterator for ImageDataIterator<'a, T> {
//     fn len(&self) -> usize {
//         return self.length; 
//     }
// }



fn main() {

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
        .find(|&q| q.supports_graphics())
        .expect("couldn't find a graphical queue family");

    let (device, mut queues) = {
        let unraw_dev_exts = vulkano::device::DeviceExtensions {
            khr_swapchain: true, 
            .. vulkano::device::DeviceExtensions::none()
        };
        let mut dev_exts = RawDeviceExtensions::from(&unraw_dev_exts);
        dev_exts.insert(std::ffi::CString::new("VK_KHR_storage_buffer_storage_class").unwrap());


        let dev_features = Features {
            geometry_shader: true, 
            .. Features::none()
        };

        Device::new(
            physical,
            &dev_features, 
            dev_exts,
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

    #[derive(Default, Debug, Clone, Copy)]
    struct Vertex { 
        position: [f32; 3]
    }
    vulkano::impl_vertex!(Vertex, position); 

    // generate some random data
    let mut data: [Vertex; 128] = unsafe { MaybeUninit::uninit().assume_init() }; // unsafe {  }; //unsafe :D
    let mut rng = thread_rng();
    for i in 0..data.len() {
        data[i].position = [
            rng.gen_range(-1.0,1.0),
            rng.gen_range(-1.0,1.0),
            rng.gen_range(0.2,1.0),
        ]
    }

    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(), 
        BufferUsage::all(), 
        false, 
        data.iter().cloned()
    ).unwrap();

    // texture
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

    let sampler = Sampler::new(device.clone(), Filter::Linear, Filter::Linear, 
        MipmapMode::Nearest, SamplerAddressMode::Repeat, SamplerAddressMode::Repeat, 
        SamplerAddressMode::Repeat, 0.0, 1.0, 0.0, 0.0).unwrap(); 

    #[allow(dead_code)] // Used to force recompilation of shader change
    const X0: &str = include_str!("./shader/fish.vs.glsl");
    mod fish_vs { 
        vulkano_shaders::shader!{
            ty: "vertex", 
            path: "./src/shader/fish.vs.glsl"
        }
    }

    #[allow(dead_code)] // Used to force recompilation of shader change
    const X1: &str = include_str!("./shader/fish.fs.glsl");
    mod fish_fs { 
        vulkano_shaders::shader!{
            ty: "fragment", 
            path: "./src/shader/fish.fs.glsl"
        }
    }

    #[allow(dead_code)] // Used to force recompilation of shader change
    const X2: &str = include_str!("./shader/fish.geo.glsl");
    mod fish_geo { 
        vulkano_shaders::shader!{
            ty: "geometry", 
            path: "./src/shader/fish.geo.glsl"
        }
    }

    let fish_vs = fish_vs::Shader::load(device.clone()).unwrap(); 
    let fish_geo = fish_geo::Shader::load(device.clone()).unwrap(); 
    let fish_fs = fish_fs::Shader::load(device.clone()).unwrap(); 

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


    let pipeline = Arc::new(GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex>()
        .vertex_shader(fish_vs.main_entry_point(), ())
        .geometry_shader(fish_geo.main_entry_point(), ())
        .point_list()
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fish_fs.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap()); 


    let set = Arc::new(PersistentDescriptorSet::start(pipeline.layout().descriptor_set_layout(0).unwrap().clone())
        .add_sampled_image(autumn_texture.clone(), sampler.clone()).unwrap()
        .build().unwrap()
    );

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

                let time = now.duration_since(t0).unwrap().as_millis() as i32;
                let dtime = now.duration_since(then).unwrap().as_millis() as i32;

                let push_constants = fish_geo::ty::PushConstantData {
                    time,
                    dtime
                };

                let clear_values = vec!([0.0, 0.0, 0.0, 1.0].into()); 
                let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(
                    device.clone(), 
                    queue.family()
                )
                    .unwrap()
                    .begin_render_pass(framebuffers[image_num].clone(), false, clear_values)
                    .unwrap()
                    .draw(
                        pipeline.clone(), 
                        &dynamic_state, 
                        vertex_buffer.clone(), 
                        set.clone(), 
                        push_constants)
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

