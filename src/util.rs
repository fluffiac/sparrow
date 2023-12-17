// useful for tokio bounds which require static
pub unsafe fn as_static<T>(t: &T) -> &'static T {
    std::mem::transmute(t)
}

// useful for tokio bounds which require static
pub unsafe fn as_static_mut<T>(t: &mut T) -> &'static mut T {
    std::mem::transmute(t)
}
