#![cfg(target_os = "macos")]
#![cfg(target_arch = "aarch64")]
#![allow(deprecated)]
#![doc = include_str!("../README.md")]

mod constants;
mod context;
mod error;
mod memory;
mod signal;
mod state;
mod trampoline;

pub use context::{HookContext, InstrumentCallback, XRegisters, XRegistersNamed};
pub use error::SigHookError;

/// Patches one 32-bit instruction at `address`.
///
/// # Parameters
/// - `address`: Runtime virtual address of the instruction.
/// - `new_opcode`: AArch64 instruction word to write.
///
/// # Returns
/// Returns the original opcode.
///
/// # Examples
/// ```rust,no_run
/// use sighook::patchcode;
///
/// let address = 0x1000_0000_u64;
/// let brk = 0xD420_0000_u32;
/// let _original = patchcode(address, brk)?;
/// # Ok::<(), sighook::SigHookError>(())
/// ```
pub fn patchcode(address: u64, new_opcode: u32) -> Result<u32, SigHookError> {
    memory::patch_u32(address, new_opcode)
}

/// Installs BRK-based instrumentation that executes the original instruction.
///
/// The target instruction is replaced with `brk #0`. On trap, callback receives
/// `(address, ctx)`, then the library resumes via an internal trampoline that
/// runs the original opcode and jumps back to `address + 4`.
///
/// # Parameters
/// - `address`: Runtime virtual address of the target instruction.
/// - `callback`: User callback for register/context edits.
///
/// # Returns
/// Returns the original opcode at `address`.
///
/// # Examples
/// ```rust,no_run
/// use sighook::{instrument, HookContext};
///
/// extern "C" fn on_hit(_address: u64, ctx: *mut HookContext) {
///     unsafe {
///         (*ctx).pc = (*ctx).pc.wrapping_add(0);
///     }
/// }
///
/// let address = 0x1000_0000_u64;
/// let _original = instrument(address, on_hit)?;
/// # Ok::<(), sighook::SigHookError>(())
/// ```
pub fn instrument(address: u64, callback: InstrumentCallback) -> Result<u32, SigHookError> {
    instrument_internal(address, callback, true)
}

/// Installs BRK-based instrumentation that does not execute original opcode.
///
/// The target instruction is replaced with `brk #0`. On trap, callback receives
/// `(address, ctx)`. If callback keeps `ctx.pc` unchanged, the library advances
/// to `address + 4` directly.
///
/// # Parameters
/// - `address`: Runtime virtual address of the target instruction.
/// - `callback`: User callback for custom replacement logic.
///
/// # Returns
/// Returns the original opcode at `address`.
///
/// # Examples
/// ```rust,no_run
/// use sighook::{instrument_no_original, HookContext};
///
/// extern "C" fn replace_logic(_address: u64, _ctx: *mut HookContext) {}
///
/// let address = 0x1000_0010_u64;
/// let _original = instrument_no_original(address, replace_logic)?;
/// # Ok::<(), sighook::SigHookError>(())
/// ```
pub fn instrument_no_original(
    address: u64,
    callback: InstrumentCallback,
) -> Result<u32, SigHookError> {
    instrument_internal(address, callback, false)
}

fn instrument_internal(
    address: u64,
    callback: InstrumentCallback,
    execute_original: bool,
) -> Result<u32, SigHookError> {
    unsafe {
        if let Some(original) = state::original_opcode_by_address(address) {
            state::register_slot(address, original, callback, execute_original)?;
            return Ok(original);
        }

        signal::ensure_handlers_installed()?;

        let original = patchcode(address, constants::BRK_OPCODE)?;
        state::register_slot(address, original, callback, execute_original)?;
        Ok(original)
    }
}

/// Installs an inline function-entry hook.
///
/// The library first tries a direct `b replace_fn`. If target is out of direct
/// branch range, it falls back to a far jump stub at function entry:
/// `ldr x16, #8; br x16; .quad replace_fn`.
///
/// The detour uses `b` (not `bl`), so `replace_fn` returns to the original
/// caller using the caller-provided `lr`.
///
/// # Parameters
/// - `addr`: Runtime virtual address of function entry.
/// - `replace_fn`: Runtime virtual address of replacement function.
///
/// # Returns
/// Returns the original first opcode at `addr`.
///
/// # Examples
/// ```rust,no_run
/// use sighook::inline_hook;
///
/// extern "C" fn replacement() {}
///
/// let target = 0x1000_1000_u64;
/// let detour = replacement as usize as u64;
/// let _original = inline_hook(target, detour)?;
/// # Ok::<(), sighook::SigHookError>(())
/// ```
pub fn inline_hook(addr: u64, replace_fn: u64) -> Result<u32, SigHookError> {
    match memory::encode_b(addr, replace_fn) {
        Ok(b_opcode) => patchcode(addr, b_opcode),
        Err(SigHookError::BranchOutOfRange) => memory::patch_far_jump(addr, replace_fn),
        Err(err) => Err(err),
    }
}

/// Returns saved original opcode for an instrumented address.
///
/// Returns `None` if the address is not registered.
///
/// # Examples
/// ```rust,no_run
/// use sighook::original_opcode;
///
/// let address = 0x1000_0000_u64;
/// let _maybe_opcode = original_opcode(address);
/// ```
pub fn original_opcode(address: u64) -> Option<u32> {
    unsafe { state::original_opcode_by_address(address) }
}
