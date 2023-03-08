#![allow(dead_code)]

use ash::vk;
use raw_window_handle::HasRawWindowHandle;

use dynamic_library::DynamicLibrary;

use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Error {
    code: vk::Result,
}

impl From<vk::Result> for Error {
    fn from(code: vk::Result) -> Self {
        Self { code }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}. ({})", self.code, self.code.as_raw())
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct Instance {
    inner: Arc<RawInstance>,
}

impl Instance {
    pub fn new() -> Result<Self> {
        #[cfg(windows)]
        let path = "vulkan-1.dll";
        #[cfg(not(all(windows, target_pointer_width = "64")))]
        compile_error!("Only Windows 64-bit is supported");

        let Some(lib) = DynamicLibrary::new(path) else {
            log::error!("Could not load dynamic library.");
            panic!()
        };
        let Some(get_instance_proc_addr) = lib
            .get("vkGetInstanceProcAddr")
            .map(|sym| unsafe { std::mem::transmute(sym.as_ptr()) }) else {
                log::error!("Could not retrieve symbol `vkGetInstanceProcAddr`.");
                panic!()
            };

        let static_fn = ash::vk::StaticFn {
            get_instance_proc_addr,
        };
        let entry = unsafe { ash::Entry::from_static_fn(static_fn) };

        let mut required_extensions = vec!["VK_KHR_surface", "VK_KHR_win32_surface"];
        if cfg!(debug_assertions) {
            log::info!("Debug utilities extension enabled.");
            required_extensions.push("VK_EXT_debug_utils");
        }
        let available_extensions = entry
            .enumerate_instance_extension_properties(None)?
            .into_iter()
            .map(ExtensionProperties::from)
            .collect::<Vec<_>>();

        let extensions_supported = required_extensions
            .iter()
            .all(|r| available_extensions.iter().any(|e| r == &e.extension_name));

        if !extensions_supported {
            panic!("Required extensions are not supported.");
        }

        let required_extensions = required_extensions
            .iter()
            .map(|&e| unsafe {
                // SAFETY: `required_extensions` does not contain nul.
                CString::from_vec_unchecked(e.into())
            })
            .collect::<Vec<_>>();
        let pp_enabled_extension_names = required_extensions
            .iter()
            .map(|e| e.as_ptr())
            .collect::<Vec<_>>();

        let app_info = vk::ApplicationInfo {
            p_application_name: unsafe {
                CStr::from_bytes_with_nul_unchecked(b"Vulkan Tutorial\0").as_ptr()
            },
            application_version: vk::make_api_version(0, 0, 1, 0),
            p_engine_name: unsafe {
                CStr::from_bytes_with_nul_unchecked(b"Tanelorn Engine\0").as_ptr()
            },
            engine_version: vk::make_api_version(0, 0, 1, 0),
            api_version: vk::API_VERSION_1_0,
            ..Default::default()
        };
        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            pp_enabled_extension_names: pp_enabled_extension_names.as_ptr(),
            enabled_extension_count: pp_enabled_extension_names
                .len()
                .try_into()
                .expect("Could not convert `usize` to `u32`"),
            ..Default::default()
        };
        let instance = unsafe { entry.create_instance(&create_info, None)? };


        log::trace!("Instance created.");
        Ok(Self {
            inner: Arc::new(RawInstance {
                _lib: lib,
                handle: instance,
            }),
        })
    }

    pub fn enumerate_physical_devices(&self) -> impl ExactSizeIterator<Item = PhysicalDevice> {
        // TODO: Document `SAFETY`
        unsafe {
            let physical_devices = self
                .inner
                .handle
                .enumerate_physical_devices()
                .unwrap()
                .into_iter()
                .map(move |p| {
                    let props = self.inner.handle.get_physical_device_properties(p);
                    let device_name = String::from_utf8_unchecked(
                        CStr::from_ptr(props.device_name.as_ptr()).to_bytes().into(),
                    );

                    PhysicalDevice {
                        handle: p,
                        instance: self.clone(),
                        props: PhysicalDeviceProperties {
                            device_type: props.device_type.into(),
                            device_name,
                        },
                    }
                })
                .collect::<Vec<PhysicalDevice>>();

            physical_devices.into_iter()
        }
    }
}

struct RawInstance {
    _lib: DynamicLibrary,
    handle: ash::Instance,
}

impl Drop for RawInstance {
    fn drop(&mut self) {
        log::trace!("Instance destroyed.");
        unsafe { self.handle.destroy_instance(None) }
    }
}

pub struct PhysicalDevice {
    handle: vk::PhysicalDevice,
    instance: Instance,
    props: PhysicalDeviceProperties,
}

impl PhysicalDevice {
    pub fn queue_family_properties(&self) -> Vec<QueueFamilyProperties> {
        let props = unsafe {
            self.instance
                .inner
                .handle
                .get_physical_device_queue_family_properties(self.handle)
        };

        props.into_iter().map(|p| p.into()).collect()
    }

    pub fn surface_support_khr(&self, queue_family_index: usize, surface: &SurfaceKhr) -> bool {
        let mut support = MaybeUninit::uninit();
        let res = unsafe {
            (surface.surface_fn.get_physical_device_surface_support_khr)(
                self.handle,
                queue_family_index
                    .try_into()
                    .expect("Failed to convert `usize` to `u32`"),
                surface.handle,
                support.as_mut_ptr(),
            )
        };

        if res == vk::Result::SUCCESS {
            unsafe { support.assume_init() != 0 }
        } else {
            panic!("Failed to get physical device surface support: {:?}", res)
        }
    }

    pub fn surface_capabilities(&self, surface: &SurfaceKhr) -> () {}

    pub fn device_name(&self) -> &str {
        &self.props.device_name
    }

    pub fn device_type(&self) -> PhysicalDeviceType {
        self.props.device_type
    }
}

pub struct PhysicalDeviceProperties {
    device_type: PhysicalDeviceType,
    device_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalDeviceType {
    Other,
    IntegratedGpu,
    DiscreteGpu,
    VirtualGpu,
    Cpu,
}

impl From<vk::PhysicalDeviceType> for PhysicalDeviceType {
    fn from(t: vk::PhysicalDeviceType) -> Self {
        match t {
            vk::PhysicalDeviceType::OTHER => Self::Other,
            vk::PhysicalDeviceType::INTEGRATED_GPU => Self::IntegratedGpu,
            vk::PhysicalDeviceType::DISCRETE_GPU => Self::DiscreteGpu,
            vk::PhysicalDeviceType::VIRTUAL_GPU => Self::VirtualGpu,
            vk::PhysicalDeviceType::CPU => Self::Cpu,
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}

#[derive(Debug)]
pub struct QueueFamilyProperties {
    capabilities: QueueFamilyCapabilities,
    queue_count: u32,
    timestamp_valid_bits: u32,
    min_image_transfer_granularity: Extent3D,
}

impl QueueFamilyProperties {
    pub fn graphics(&self) -> bool {
        self.capabilities.graphics
    }

    pub fn compute(&self) -> bool {
        self.capabilities.compute
    }

    pub fn transfer(&self) -> bool {
        self.capabilities.transfer
    }

    pub fn sparse_binding(&self) -> bool {
        self.capabilities.sparse_binding
    }

    pub fn video_decode(&self) -> bool {
        self.capabilities.video_decode
    }

    pub fn video_encode(&self) -> bool {
        self.capabilities.video_encode
    }

    pub fn protected(&self) -> bool {
        self.capabilities.protected
    }
}

impl From<vk::QueueFamilyProperties> for QueueFamilyProperties {
    fn from(props: vk::QueueFamilyProperties) -> Self {
        let capabilities = QueueFamilyCapabilities {
            graphics: props.queue_flags.contains(vk::QueueFlags::GRAPHICS),
            compute: props.queue_flags.contains(vk::QueueFlags::COMPUTE),
            transfer: props.queue_flags.contains(vk::QueueFlags::TRANSFER),
            sparse_binding: props.queue_flags.contains(vk::QueueFlags::SPARSE_BINDING),
            video_decode: props.queue_flags.contains(vk::QueueFlags::VIDEO_DECODE_KHR),
            video_encode: props.queue_flags.contains(vk::QueueFlags::VIDEO_ENCODE_KHR),
            protected: props.queue_flags.contains(vk::QueueFlags::PROTECTED),
        };

        Self {
            capabilities,
            queue_count: props.queue_count,
            timestamp_valid_bits: props.timestamp_valid_bits,
            min_image_transfer_granularity: props.min_image_transfer_granularity.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueueFamilyCapabilities {
    graphics: bool,
    compute: bool,
    transfer: bool,
    sparse_binding: bool,
    video_decode: bool,
    video_encode: bool,
    protected: bool,
}

#[derive(Debug, Clone)]
pub struct Extent3D {
    width: u32,
    height: u32,
    depth: u32,
}

impl From<vk::Extent3D> for Extent3D {
    fn from(
        vk::Extent3D {
            width,
            height,
            depth,
        }: vk::Extent3D,
    ) -> Self {
        Self {
            width,
            height,
            depth,
        }
    }
}

#[derive(Clone)]
pub struct Device {
    inner: Arc<RawDevice>,
    instance: Instance,
}

impl Device {
    pub fn new(physical_device: &PhysicalDevice, queue_family_index: usize) -> Self {
        let queue_priority = 1.0f32;
        let queue_create_info = vk::DeviceQueueCreateInfo {
            queue_family_index: queue_family_index
                .try_into()
                .expect("Could not convert `usize` to `u32`"),
            queue_count: 1,
            p_queue_priorities: &queue_priority,
            ..Default::default()
        };

        let required_extensions = ["VK_KHR_swapchain"];
        let available_extensions = unsafe {
            physical_device
                .instance
                .inner
                .handle
                .enumerate_device_extension_properties(physical_device.handle)
                .unwrap()
                .into_iter()
                .map(ExtensionProperties::from)
                .collect::<Vec<_>>()
        };
        let supported = required_extensions.iter().all(|required| {
            available_extensions
                .iter()
                .any(|available| required == &available.extension_name)
        });
        if !supported {
            panic!("Required device extension(s) are not supported.")
        }

        let enabled_extensions = required_extensions
            .iter()
            .map(|&e| unsafe {
                // SAFETY: `required_extensions` does not contain nul.
                CString::from_vec_unchecked(e.into())
            })
            .collect::<Vec<_>>();
        let enabled_extension_pointers = enabled_extensions
            .iter()
            .map(|e| e.as_ptr())
            .collect::<Vec<_>>();

        let create_info = vk::DeviceCreateInfo {
            queue_create_info_count: 1,
            p_queue_create_infos: &queue_create_info,
            enabled_extension_count: enabled_extension_pointers
                .len()
                .try_into()
                .expect("Failed to convert `usize` to `u32`"),
            pp_enabled_extension_names: enabled_extension_pointers.as_ptr(),
            ..Default::default()
        };

        let handle = unsafe {
            physical_device
                .instance
                .inner
                .handle
                .create_device(physical_device.handle, &create_info, None)
                .unwrap()
        };

        log::trace!("Device created.");
        Self {
            inner: Arc::new(RawDevice { handle }),
            instance: physical_device.instance.clone(),
        }
    }

    pub fn get_queue(&self, queue_family_index: usize, queue_index: usize) -> Queue {
        let handle = unsafe {
            self.inner.handle.get_device_queue(
                queue_family_index
                    .try_into()
                    .expect("Could not convert `usize` to `u32`"),
                queue_index
                    .try_into()
                    .expect("Could not convert `usize` to `u32`"),
            )
        };

        debug_assert_ne!(handle, vk::Queue::null());

        Queue {
            handle,
            device: self.clone(),
        }
    }
}

struct RawDevice {
    handle: ash::Device,
}

impl Drop for RawDevice {
    fn drop(&mut self) {
        unsafe { self.handle.destroy_device(None) };
        log::trace!("Device destroyed.");
    }
}

pub struct Queue {
    handle: vk::Queue,
    device: Device,
}

pub struct SurfaceKhr {
    handle: vk::SurfaceKHR,
    surface_fn: vk::KhrSurfaceFn,
    win32_surface_fn: vk::KhrWin32SurfaceFn,
    instance: Instance,
}

impl SurfaceKhr {
    pub fn new<W: HasRawWindowHandle>(instance: &Instance, window: &W) -> Self {
        use raw_window_handle::{RawWindowHandle, Win32WindowHandle};

        let get_instance_proc_addr = unsafe {
            std::mem::transmute::<_, ash::vk::PFN_vkGetInstanceProcAddr>(
                instance
                    .inner
                    ._lib
                    .get("vkGetInstanceProcAddr")
                    .unwrap_unchecked()
                    .as_ptr(),
            )
        };

        let loader = |name: &CStr| unsafe {
            std::mem::transmute(get_instance_proc_addr(
                instance.inner.handle.handle(),
                name.as_ptr(),
            ))
        };
        let surface_fn = vk::KhrSurfaceFn::load(loader);
        let win32_surface_fn = vk::KhrWin32SurfaceFn::load(loader);

        match window.raw_window_handle() {
            RawWindowHandle::Win32(Win32WindowHandle {
                hwnd, hinstance, ..
            }) => {
                let create_info = vk::Win32SurfaceCreateInfoKHR {
                    hinstance,
                    hwnd,
                    ..Default::default()
                };
                let mut handle = MaybeUninit::uninit();
                unsafe {
                    let res = (win32_surface_fn.create_win32_surface_khr)(
                        instance.inner.handle.handle(),
                        &create_info,
                        std::ptr::null(),
                        handle.as_mut_ptr(),
                    );

                    if res == vk::Result::SUCCESS {
                        log::trace!("Surface created.");
                        Self {
                            handle: handle.assume_init(),
                            surface_fn,
                            win32_surface_fn,
                            instance: instance.clone(),
                        }
                    } else {
                        panic!("Failed to create surface object: {:?}", res)
                    }
                }
            }
            _ => panic!("not supported"),
        }
    }
}

impl Drop for SurfaceKhr {
    fn drop(&mut self) {
        unsafe {
            (self.surface_fn.destroy_surface_khr)(
                self.instance.inner.handle.handle(),
                self.handle,
                std::ptr::null(),
            )
        };
        log::trace!("Surface destroyed.");
    }
}

#[derive(Debug, Clone)]
pub struct ExtensionProperties {
    extension_name: String,
    spec_version: u32,
}

impl From<vk::ExtensionProperties> for ExtensionProperties {
    fn from(props: vk::ExtensionProperties) -> Self {
        Self {
            // SAFETY: According to the Vulkan Specification, `extension_name` is
            // null-terminated UTF-8 string.
            extension_name: unsafe {
                String::from_utf8_unchecked(
                    CStr::from_ptr(props.extension_name.as_ptr())
                        .to_bytes()
                        .into(),
                )
            },
            spec_version: props.spec_version,
        }
    }
}

pub struct SwapchainKhr {
    handle: vk::SwapchainKHR,
    fp: vk::KhrSwapchainFn,
    device: Device,
}

impl SwapchainKhr {
    pub fn new(device: &Device, surface: &SurfaceKhr) -> Self {
        let fp = vk::KhrSwapchainFn::load(|name| unsafe {
            std::mem::transmute(
                device
                    .instance
                    .inner
                    .handle
                    .get_device_proc_addr(device.inner.handle.handle(), name.as_ptr()),
            )
        });

        let create_info = vk::SwapchainCreateInfoKHR {
            ..Default::default()
        };

        let mut handle = MaybeUninit::uninit();
        let res = unsafe {
            (fp.create_swapchain_khr)(
                device.inner.handle.handle(),
                &create_info,
                std::ptr::null(),
                handle.as_mut_ptr(),
            )
        };

        if res == vk::Result::SUCCESS {
            Self {
                handle: unsafe { handle.assume_init() },
                fp,
                device: device.clone(),
            }
        } else {
            panic!("Failed to create swapchain: {}", res)
        }
    }
}

impl Drop for SwapchainKhr {
    fn drop(&mut self) {
        unsafe {
            (self.fp.destroy_swapchain_khr)(
                self.device.inner.handle.handle(),
                self.handle,
                std::ptr::null(),
            );
        }
    }
}

pub struct SurfaceTransformKhr {
    identity: bool,
    rotate_90: bool,
    rotate_180: bool,
    rotate_270: bool,
    horizontal_mirror: bool,
    horizontal_mirror_rotate_90: bool,
    horizontal_mirror_rotate_180: bool,
    horizontal_mirror_rotate_270: bool,
    inherit: bool,
}

impl SurfaceTransformKhr {
    pub fn identity(&self) -> bool {
        self.identity
    }

    pub fn rotate_90(&self) -> bool {
        self.rotate_90
    }

    pub fn rotate_180(&self) -> bool {
        self.rotate_180
    }

    pub fn rotate_270(&self) -> bool {
        self.rotate_270
    }

    pub fn horizontal_mirror(&self) -> bool {
        self.horizontal_mirror
    }

    pub fn horizontal_mirror_rotate_90(&self) -> bool {
        self.horizontal_mirror_rotate_90
    }

    pub fn horizontal_mirror_rotate_180(&self) -> bool {
        self.horizontal_mirror_rotate_180
    }

    pub fn horizontal_mirror_rotate_270(&self) -> bool {
        self.horizontal_mirror_rotate_270
    }

    pub fn inherit(&self) -> bool {
        self.inherit
    }
}
