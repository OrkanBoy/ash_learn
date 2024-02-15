use std::{ffi::{c_char, c_void, CStr, CString}, mem};

use ash::{extensions::{ext::DebugUtils, khr::{Surface, Swapchain, Win32Surface}}, vk::{self, DebugUtilsMessengerEXT, SurfaceKHR, HINSTANCE, HWND}};
use ash::extensions::*;
use winit::raw_window_handle::{DisplayHandle, HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle, WindowHandle};
const EXTENSION_NAMES: &[*const c_char] = &[
    khr::Surface::name().as_ptr(), 
    khr::Win32Surface::name().as_ptr(),
    DebugUtils::name().as_ptr()
];

const LAYER_NAMES: &[*const c_char] = &["VK_LAYER_KHRONOS_validation\0".as_ptr() as *const c_char];

pub fn create_instance(entry: &ash::Entry) -> ash::Instance {
    let app_name = CString::new("Vulkan Application").unwrap();
    let engine_name = CString::new("No Engine").unwrap();

    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .engine_name(&engine_name)
        .application_version(vk::make_api_version(0, 0, 0, 1))
        .engine_version(vk::make_api_version(0, 0, 0, 1))
        .api_version(vk::make_api_version(0, 1, 3, 0));


    let info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&EXTENSION_NAMES)
        .enabled_layer_names(&LAYER_NAMES);

    unsafe { entry.create_instance(&info, None).unwrap() }
}

pub fn create_messenger(debug_utils: &DebugUtils) -> DebugUtilsMessengerEXT {
    let create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        )
        .pfn_user_callback(Some(vulkan_debug_callback));

    unsafe {
        debug_utils
            .create_debug_utils_messenger(&create_info, None)
            .unwrap()
    }
}

unsafe extern "system" fn vulkan_debug_callback(
    flag: vk::DebugUtilsMessageSeverityFlagsEXT,
    typ: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> vk::Bool32 {
    type Flag = vk::DebugUtilsMessageSeverityFlagsEXT;

    let msg = format!(
        "(Validation Layer): {:?} - {:?}",
        typ,
        CStr::from_ptr((*p_callback_data).p_message)
    );
    match flag {
        Flag::VERBOSE => log::debug!("{msg}"),
        Flag::INFO => log::info!("{msg}"),
        Flag::WARNING => log::warn!("{msg}"),
        _ => log::error!("{msg}"),
    }
    vk::FALSE
}


pub unsafe fn create_surface(entry: &ash::Entry, instance: &ash::Instance, window: &winit::window::Window) -> SurfaceKHR {
    let display_handle = window.display_handle().unwrap().as_raw();
    let window_handle = window.window_handle().unwrap().as_raw();

    match (display_handle, window_handle) {
        (RawDisplayHandle::Windows(_), RawWindowHandle::Win32(window)) => {
            let surface_desc: vk::Win32SurfaceCreateInfoKHRBuilder<'_> = vk::Win32SurfaceCreateInfoKHR::builder()
                .hinstance(window.hinstance.unwrap().get() as HINSTANCE)
                .hwnd(window.hwnd.get() as HWND);
            let surface_fn = khr::Win32Surface::new(entry, instance);
            surface_fn.create_win32_surface(&surface_desc, None).unwrap()
        }
        _ => unimplemented!()
    }
}