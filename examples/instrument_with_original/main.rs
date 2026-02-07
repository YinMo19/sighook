use sighook::{HookContext, instrument};

const ADD_INSN_OFFSET: u64 = 0x14;

extern "C" fn on_hit(_address: u64, ctx: *mut HookContext) {
    unsafe {
        #[cfg(target_arch = "aarch64")]
        {
            (*ctx).regs.named.x8 = 40;
            (*ctx).regs.named.x9 = 2;
        }

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            (*ctx).rdx = 40;
            (*ctx).rax = 2;
        }
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
        let _ = instrument(target_address, on_hit);
    }
}
