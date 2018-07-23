#![allow(dead_code)]

extern crate winit;

#[macro_use]
extern crate vulkano;

#[macro_use]
extern crate vulkano_shader_derive;

extern crate vulkano_win;

extern crate vulkano_text;

extern crate cgmath;

extern crate time;

mod camera;
mod fps;
mod frame;
mod vulkan;

use cgmath::Matrix4;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::buffer::CpuBufferPool;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::DynamicState;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::framebuffer::Framebuffer;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::framebuffer::Subpass;
use vulkano::image::AttachmentImage;
use vulkano::image::ImageUsage;
use vulkano::instance::Instance;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::swapchain;
use vulkano::swapchain::AcquireError;
use vulkano::swapchain::PresentMode;
use vulkano::swapchain::SurfaceTransform;
use vulkano::swapchain::Swapchain;
use vulkano::swapchain::SwapchainCreationError;
use vulkano::sync::now;
use vulkano::sync::GpuFuture;
use vulkano_text::DrawTextTrait;

use vulkano::instance::debug::DebugCallback;

use std::mem;
use std::sync::Arc;

use cgmath::SquareMatrix;

fn main() {
    let mut fps = fps::FPS::new(time::Duration::milliseconds(100));

    let instance = vulkan::initialize_instance();
    let mut scene = vulkan::Scene::new(&instance);

    let mut cam = camera::Camera2::new();

    let depth_buffer = AttachmentImage::transient(
        scene.device.clone(),
        scene.images[0].dimensions(),
        Format::D16Unorm,
    ).unwrap();

    let render_pass = Arc::new(
        single_pass_renderpass!(
            scene.queue.device().clone(),
            attachments: {
                final_color: {
                    load: Clear,
                    store: Store,
                    format: scene.swapchain.format(),
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
                color: [final_color],
                depth_stencil: {depth}
            }
        ).unwrap(),
    );

    let sub_pass = Subpass::from(render_pass.clone(), 0).unwrap();

    let (vs, fs) = create_shader_modules(scene.device.clone());

    let mut framebuffers: Option<Vec<Arc<vulkano::framebuffer::Framebuffer<_, _>>>> = None;

    let vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>> = {
        CpuAccessibleBuffer::from_iter(
            scene.device.clone(),
            BufferUsage::all(),
            [
                Vertex {
                    pos: [-0.5, -0.25, -0.5],
                    color: [1.0, 0.0, 0.0, 1.0],
                },
                Vertex {
                    pos: [0.0, 0.5, 1.0],
                    color: [0.0, 1.0, 0.0, 1.0],
                },
                Vertex {
                    pos: [0.25, -0.1, 0.0],
                    color: [0.0, 0.0, 1.0, 1.0],
                },
                Vertex {
                    pos: [0.0, 0.5, 1.0],
                    color: [0.0, 1.0, 0.0, 1.0],
                },
                Vertex {
                    pos: [0.25, -0.1, 0.0],
                    color: [0.0, 0.0, 1.0, 1.0],
                },
                Vertex {
                    pos: [0.5, 0.5, 0.0],
                    color: [0.0, 0.0, 1.0, 1.0],
                },
                Vertex {
                    pos: [0.5, 0.5, 0.0],
                    color: [0.0, 0.0, 1.0, 1.0],
                },
                Vertex {
                    pos: [1.5, 1.5, 0.0],
                    color: [0.0, 0.0, 1.0, 1.0],
                },
                Vertex {
                    pos: [0.5, 1.5, 0.0],
                    color: [0.0, 1.0, 1.0, 1.0],
                },
            ].iter()
                .cloned(),
        ).expect("Failed to create vertex buffer")
    };

    let pipeline: Arc<GraphicsPipelineAbstract + Send + Sync> = {
        Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .render_pass(sub_pass)
                .build(scene.device.clone())
                .unwrap(),
        ) as Arc<_>
    };

    let mvp = scene.camera.projection * cam.view_matrix() * scene.camera.world;
    let uniform_buffer_pool: CpuBufferPool<vs::ty::bufferVals> =
        CpuBufferPool::uniform_buffer(scene.device.clone());
    let mut ds_pool =
        vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool::new(pipeline.clone(), 0);

    let mut recreate_swapchain = false;
    let mut previous_frame_end = Box::new(now(scene.device.clone())) as Box<GpuFuture>;

    let _callback = DebugCallback::errors_and_warnings(&instance, |msg| {
        println!("Debug callback: {:?}", msg.description);
    }).ok();

    let mut text_drawer = vulkano_text::DrawText::new(
        scene.device.clone(),
        scene.queue.clone(),
        scene.swapchain.clone(),
        &scene.images,
    );

    // Frame system
    // let mut frame_system = frame::FrameSystem::new(scene.queue.clone(), scene.swapchain.format());

    loop {
        previous_frame_end.cleanup_finished();

        if recreate_swapchain {
            let dimensions = {
                let (new_width, new_height) = scene.window.window().get_inner_size().unwrap();
                [new_width, new_height]
            };

            let (new_swapchain, new_images) =
                match scene.swapchain.recreate_with_dimension(dimensions) {
                    Ok(r) => r,
                    // Tends to happen on manual resize
                    Err(SwapchainCreationError::UnsupportedDimensions) => {
                        continue;
                    }
                    Err(err) => panic!("{:?}", err),
                };

            mem::replace(&mut scene.swapchain, new_swapchain);
            mem::replace(&mut scene.images, new_images);

            framebuffers = None;

            recreate_swapchain = false;
        }

        if framebuffers.is_none() {
            let new_framebuffers = Some(
                scene
                    .images
                    .iter()
                    .map(|image| {
                        Arc::new(
                            Framebuffer::start(render_pass.clone())
                                .add(image.clone())
                                .unwrap()
                                .add(depth_buffer.clone())
                                .unwrap()
                                .build()
                                .unwrap(),
                        )
                    })
                    .collect::<Vec<_>>(),
            );

            mem::replace(&mut framebuffers, new_framebuffers);
        }

        let (image_num, acquire_future) =
            match swapchain::acquire_next_image(scene.swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    println!("ood");
                    recreate_swapchain = true;
                    continue;
                }
                Err(err) => panic!("{:?}", err),
            };

        let [width, height] = scene.images[0].dimensions();

        let mvp = scene.camera.projection * cam.view_matrix() * scene.camera.world;
        let uniform_buffer = uniform_buffer_pool.next(vs::ty::bufferVals { mvp: mvp.into() }).unwrap();
        let descriptor_set = ds_pool.next().add_buffer(uniform_buffer).unwrap().build().unwrap();

        text_drawer.queue_text(
            200.0,
            50.0,
            20.0,
            [1.0, 1.0, 1.0, 1.0],
            &format!(
                "Reander time: {} ms ({} FPS)",
                fps.average_render_time(),
                fps.current_fps()
            ),

        );

        let mut command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(
            scene.device.clone(),
            scene.queue.family(),
        ).unwrap()
            .begin_render_pass(
                framebuffers.as_ref().unwrap()[image_num].clone(),
                false,
                vec![[0.0, 0.0, 0.0, 0.0].into(), 1.0f32.into()],
            )
            .unwrap()
            .draw(
                pipeline.clone(),
                DynamicState {
                    viewports: Some(vec![Viewport {
                        origin: [0.0, 0.0],
                        dimensions: [width as f32, height as f32],
                        depth_range: 0.0..1.0,
                    }]),
                    ..DynamicState::none()
                },
                vec![vertex_buffer.clone()],
                descriptor_set,
                (),
            )
            .unwrap()
            .end_render_pass()
            .unwrap()
            .draw_text(&mut text_drawer, image_num)
            .build().unwrap();

        let future = previous_frame_end.join(acquire_future)
            .then_execute(scene.queue.clone(), command_buffer).unwrap()
            .then_swapchain_present(scene.queue.clone(), scene.swapchain.clone(), image_num)
            .then_signal_fence_and_flush().unwrap();

//         let mut frame =
//             frame_system.frame(future, scene.images[image_num].clone(), depth_buffer.clone(), Matrix4::identity());
//         let mut
// after_future = None;
//         while let Some(pass) = frame.next_pass() {
//             match pass {
//                 frame::Pass::Deferred(mut draw_pass) => {
//                     let mvp = scene.camera.projection * cam.view_matrix() * scene.camera.world;
//                     let uniform_buffer = uniform_buffer_pool
//                         .next(vs::ty::bufferVals { mvp: mvp.into() })
//                         .unwrap();
//                     let descriptor_set = ds_pool
//                         .next()
//                         .add_buffer(uniform_buffer)
//                         .unwrap()
//                         .build()
//                         .unwrap();

//                     let cb = AutoCommandBufferBuilder::secondary_graphics(
//                         scene.queue.device().clone(),
//                         scene.queue.family(),
//                         pipeline.clone().subpass(),
//                     ).unwrap()
//                         .draw(
//                             pipeline.clone(),
//                             DynamicState {
//                                 viewports: Some(vec![Viewport {
//                                     origin: [0.0, 0.0],
//                                     dimensions: [width as f32, height as f32],
//                                     depth_range: 0.0..1.0,
//                                 }]),
//                                 ..DynamicState::none()
//                             },
//                             vec![vertex_buffer.clone()],
//                             descriptor_set,
//                             (),
//                         )
//                         .unwrap()
//                         .build()
//                         .unwrap();

//                     draw_pass.execute(cb);
//                 },
//                 frame::Pass::Text(mut text_pass) => {
//                     text_pass.write(
//                         &format!(
//                             "Reander time: {} ms ({} FPS)",
//                             fps.average_render_time(),
//                             fps.current_fps()
//                         ),
//                         &mut text_drawer, image_num);
//                 },
//                 frame::Pass::Finished(af) => {
//                     after_future = Some(af);
//                 },
//                 _ => { }
//             }
//         }

        // let after_frame = after_future
        //     .unwrap()
        //     .then_swapchain_present(scene.queue.clone(), scene.swapchain.clone(), image_num)
        //     .then_signal_fence_and_flush()
        //     .unwrap();

        previous_frame_end = Box::new(future) as Box<_>;

        fps.end_frame();

        let mut done = false;
        scene.events_loop.poll_events(|ev| match ev {
            winit::Event::WindowEvent {
                event: winit::WindowEvent::Closed,
                ..
            } => done = true,
            winit::Event::WindowEvent {
                event: winit::WindowEvent::Resized(_, _),
                ..
            } => {
                recreate_swapchain = true;
                println!("resize");
            }
            winit::Event::WindowEvent {
                event: winit::WindowEvent::KeyboardInput { input, .. },
                ..
            } => cam.handle_input(&input, fps.average_render_time() as f32 / 1000.0),
            _ => (),
        });

        if done {
            return;
        }
    }
}

fn create_shader_modules(device: Arc<Device>) -> (vs::Shader, fs::Shader) {
    let vs = vs::Shader::load(device.clone()).expect("Could not create shader module");
    let fs = fs::Shader::load(device.clone()).expect("Could not create shader module");

    return (vs, fs);
}

mod vs {
    #[derive(VulkanoShader)]
    #[ty = "vertex"]
    #[src = "
#version 400

#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (std140, binding = 0) uniform bufferVals {
    mat4 mvp;
} myBufferVals;

layout (location = 0) in vec3 pos;
layout (location = 1) in vec4 color;
layout (location = 0) out vec4 out_color;
void main() {
    out_color = color;
    gl_Position = myBufferVals.mvp * vec4(pos, 1.0);
    // gl_Position = vec4(pos, 1.0);
}
"]
    struct Dummy;
}

mod fs {
    #[derive(VulkanoShader)]
    #[ty = "fragment"]
    #[src = "
#version 400
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable
layout (location = 0) in vec4 color;
layout (location = 0) out vec4 f_color;
void main() {
   //outColor = vec4(1.0, 0.0, 0.0, 1.0);
   f_color = color;
}
"]
    struct Dummy;
}

#[derive(Debug, Clone)]
struct Vertex {
    pos: [f32; 3],
    color: [f32; 4],
}
impl_vertex!(Vertex, pos, color);
