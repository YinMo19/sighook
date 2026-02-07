use sighook::inline_hook;

extern "C" fn replacement(a: i32, b: i32) -> i32 {
    a * b
}

#[used]
#[cfg_attr(target_os = "macos", unsafe(link_section = "__DATA,__mod_init_func"))]
static INIT_ARRAY: extern "C" fn() = init;

extern "C" fn init() {
    unsafe {
        let symbol = libc::dlsym(libc::RTLD_DEFAULT, c"target_add".as_ptr());
        if symbol.is_null() {
            return;
        }

        let function_entry = symbol as u64;
        let replacement_fn = replacement as usize as u64;
        let _ = inline_hook(function_entry, replacement_fn);
    }
}
