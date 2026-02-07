use crate::context::remap_ctx;
use crate::error::SigHookError;
use crate::memory::{is_brk, last_errno, read_u32};
use crate::state;
use libc::{c_int, c_void};
use std::mem::zeroed;
use std::ptr::null_mut;

extern "C" fn trap_handler(_sig: c_int, info: *mut libc::siginfo_t, uctx: *mut c_void) {
    if info.is_null() || uctx.is_null() {
        unsafe { libc::_exit(1) }
    }

    let uc = unsafe { &mut *(uctx as *mut libc::ucontext_t) };
    if uc.uc_mcontext.is_null() {
        unsafe { libc::_exit(1) }
    }

    let raw_ss = unsafe { &mut (*uc.uc_mcontext).__ss };
    let ctx_ptr = unsafe { remap_ctx(raw_ss as *mut libc::__darwin_arm_thread_state64) };
    let ctx = unsafe { &mut *ctx_ptr };

    let address = ctx.pc;
    let slot = unsafe { state::slot_by_address(address) };
    let slot = match slot {
        Some(slot) => slot,
        None => unsafe { libc::_exit(1) },
    };

    let callback = match slot.callback {
        Some(cb) => cb,
        None => unsafe { libc::_exit(1) },
    };

    let opcode = read_u32(address);
    if !is_brk(opcode) {
        unsafe { libc::_exit(1) }
    }

    let original_pc = ctx.pc;
    callback(address, ctx_ptr);

    if ctx.pc != original_pc {
        return;
    }

    if slot.execute_original {
        if slot.trampoline_pc == 0 {
            unsafe { libc::_exit(1) }
        }
        ctx.pc = slot.trampoline_pc;
    } else {
        ctx.pc = ctx.pc.wrapping_add(4);
    }
}

fn install_signal(signum: c_int) -> Result<(), SigHookError> {
    unsafe {
        let mut act: libc::sigaction = zeroed();
        act.sa_flags = libc::SA_SIGINFO;
        act.sa_sigaction = trap_handler as usize;

        if libc::sigemptyset(&mut act.sa_mask) != 0 {
            return Err(SigHookError::SigEmptySetFailed {
                signum,
                errno: last_errno(),
            });
        }

        if libc::sigaction(signum, &act, null_mut()) != 0 {
            return Err(SigHookError::SigActionFailed {
                signum,
                errno: last_errno(),
            });
        }
    }

    Ok(())
}

pub(crate) unsafe fn ensure_handlers_installed() -> Result<(), SigHookError> {
    if unsafe { state::HANDLERS_INSTALLED } {
        return Ok(());
    }

    install_signal(libc::SIGTRAP)?;
    install_signal(libc::SIGILL)?;

    unsafe {
        state::HANDLERS_INSTALLED = true;
    }
    Ok(())
}
