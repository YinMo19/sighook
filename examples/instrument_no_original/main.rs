use sighook::{HookContext, instrument_no_original};

#[cfg(not(target_os = "linux"))]
const ADD_INSN_OFFSET: u64 = 0x14;

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const X86_PATCHPOINT_OFFSET: u64 = 0x4;

extern "C" fn replace_logic(_address: u64, ctx: *mut HookContext) {
    unsafe {
        #[cfg(target_arch = "aarch64")]
        {
            (*ctx).regs.named.x0 = 99;
        }

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            (*ctx).rax = 99;
        }
    }
}

#[used]
#[cfg_attr(target_os = "macos", unsafe(link_section = "__DATA,__mod_init_func"))]
#[cfg_attr(target_os = "linux", unsafe(link_section = ".init_array"))]
static INIT_ARRAY: extern "C" fn() = init;

extern "C" fn init() {
    unsafe {
        let target_address = {
            #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
            {
                let symbol = libc::dlsym(libc::RTLD_DEFAULT, c"calc_add_insn".as_ptr());
                if symbol.is_null() {
                    return;
                }
                symbol as u64
            }

            #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
            {
                let symbol = libc::dlsym(libc::RTLD_DEFAULT, c"calc".as_ptr());
                if symbol.is_null() {
                    return;
                }
                symbol as u64 + X86_PATCHPOINT_OFFSET
            }

            #[cfg(not(target_os = "linux"))]
            {
                let symbol = libc::dlsym(libc::RTLD_DEFAULT, c"calc".as_ptr());
                if symbol.is_null() {
                    return;
                }
                symbol as u64 + ADD_INSN_OFFSET
            }
        };

        let _ = instrument_no_original(target_address, replace_logic);
    }
}
