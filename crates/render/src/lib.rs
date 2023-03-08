mod vulkan;

use raw_window_handle::HasRawWindowHandle;

use self::vulkan as vk;

pub trait Render {
    fn render(&self);
}

pub struct Renderer {
    instance: vk::Instance,
    physical_device: vk::PhysicalDevice,
    device: vk::Device,
    queue: vk::Queue,
    surface: vk::SurfaceKhr,
}

impl Renderer {
    pub fn new<W: HasRawWindowHandle>(window: &W) -> Self {
        let instance = vk::Instance::new().unwrap();
        let surface = vk::SurfaceKhr::new(&instance, window);
        let (queue_family_index, physical_device) = instance
            .enumerate_physical_devices()
            .find_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .find(|&(i, props)| props.graphics() && p.surface_support_khr(i, &surface))
                    .map(|(i, _)| (i, p))
            })
            .unwrap();
        log::info!("Using: {}", physical_device.device_name());
        let device = vk::Device::new(&physical_device, queue_family_index);
        let queue = device.get_queue(queue_family_index, 0);

        Self {
            instance,
            physical_device,
            device,
            queue,
            surface,
        }
    }

    pub fn draw_frame(&self) {
        println!("drawing");
    }
}
