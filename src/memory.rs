use crate::constants::{BR_X16, BRK_MASK, BRK_OPCODE, LDR_X16_LITERAL_8, VM_PROT_COPY};
use crate::error::SigHookError;
use libc::{c_int, c_void};

unsafe extern "C" {
    fn sys_icache_invalidate(start: *mut c_void, len: usize);
    fn mach_vm_protect(
        target_task: libc::vm_map_t,
        address: libc::mach_vm_address_t,
        size: libc::mach_vm_size_t,
        set_maximum: libc::boolean_t,
        new_protection: libc::vm_prot_t,
    ) -> libc::kern_return_t;
}

#[inline]
pub(crate) fn last_errno() -> c_int {
    unsafe { *libc::__error() }
}

#[inline]
pub(crate) fn is_brk(opcode: u32) -> bool {
    (opcode & BRK_MASK) == (BRK_OPCODE & BRK_MASK)
}

#[inline]
pub(crate) fn read_u32(address: u64) -> u32 {
    unsafe { u32::from_le(std::ptr::read_volatile(address as *const u32)) }
}

fn page_size() -> Result<usize, SigHookError> {
    let value = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    if value <= 0 {
        return Err(SigHookError::PageSizeUnavailable);
    }
    Ok(value as usize)
}

fn protect_range_start_len(address: usize, len: usize, page_size: usize) -> (usize, usize) {
    let start = address & !(page_size - 1);
    let end_inclusive = address + len - 1;
    let end_page = end_inclusive & !(page_size - 1);
    let total = (end_page + page_size) - start;
    (start, total)
}

fn patch_bytes(address: u64, bytes: &[u8]) -> Result<Vec<u8>, SigHookError> {
    if address == 0 || (address & 0b11) != 0 || bytes.is_empty() {
        return Err(SigHookError::InvalidAddress);
    }

    let page_size = page_size()?;
    let addr = address as usize;
    let (protect_start, protect_len) = protect_range_start_len(addr, bytes.len(), page_size);

    let kr = unsafe {
        mach_vm_protect(
            libc::mach_task_self(),
            protect_start as u64,
            protect_len as u64,
            0,
            libc::VM_PROT_READ | libc::VM_PROT_WRITE | VM_PROT_COPY,
        )
    };
    if kr != 0 {
        return Err(SigHookError::ProtectWritableFailed {
            kr,
            errno: last_errno(),
        });
    }

    let mut original = vec![0u8; bytes.len()];
    unsafe {
        std::ptr::copy_nonoverlapping(addr as *const u8, original.as_mut_ptr(), bytes.len());
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), addr as *mut u8, bytes.len());
        sys_icache_invalidate(addr as *mut c_void, bytes.len());
    }

    let kr_restore = unsafe {
        mach_vm_protect(
            libc::mach_task_self(),
            protect_start as u64,
            protect_len as u64,
            0,
            libc::VM_PROT_READ | libc::VM_PROT_EXECUTE,
        )
    };
    if kr_restore != 0 {
        return Err(SigHookError::ProtectExecutableFailed {
            kr: kr_restore,
            errno: last_errno(),
        });
    }

    Ok(original)
}

pub(crate) fn patch_u32(address: u64, new_opcode: u32) -> Result<u32, SigHookError> {
    let original = patch_bytes(address, &new_opcode.to_le_bytes())?;
    let mut opcode_bytes = [0u8; 4];
    opcode_bytes.copy_from_slice(&original[0..4]);
    Ok(u32::from_le_bytes(opcode_bytes))
}

/// Encodes an AArch64 `b` instruction from `from_address` to `to_address`.
///
/// The branch immediate is 26-bit signed (word-aligned), giving a ±128MB range.
pub(crate) fn encode_b(from_address: u64, to_address: u64) -> Result<u32, SigHookError> {
    if (from_address & 0b11) != 0 || (to_address & 0b11) != 0 {
        return Err(SigHookError::InvalidAddress);
    }

    let offset = (to_address as i128) - (from_address as i128);
    if (offset & 0b11) != 0 {
        return Err(SigHookError::BranchOutOfRange);
    }

    let imm26 = offset >> 2;
    let min = -(1_i128 << 25);
    let max = (1_i128 << 25) - 1;
    if imm26 < min || imm26 > max {
        return Err(SigHookError::BranchOutOfRange);
    }

    let imm26_bits = (imm26 as i64 as u32) & 0x03FF_FFFF;
    Ok(0x1400_0000 | imm26_bits)
}

pub(crate) fn patch_far_jump(from_address: u64, to_address: u64) -> Result<u32, SigHookError> {
    if (from_address & 0b11) != 0 {
        return Err(SigHookError::InvalidAddress);
    }

    let mut bytes = [0u8; 16];
    bytes[0..4].copy_from_slice(&LDR_X16_LITERAL_8.to_le_bytes());
    bytes[4..8].copy_from_slice(&BR_X16.to_le_bytes());
    bytes[8..16].copy_from_slice(&to_address.to_le_bytes());

    let original = patch_bytes(from_address, &bytes)?;
    let mut opcode_bytes = [0u8; 4];
    opcode_bytes.copy_from_slice(&original[0..4]);
    Ok(u32::from_le_bytes(opcode_bytes))
}
