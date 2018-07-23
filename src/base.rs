use std::sync::Arc;
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::swapchain::Surface;
use vulkano::swapchain::{PresentMode, SurfaceTransform, Swapchain};
use vulkano::sync::Semaphore;
use vulkano_win;
use vulkano_win::VkSurfaceBuild;
use winit;

pub struct VkBase<'a> {
    physical: PhysicalDevice<'a>,
    window: Arc<Surface<winit::Window>>,
    events_loop: winit::EventsLoop,
    device: Arc<Device>,
    queue: Arc<Queue>,
}

impl<'a> VkBase<'a> {
    fn new(instance: &'a Arc<Instance>) -> VkBase<'a> {
        let physical = VkBase::initialize_physical_device(instance);
        let events_loop = winit::EventsLoop::new();
        // let window = VkBase::initialize_window(instance);
        let window = winit::WindowBuilder::new()
            .build_vk_surface(&events_loop, instance.clone())
            .unwrap();
        let (device, queue) = VkBase::get_device_and_queue(&window, physical);

        VkBase {
            physical,
            window,
            events_loop,
            device,
            queue,
        }
    }

    fn initialize_vulkano() -> Arc<Instance> {
        let extensions = vulkano_win::required_extensions();

        Instance::new(None, &extensions, None).expect("Failed to create Vulkan instance")
    }

    fn initialize_physical_device(instance: &Arc<Instance>) -> PhysicalDevice {
        PhysicalDevice::enumerate(&instance)
            .next()
            .expect("No device available")
    }

    fn initialize_window(instance: &Arc<Instance>) -> Arc<Surface<winit::Window>> {
        let events_loop = winit::EventsLoop::new();

        let window = winit::WindowBuilder::new()
            .build_vk_surface(&events_loop, instance.clone())
            .unwrap();
    }

    fn get_device_and_queue(
        window: &Arc<Surface<winit::Window>>,
        physical: PhysicalDevice,
    ) -> (Arc<Device>, Arc<Queue>) {
        // TODO: Queue selection
        let queue = physical
            .queue_families()
            .find(|&q| q.supports_graphics() && window.is_supported(q).unwrap_or(false))
            .expect("Couldn't find a graphical queue");

        let (device, mut queues) = {
            let device_ext = DeviceExtensions {
                khr_swapchain: true,
                ..DeviceExtensions::none()
            };

            Device::new(
                physical,
                physical.supported_features(),
                &device_ext,
                [(queue, 0.5)].iter().cloned(),
            ).expect("Failed to create device")
        };
        let queue = queues.next().unwrap();

        return (device, queue);
    }

    fn prepare(&mut self) {
        self.init_swapchain();
    }

    fn init_swapchain(&mut self) {
        let (mut swapchain, mut images) = {
            let caps = self.window
                .capabilities(self.physical)
                .expect("Failed to get surface capabilities");

            let alpha = caps.supported_composite_alpha.iter().next().unwrap();
            let dimensions = {
                let (width, height) = self.window.window().get_inner_size().unwrap();
                [width, height]
            };
            let dimensions = caps.current_extent.unwrap_or(dimensions);

            let format = caps.supported_formats[0].0;

            Swapchain::new(
                self.device.clone(),
                self.window.clone(),
                caps.min_image_count,
                format,
                dimensions,
                1,
                caps.supported_usage_flags,
                &self.queue,
                SurfaceTransform::Identity,
                alpha,
                PresentMode::Fifo,
                true,
                None,
            ).expect("Failed to create swapchain")
        };

        // self.swapchain = swapchain;
    }
}

pub struct Triangle<'a> {
    vk_base: VkBase<'a>,
    present_complete_semaphore: Option<Semaphore>,
    render_complete_semaphore: Option<Semaphore>,
}

impl<'a> Triangle<'a> {
    fn prepare(&mut self) {
        self.vk_base.prepare();
        self.prepare_synchronization_primitives()
    }

    fn prepare_synchronization_primitives(&mut self) {
        self.present_complete_semaphore = Some(
            Semaphore::from_pool(self.vk_base.device.clone()).expect("Could not get semaphore"),
        );
        self.render_complete_semaphore = Some(
            Semaphore::from_pool(self.vk_base.device.clone()).expect("Could not get semaphore"),
        );
    }
}

// fn main() {
//     // Create Vulkano instance
//     let instance = {
//         let extensions = vulkano_win::required_extensions();
//         // let app_info = vulkano::instance::ApplicationInfo::default();

//         // for layer in vulkano::instance::layers_list().unwrap() {
//         //     println!("Available layer: {}", layer.name());
//         // }
//         // let layers = vulkano::instance::layers_list().unwrap();

//         Instance::new(None, &extensions, None).expect("failed to create Vulkan instance")
//     };

//     // Choose device (first for simplicity)
//     let physical = vulkano::instance::PhysicalDevice::enumerate(&instance)
//         .next().expect("No device available");

//     // Debug information
//     println!("Using device: {} (type: {:?})", physical.name(), physical.ty());

//     // Create window
//     let mut events_loop = winit::EventsLoop::new();
//     let window = winit::WindowBuilder::new().build_vk_surface(&events_loop, instance.clone()).unwrap();
//     let mut dimensions = {
//         let (width, height) = window.window().get_inner_size().unwrap();
//         [width, height]
//     };

//     // Choose GPU queue
//     // TODO: Start using 2 queues, 1 for draw and 1 for compute
//     let queue = physical.queue_families().find(|&q| {
//         // Take first queue that support drawing
//         q.supports_graphics() && window.is_supported(q).unwrap_or(false)
//     }).expect("Couldn't find a graphical queue");

//     // Initialize device
//     let (device, mut queues) = {
//         let device_ext = vulkano::device::DeviceExtensions {
//             khr_swapchain: true,
//             .. vulkano::device::DeviceExtensions::none()
//         };

//         Device::new(physical, physical.supported_features(), &device_ext,
//         [(queue, 0.5)].iter().cloned()).expect("Failed to create device")
//     };

//     let queue = queues.next().unwrap();

//     // Create swapchain
//     let (mut swapchain, mut images) = {
//         let caps = window.capabilities(physical).expect("Failed to get surface capabilities");

//         // let dimensions = caps.current_extent.unwrap_or([width, height]);

//         let alpha = caps.supported_composite_alpha.iter().next().unwrap();
//         dimensions = caps.current_extent.unwrap_or(dimensions);

//         let format = caps.supported_formats[0].0;

//         Swapchain::new(device.clone(), window.clone(), caps.min_image_count, format,
//                        dimensions, 1, caps.supported_usage_flags, &queue,
//                        SurfaceTransform::Identity, alpha, PresentMode::Fifo, true,
//                        None).expect("Failed to create swapchain")
//     };

//     // Buffer for triangle store
//     let vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>> = {
//         CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), [
//             Vertex { position: [-0.5, -0.25] },
//             Vertex { position: [0.0, 0.5] },
//             Vertex { position: [0.25, -0.1] }
//         ].iter().cloned()).expect("Failed to create buffer")
//     };

//     // Create shaders
//     mod vs {
//         #[derive(VulkanoShader)]
//         #[ty = "vertex"]
//         #[src = "
// #version 450

// layout(location = 0) in vec2 position;

// void main() {
//     gl_Position = vec4(position, 0.0, 1.0);
// }
// "]
//         struct Dummy;
//     }

//     mod fs {
//         #[derive(VulkanoShader)]
//         #[ty = "fragment"]
//         #[src = "
// #version 450

// layout(location = 0) out vec4 f_color;

// void main() {
//     f_color = vec4(1.0, 0.0, 0.0, 1.0);
// }
// "]
//         struct Dummy;
//     }

//     // Create render pass
//     let render_pass = Arc::new(single_pass_renderpass!(
//         device.clone(),
//         attachments: {
//             color: {
//                 load: Clear,
//                 store: Store,
//                 format: swapchain.format(),
//                 samples: 1,
//             }
//         },
//         pass: {
//             color: [color],
//             depth_stencil: {}
//         }
//     ).unwrap());

//     let pipeline: Arc<GraphicsPipelineAbstract + Send + Sync> = {
//         let vs = vs::Shader::load(device.clone()).expect("Failed to create shader module");
//         let fs = fs::Shader::load(device.clone()).expect("Failed to create shader module");

//         Arc::new(GraphicsPipeline::start()
//                  .vertex_input_single_buffer::<Vertex>()
//                  .vertex_shader(vs.main_entry_point(), ())
//                  .triangle_list()
//                  .viewports_dynamic_scissors_irrelevant(1)
//                  .fragment_shader(fs.main_entry_point(), ())
//                  .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
//                  .build(device.clone())
//                  .unwrap()) as Arc<_>
//     };

//     let mut framebuffers: Option<Vec<Arc<vulkano::framebuffer::Framebuffer<_,_>>>> = None;

//     // Swapchains may invalidate on resize, remember if we need to recreate
//     let mut recreate_swapchain = false;

//     // Avoid GpuFutre blocking
//     let mut previous_frame_end = Box::new(now(device.clone())) as Box<GpuFuture>;

//     loop {
//         // Polls fences to free resources no longer needed
//         previous_frame_end.cleanup_finished();

//         if recreate_swapchain {
//             dimensions = {
//                 let (new_width, new_height) = window.window().get_inner_size().unwrap();
//                 [new_width, new_height]
//             };

//             let (new_swapchain, new_images) = match swapchain.recreate_with_dimension(dimensions) {
//                 Ok(r) => r,
//                 // Tends to happen on manual resize
//                 Err(SwapchainCreationError::UnsupportedDimensions) => {
//                     continue;
//                 },
//                 Err(err) => panic!("{:?}", err)
//             };

//             mem::replace(&mut swapchain, new_swapchain);
//             mem::replace(&mut images, new_images);

//             framebuffers = None;

//             recreate_swapchain = false;
//         }

//         if framebuffers.is_none() {
//             let new_framebuffers = Some(images.iter().map(|image| {
//                 Arc::new(Framebuffer::start(render_pass.clone())
//                          .add(image.clone()).unwrap()
//                          .build().unwrap())
//             }).collect::<Vec<_>>());
//             mem::replace(&mut framebuffers, new_framebuffers);
//         }
//         let (image_num, acquire_future) = match swapchain::acquire_next_image(swapchain.clone(), None) {
//             Ok(r) => r,
//             Err(AcquireError::OutOfDate) => {
//                 recreate_swapchain = true;
//                 continue;
//             },
//             Err(err) => panic!("{:?}", err)
//         };

//         // Build command buffer to draw (expensive operation)
//         let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap()
//             .begin_render_pass(framebuffers.as_ref().unwrap()[image_num].clone(), false, vec![[0.0, 0.0, 1.0, 1.0].into()])
//             .unwrap()
//             .draw(pipeline.clone(),
//                   DynamicState {
//                       line_width: None,
//                       // TODO: https://github.com/vulkano-rs/vulkano-examples/blob/master/triangle/main.rs#L420
//                       // Find a way to do this without having to dynamically allocate a Vec every frame.
//                       viewports: Some(vec![Viewport {
//                           origin: [0.0, 0.0],
//                           dimensions: [dimensions[0] as f32, dimensions[1] as f32],
//                           depth_range: 0.0 .. 1.0
//                       }]),
//                       scissors: None,
//                   },
//                   vec![vertex_buffer.clone()], (), ())
//             .unwrap()
//             .end_render_pass()
//             .unwrap().build().unwrap();

//         let future = previous_frame_end.join(acquire_future)
//             .then_execute(queue.clone(), command_buffer).unwrap()
//             .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
//             .then_signal_fence_and_flush().unwrap();
//         previous_frame_end = Box::new(future) as Box<_>;

//         let mut done = false;
//         events_loop.poll_events(|ev| {
//             match ev {
//                 winit::Event::WindowEvent { event: winit::WindowEvent::Closed, .. } => done = true,
//                 winit::Event::WindowEvent { event: winit::WindowEvent::Resized(_, _), .. } => recreate_swapchain = true,
//                 _ => ()
//             }
//         });
//         if done { return; }
//     }
// }

// #[derive(Debug, Clone)]
// struct Vertex { position: [f32; 2] }
// impl_vertex!(Vertex, position);
