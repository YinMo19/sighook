#![allow(deprecated)]
#![doc = include_str!("../README.md")]

#[cfg(not(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(target_os = "linux", target_arch = "aarch64"),
    all(target_os = "linux", target_arch = "x86_64")
)))]
compile_error!("sighook only supports macOS aarch64, Linux aarch64, and Linux x86_64.");

mod constants;
mod context;
mod error;
mod memory;
mod signal;
mod state;
mod trampoline;

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub use context::{HookContext, InstrumentCallback};
#[cfg(target_arch = "aarch64")]
pub use context::{HookContext, InstrumentCallback, XRegisters, XRegistersNamed};
pub use error::SigHookError;

#[cfg(target_arch = "aarch64")]
pub fn patchcode(address: u64, new_opcode: u32) -> Result<u32, SigHookError> {
    memory::patch_u32(address, new_opcode)
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub fn patchcode(address: u64, new_opcode: u32) -> Result<u32, SigHookError> {
    memory::patch_u32(address, new_opcode)
}

pub fn instrument(address: u64, callback: InstrumentCallback) -> Result<u32, SigHookError> {
    instrument_internal(address, callback, true)
}

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
        if let Some((bytes, len)) = state::original_bytes_by_address(address) {
            state::register_slot(
                address,
                &bytes[..len as usize],
                len,
                callback,
                execute_original,
            )?;
            return state::original_opcode_by_address(address).ok_or(SigHookError::InvalidAddress);
        }

        signal::ensure_handlers_installed()?;

        #[cfg(target_arch = "aarch64")]
        let step_len: u8 = memory::instruction_width();

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        let step_len: u8 = memory::instruction_width_at(address)?;

        #[cfg(target_arch = "aarch64")]
        let original_bytes = {
            let original = memory::patch_u32(address, constants::BRK_OPCODE)?;
            original.to_le_bytes().to_vec()
        };

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        let original_bytes = {
            let original_bytes = memory::read_bytes(address, step_len as usize)?;
            let _ = memory::patch_u8(address, memory::int3_opcode())?;
            original_bytes
        };

        state::register_slot(
            address,
            &original_bytes,
            step_len,
            callback,
            execute_original,
        )?;
        state::original_opcode_by_address(address).ok_or(SigHookError::InvalidAddress)
    }
}

pub fn inline_hook(addr: u64, replace_fn: u64) -> Result<u32, SigHookError> {
    #[cfg(target_arch = "aarch64")]
    {
        match memory::encode_b(addr, replace_fn) {
            Ok(b_opcode) => patchcode(addr, b_opcode),
            Err(SigHookError::BranchOutOfRange) => memory::patch_far_jump(addr, replace_fn),
            Err(err) => Err(err),
        }
    }

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        if let Ok(jmp) = memory::encode_jmp_rel32(addr, replace_fn) {
            let original = memory::patch_bytes_public(addr, &jmp)?;
            let mut opcode = [0u8; 4];
            if original.len() >= 4 {
                opcode.copy_from_slice(&original[..4]);
                return Ok(u32::from_le_bytes(opcode));
            }
            return Err(SigHookError::InvalidAddress);
        }

        let abs = memory::encode_absolute_jump(replace_fn);
        let original = memory::patch_bytes_public(addr, &abs)?;
        let mut opcode = [0u8; 4];
        if original.len() >= 4 {
            opcode.copy_from_slice(&original[..4]);
            return Ok(u32::from_le_bytes(opcode));
        }
        Err(SigHookError::InvalidAddress)
    }
}

pub fn original_opcode(address: u64) -> Option<u32> {
    unsafe { state::original_opcode_by_address(address) }
}
