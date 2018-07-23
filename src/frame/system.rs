use std::sync::Arc;

use cgmath::Matrix4;
use cgmath::SquareMatrix;
use cgmath::Vector3;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBuffer;
use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::framebuffer::Framebuffer;
use vulkano::framebuffer::FramebufferAbstract;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::framebuffer::Subpass;
use vulkano::image::AttachmentImage;
use vulkano::image::ImageAccess;
use vulkano::image::ImageUsage;
use vulkano::image::ImageViewAccess;
use vulkano::sync::GpuFuture;
use vulkano_text::{DrawText, DrawTextTrait};

pub struct FrameSystem {
    queue: Arc<Queue>,
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
}

impl FrameSystem {
    pub fn new(queue: Arc<Queue>, output_format: Format) -> FrameSystem {
        let render_pass = single_pass_renderpass!(
            queue.device().clone(),
            attachments: {
                final_color: {
                    load: Clear,
                    store: Store,
                    format: output_format,
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
        ).unwrap();

        FrameSystem {
            queue,
            render_pass: Arc::new(render_pass)
        }
    }

    pub fn deferred_render_pass(&self) -> Subpass<Arc<RenderPassAbstract + Send + Sync>> {
        Subpass::from(self.render_pass.clone(), 0).unwrap()
    }

    pub fn frame<F, I>(
        &mut self,
        before_future: F,
        final_image: I,
        depth_buffer: Arc<AttachmentImage>,
        world_to_framebuffer: Matrix4<f32>,
    ) -> Frame
    where
        F: GpuFuture + 'static,
        I: ImageAccess + ImageViewAccess + Clone + Send + Sync + 'static,
    {
        let framebuffer = Arc::new(
            Framebuffer::start(self.render_pass.clone())
                .add(final_image.clone())
                .unwrap()
                .add(depth_buffer.clone())
                .unwrap()
                .build()
                .unwrap()
        );

        let command_buffer = Some(
            AutoCommandBufferBuilder::primary_one_time_submit(
                self.queue.device().clone(),
                self.queue.family(),
            ).unwrap()
                .begin_render_pass(
                    framebuffer.clone(),
                    true,
                    vec![[0.0, 0.0, 0.0, 0.0].into(), 1.0f32.into()],
                )
                .unwrap(),
        );

        Frame {
            system: self,
            num_pass: 0,
            before_cb_main_future: Some(Box::new(before_future)),
            framebuffer,
            command_buffer,
            world_to_framebuffer,
        }
    }
}

pub struct Frame<'a> {
    system: &'a mut FrameSystem,
    num_pass: u8,
    before_cb_main_future: Option<Box<GpuFuture>>,
    framebuffer: Arc<FramebufferAbstract + Send + Sync>,
    command_buffer: Option<AutoCommandBufferBuilder>,
    world_to_framebuffer: Matrix4<f32>,
}

impl<'a> Frame<'a> {
    pub fn next_pass<'f>(&'f mut self) -> Option<Pass<'f, 'a>> {
        match {
            let current_pass = self.num_pass;
            self.num_pass += 1;
            current_pass
        } {
            0 => Some(Pass::Deferred(DrawPass { frame: self })),
            1 => {
                self.command_buffer = Some(
                    self
                        .command_buffer
                        .take()
                        .unwrap()
                        .end_render_pass()
                        .unwrap()
                );

                Some(Pass::Text(TextPass { frame: self }))
            },
            2 => {
                let command_buffer = self
                    .command_buffer
                    .take()
                    .unwrap()
                    .build()
                    .unwrap();

                let after_main_cb = self
                    .before_cb_main_future
                    .take()
                    .unwrap()
                    .then_execute(self.system.queue.clone(), command_buffer)
                    .unwrap();

                Some(Pass::Finished(Box::new(after_main_cb)))
            }
            _ => None,
        }
    }
}

pub enum Pass<'f, 's: 'f> {
    Deferred(DrawPass<'f, 's>),
    EndRenderPass,
    Text(TextPass<'f, 's>),
    Finished(Box<GpuFuture>),
}

pub struct DrawPass<'f, 's: 'f> {
    frame: &'f mut Frame<'s>,
}

impl<'f, 's: 'f> DrawPass<'f, 's> {
    #[inline]
    pub fn execute<C>(&mut self, command_buffer: C)
    where
        C: CommandBuffer + Send + Sync + 'static,
    {
        unsafe {
            self.frame.command_buffer = Some(
                self.frame
                    .command_buffer
                    .take()
                    .unwrap()
                    .execute_commands(command_buffer)
                    .unwrap(),
            );
        }
    }

    #[inline]
    pub fn viewport_dimensions(&self) -> [u32; 2] {
        let dims = self.frame.framebuffer.dimensions();
        [dims[0], dims[1]]
    }

    #[inline]
    pub fn world_to_framebuffer_matrix(&self) -> Matrix4<f32> {
        self.frame.world_to_framebuffer
    }
}

pub struct TextPass<'f, 's: 'f> {
    frame: &'f mut Frame<'s>
}

impl<'f, 's: 'f> TextPass<'f, 's> {
    #[inline]
    pub fn write(&mut self, text: &String, text_drawer: &mut DrawText, image_num: usize) {
        text_drawer.queue_text(
            200.0,
            50.0,
            20.0,
            [1.0, 1.0, 1.0, 1.0],
            &text
        );

        self.frame.command_buffer = Some(
            self.frame
                .command_buffer
                .take()
                .unwrap()
                .draw_text(text_drawer, image_num)
        );
    }
}
