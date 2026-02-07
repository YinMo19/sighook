use sighook::patchcode;

#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
const ADD_INSN_OFFSET: u64 = 0x14;
const MUL_W0_W8_W9: u32 = 0x1B09_7D00;

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

            #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
            {
                let symbol = libc::dlsym(libc::RTLD_DEFAULT, c"calc".as_ptr());
                if symbol.is_null() {
                    return;
                }
                symbol as u64 + ADD_INSN_OFFSET
            }
        };

        let _ = patchcode(target_address, MUL_W0_W8_W9);
    }
}
