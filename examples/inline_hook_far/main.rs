use sighook::inline_hook;

extern "C" fn replacement(a: i32, b: i32) -> i32 {
    a * b
}

#[used]
#[cfg_attr(
    any(target_os = "macos", target_os = "ios"),
    unsafe(link_section = "__DATA,__mod_init_func")
)]
#[cfg_attr(
    any(target_os = "linux", target_os = "android"),
    unsafe(link_section = ".init_array")
)]
static INIT_ARRAY: extern "C" fn() = init;

extern "C" fn init() {
    unsafe {
        let symbol = libc::dlsym(libc::RTLD_DEFAULT, c"target_add".as_ptr());
        if symbol.is_null() {
            return;
        }

        let function_entry = symbol as u64;
        let replacement_fn = replacement as *const () as usize as u64;
        let _ = inline_hook(function_entry, replacement_fn);
    }
}
