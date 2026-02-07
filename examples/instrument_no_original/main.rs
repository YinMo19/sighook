use sighook::{HookContext, instrument_no_original};

const ADD_INSN_OFFSET: u64 = 0x14;

extern "C" fn replace_logic(_address: u64, ctx: *mut HookContext) {
    unsafe {
        (*ctx).regs.named.x0 = 99;
    }
}

#[used]
#[cfg_attr(target_os = "macos", unsafe(link_section = "__DATA,__mod_init_func"))]
static INIT_ARRAY: extern "C" fn() = init;

extern "C" fn init() {
    unsafe {
        let symbol = libc::dlsym(libc::RTLD_DEFAULT, c"calc".as_ptr());
        if symbol.is_null() {
            return;
        }

        let target_address = symbol as u64 + ADD_INSN_OFFSET;
        let _ = instrument_no_original(target_address, replace_logic);
    }
}
