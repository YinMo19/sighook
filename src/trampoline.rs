use crate::constants::{BR_X16, LDR_X16_LITERAL_8};
use crate::error::SigHookError;
use crate::memory::last_errno;
use libc::c_void;
use std::ptr::null_mut;

unsafe extern "C" {
    fn sys_icache_invalidate(start: *mut c_void, len: usize);
}

pub(crate) fn create_original_trampoline(
    next_pc: u64,
    original_opcode: u32,
) -> Result<u64, SigHookError> {
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    if page_size <= 0 {
        return Err(SigHookError::PageSizeUnavailable);
    }

    let page_size = page_size as usize;

    let memory = unsafe {
        libc::mmap(
            null_mut(),
            page_size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANON,
            -1,
            0,
        )
    };

    if memory == libc::MAP_FAILED {
        return Err(SigHookError::MmapFailed {
            errno: last_errno(),
        });
    }

    let base = memory as usize;

    unsafe {
        #[allow(clippy::identity_op)]
        std::ptr::write_unaligned((base + 0) as *mut u32, original_opcode.to_le());
        std::ptr::write_unaligned((base + 4) as *mut u32, LDR_X16_LITERAL_8.to_le());
        std::ptr::write_unaligned((base + 8) as *mut u32, BR_X16.to_le());
        std::ptr::write_unaligned((base + 12) as *mut u64, next_pc.to_le());

        sys_icache_invalidate(memory, 20);

        if libc::mprotect(memory, page_size, libc::PROT_READ | libc::PROT_EXEC) != 0 {
            return Err(SigHookError::TrampolineProtectFailed {
                errno: last_errno(),
            });
        }
    }

    Ok(base as u64)
}
