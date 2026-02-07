#[repr(C)]
#[derive(Copy, Clone)]
pub struct XRegistersNamed {
    pub x0: u64,
    pub x1: u64,
    pub x2: u64,
    pub x3: u64,
    pub x4: u64,
    pub x5: u64,
    pub x6: u64,
    pub x7: u64,
    pub x8: u64,
    pub x9: u64,
    pub x10: u64,
    pub x11: u64,
    pub x12: u64,
    pub x13: u64,
    pub x14: u64,
    pub x15: u64,
    pub x16: u64,
    pub x17: u64,
    pub x18: u64,
    pub x19: u64,
    pub x20: u64,
    pub x21: u64,
    pub x22: u64,
    pub x23: u64,
    pub x24: u64,
    pub x25: u64,
    pub x26: u64,
    pub x27: u64,
    pub x28: u64,
    pub x29: u64,
    pub x30: u64,
}

/// AArch64 general-purpose register view.
///
/// Access either as indexed array (`x`) or named fields (`named`).
#[repr(C)]
#[derive(Copy, Clone)]
pub union XRegisters {
    pub x: [u64; 31],
    pub named: XRegistersNamed,
}

/// Zero-copy execution context view used by instrumentation callbacks.
///
/// This layout is intentionally compatible with Darwin
/// `__darwin_arm_thread_state64`.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct HookContext {
    pub regs: XRegisters,
    pub sp: u64,
    pub pc: u64,
    pub cpsr: u32,
    pub pad: u32,
}

/// Callback signature for BRK-based instrumentation.
///
/// - `address`: Instruction address that triggered the trap.
/// - `ctx`: Mutable execution context for emulation/state edits.
pub type InstrumentCallback = extern "C" fn(address: u64, ctx: *mut HookContext);

/// Reinterprets Darwin thread state as `HookContext` without copying.
///
/// # Safety
/// Caller must ensure `thread_state` points to a valid
/// `__darwin_arm_thread_state64` memory block.
pub unsafe fn remap_ctx(thread_state: *mut libc::__darwin_arm_thread_state64) -> *mut HookContext {
    thread_state.cast::<HookContext>()
}
