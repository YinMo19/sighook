# sighook

`Sighook` is a macOS (`aarch64`) runtime patching crate focused on:

- instruction-level instrumentation via `BRK + signal handler`
- function-entry inline detours (near and far jump)

It is designed for low-level experimentation, reverse engineering, and custom runtime instrumentation workflows.

## Features

- `patchcode(address, opcode)` for raw 32-bit instruction patching
- `instrument(address, callback)` to trap and then execute original opcode
- `instrument_no_original(address, callback)` to trap and skip original opcode
- `inline_hook(addr, replace_fn)` with automatic far-jump fallback
- zero-copy context remap (`HookContext`) in callbacks
- register union access: `ctx.regs.x[i]` and `ctx.regs.named.xN`

## Platform

- macOS on Apple Silicon (`aarch64`)
- single-thread model (`static mut` internal state)

## Installation

```toml
[dependencies]
sighook = "0.1"
```

## Quick Start

### 1) BRK instrumentation (execute original opcode)

```rust,no_run
use sighook::{instrument, HookContext};

extern "C" fn on_hit(_address: u64, ctx: *mut HookContext) {
    unsafe {
        // Example: touch a register before original opcode executes.
        (*ctx).regs.named.x0 = (*ctx).regs.named.x0.wrapping_add(1);
    }
}

let target_instruction = 0x1000_0000_u64;
let _original = instrument(target_instruction, on_hit)?;
# Ok::<(), sighook::SigHookError>(())
```

### 2) BRK instrumentation (do not execute original opcode)

```rust,no_run
use sighook::{instrument_no_original, HookContext};

extern "C" fn replace_logic(_address: u64, ctx: *mut HookContext) {
    unsafe {
        // Example: fully replace behavior by editing result register directly.
        (*ctx).regs.named.x0 = 0x1234;
    }
}

let target_instruction = 0x1000_0010_u64;
let _original = instrument_no_original(target_instruction, replace_logic)?;
# Ok::<(), sighook::SigHookError>(())
```

### 3) Inline function hook

```rust,no_run
use sighook::inline_hook;

extern "C" fn replacement() {}

let function_entry = 0x1000_1000_u64;
let replacement_addr = replacement as usize as u64;
let _original = inline_hook(function_entry, replacement_addr)?;
# Ok::<(), sighook::SigHookError>(())
```

## API Notes

- `instrument(...)` executes original opcode through an internal trampoline.
- `instrument_no_original(...)` skips original opcode unless callback changes `ctx.pc`.
- `inline_hook(...)` first tries direct `b`; if out of range, it patches a far-jump stub.
- `inline_hook(...)` uses `b` (not `bl`), so replacement returns to original caller via `lr`.

## Safety Notes

This crate performs runtime code patching and raw context mutation.

- Ensure target addresses are valid runtime addresses.
- Ensure callback logic preserves ABI expectations.
- Test on disposable binaries first.

## License

GPL-2.0-only
