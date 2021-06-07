pub mod instance {
    #[derive(Debug, Copy, Clone, PartialEq, strum_macros::AsRefStr, strum_macros::EnumString)]
    pub enum Layer {
        #[strum(serialize = "VK_LAYER_KHRONOS_validation")]
        KhronosValidation,
        #[strum(serialize = "VK_LAYER_LUNARG_monitor")]
        LunargMonitor,
        #[strum(serialize = "VK_LAYER_LUNARG_gfxreconstruct")]
        LunargGfxreconstruct,
    }

    #[derive(Debug, Copy, Clone, PartialEq, strum_macros::AsRefStr, strum_macros::EnumString)]
    pub enum Extension {
        #[strum(serialize = "VK_EXT_debug_utils")]
        ExtDebugUtils,
        #[strum(serialize = "VK_KHR_win32_surface")]
        KhrWin32Surface,
        #[strum(serialize = "VK_KHR_surface")]
        KhrSurface,
        #[strum(serialize = "VK_KHR_xlib_surface")]
        KhrXlibSurface,
        #[strum(serialize = "VK_KHR_xcb_surface")]
        KhrXcbSurface,
        #[strum(serialize = "VK_KHR_display")]
        KhrDisplay,
    }
}

pub mod device {
    mod layer {}

    #[derive(Debug, Copy, Clone, PartialEq, strum_macros::AsRefStr, strum_macros::EnumString)]
    pub enum Extension {
        #[strum(serialize = "VK_KHR_swapchain")]
        KhrSwapchain,
        #[strum(serialize = "VK_KHR_deferred_host_operations")]
        KhrDeferredHostOperations,
        #[strum(serialize = "VK_KHR_ray_tracing_pipeline")]
        KhrRayTracingPipeline,
        #[strum(serialize = "VK_KHR_acceleration_structure")]
        KhrAccelerationStructure,
        #[strum(serialize = "VK_KHR_shader_non_semantic_info")]
        KhrShaderNonSemanticInfo,
        #[strum(serialize = "VK_KHR_ray_query")]
        KhrRayQuery,
        #[strum(serialize = "VK_KHR_synchronization2")]
        KhrSynchronization2,
        #[strum(serialize = "VK_KHR_vulkan_memory_model")]
        KhrVulkanMemoryModel,
    }
}

#[test]
fn name_str_test() {
    let a = instance::Layer::KhronosValidation;
    dbg!(a.as_ref());
    dbg!(a);
}
