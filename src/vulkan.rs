use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::image::SwapchainImage;
use vulkano::instance;
use vulkano::instance::{
    ApplicationInfo, Instance, PhysicalDevice, QueueFamily,
};
use vulkano::swapchain::PresentMode;
use vulkano::swapchain::{Surface, SurfaceTransform, Swapchain};
use vulkano_win;
use vulkano_win::VkSurfaceBuild;
use winit;
use camera::Camera;

use std::borrow::Cow;
use std::sync::Arc;

pub struct Scene<'a> {
    instance: Arc<Instance>,
    physical: PhysicalDevice<'a>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    // command_buffer: AutoCommandBufferBuilder,
    pub events_loop: winit::EventsLoop,
    pub window: Arc<Surface<winit::Window>>,
    pub swapchain: Arc<Swapchain<winit::Window>>,
    pub images: Vec<Arc<SwapchainImage<winit::Window>>>,
    pub camera: Camera
}

impl<'a> Scene<'a> {
    pub fn new(instance: &'a Arc<Instance>) -> Scene<'a> {
        let physical = get_physical_device(&instance);
        let queue_family = get_queue_family(&physical);
        let (device, queue) = initialize_device_and_queues(&physical, queue_family);
        // let command_buffer = initialize_command_buffer(device.clone(), queue.clone());
        let (events_loop, window) = initialize_events_loop_and_window(instance.clone());
        let (swapchain, images) = initialize_swapchain(window.clone(), &physical, device.clone(), queue.clone());
        let camera = Camera::new();

        Scene {
            instance: instance.clone(),
            physical,
            device,
            queue,
            // command_buffer,
            events_loop,
            window,
            swapchain,
            images,
            camera
        }
    }
}

pub fn initialize_instance() -> Arc<Instance> {
    println!("Creating Vulkan instance");
    print_layer_list();

    let mut app_info = ApplicationInfo::default();
    app_info.application_name = Some(Cow::from("Vulkano test"));

    let extensions = vulkano_win::required_extensions();

    // let layer = "VK_LAYER_LUNARG_standard_validation";
    let layers = vec![];

    Instance::new(Some(&app_info), &extensions, layers).expect("Failed to create instance")
}

fn print_layer_list() {
    println!("Vulkan debugging layers available:");

    let mut layers = instance::layers_list().unwrap();
    while let Some(l) = layers.next() {
        println!("\t{}", l.name());
    }
}

fn get_physical_device<'a>(instance: &'a Arc<Instance>) -> PhysicalDevice<'a> {
    print_device_list(instance.clone());

    // Select first physical device
    let physical = PhysicalDevice::enumerate(&instance)
        .next()
        .expect("No physical device found");

    println!(
        "Using device: {} (type: {:?})",
        physical.name(),
        physical.ty()
    );

    physical
}

fn print_device_list(instance: Arc<Instance>) {
    for (i, physical) in PhysicalDevice::enumerate(&instance).enumerate() {
        println!(
            "Device {}: {} (type: {:?})",
            i,
            physical.name(),
            physical.ty()
        );
    }
}

// TODO: Create window

fn get_queue_family<'a>(physical: &'a PhysicalDevice) -> QueueFamily<'a> {
    print_queue_families(physical);

    let queue = physical
        .queue_families()
        .find(|&q| {
            q.supports_graphics() // TODO: window supported
        })
        .expect("Coulnd't find a graphical queue");

    queue
}

fn print_queue_families<'a>(physical: &'a PhysicalDevice) {
    println!("Queue families available:");

    for (i, queue_family) in physical.queue_families().enumerate() {
        println!("\tQueue Family Index: {:?}", i);
        println!("\t\tID {:?}", queue_family.id());
        println!("\t\tQueue Count: {:?}", queue_family.queues_count());
        println!(
            "\t\tSupports Graphics: {:?}",
            queue_family.supports_graphics()
        );
        println!(
            "\t\tSupports Compute: {:?}",
            queue_family.supports_compute()
        );
        println!(
            "\t\tSupports Transfers: {:?}",
            queue_family.supports_transfers()
        );
        println!(
            "\t\tSupports Sparse Biding: {:?}",
            queue_family.supports_sparse_binding()
        );
    }
}

fn initialize_device_and_queues<'a>(
    physical: &'a PhysicalDevice,
    queue_family: QueueFamily,
) -> (Arc<Device>, Arc<Queue>) {
    let (device, mut queues) = {
        // TODO: Look into extensions
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };

        Device::new(
            *physical,
            physical.supported_features(),
            &device_extensions,
            [(queue_family, 0.5)].iter().cloned(),
        ).expect("Failed to create device")
    };

    let queue = queues.next().unwrap();

    (device, queue)
}

fn initialize_command_buffer(device: Arc<Device>, queue: Arc<Queue>) -> AutoCommandBufferBuilder {
    AutoCommandBufferBuilder::primary(device.clone(), queue.family())
        .expect("Could not initialize command buffer")
}

fn initialize_events_loop_and_window(
    instance: Arc<Instance>,
) -> (winit::EventsLoop, Arc<Surface<winit::Window>>) {
    let events_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new()
        .build_vk_surface(&events_loop, instance.clone())
        .unwrap();

    (events_loop, window)
}

fn initialize_swapchain<'a>(
    window: Arc<Surface<winit::Window>>,
    physical: &'a PhysicalDevice,
    device: Arc<Device>,
    queue: Arc<Queue>,
) -> (
    Arc<Swapchain<winit::Window>>,
    Vec<Arc<SwapchainImage<winit::Window>>>,
) {
    let (swapchain, images) = {
        let caps = window
            .capabilities(*physical)
            .expect("Failed to get surface capabilities");

        // TODO: Select best alpha?
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let dimensions = {
            let (width, height) = window.window().get_inner_size().unwrap();
            [width, height]
        };
        let dimensions = caps.current_extent.unwrap_or(dimensions);

        // TODO: Select best format?
        let format = caps.supported_formats[0].0;

        Swapchain::new(
            device.clone(),
            window.clone(),
            caps.min_image_count,
            format,
            dimensions,
            1,
            caps.supported_usage_flags,
            &queue,
            SurfaceTransform::Identity,
            alpha,
            PresentMode::Fifo,
            true,
            None,
        ).expect("Failed to create swapchain")
    };

    (swapchain, images)
}
