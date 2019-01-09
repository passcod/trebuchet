/// Assumes true since very few platforms won't.
///
/// TODO: Actually check. (How?!)
pub fn has_opengl() -> bool {
    true
}

/// Checks OpenCL availability by listing the system's OpenCL platforms.
///
/// Only checks once per running instance.
#[cfg(feature = "gpu")]
pub fn has_opencl() -> bool {
    lazy_static! {
        static ref OCL_AVAILABLE: Option<bool> = ocl_core::get_platform_ids()
        .ok()
        .map(|list| !list.is_empty());
    }

    OCL_AVAILABLE.unwrap_or(false)
}

#[cfg(not(feature = "gpu"))]
pub fn has_opencl() -> bool {
    false
}
