# sighook

[![crates.io](https://img.shields.io/crates/v/sighook.svg)](https://crates.io/crates/sighook)
[![docs.rs](https://docs.rs/sighook/badge.svg)](https://docs.rs/sighook)
[![CI](https://github.com/YinMo19/sighook/actions/workflows/ci.yml/badge.svg)](https://github.com/YinMo19/sighook/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/sighook.svg)](https://spdx.org/licenses/GPL-2.0-only.html)

`Sighook` is a runtime patching crate focused on:

- instruction-level instrumentation via trap instruction + signal handler
- function-entry inline detours (near and far jump)

It is designed for low-level experimentation, reverse engineering, and custom runtime instrumentation workflows.

## Features

- `patchcode(address, opcode)` for raw instruction patching
- `instrument(address, callback)` to trap and then execute original opcode
- `instrument_no_original(address, callback)` to trap and skip original opcode
- `inline_hook(addr, replace_fn)` with automatic far-jump fallback
- zero-copy context remap (`HookContext`) in callbacks
- architecture-specific callback context (`aarch64` and `x86_64` layouts)

## Platform Support

- `aarch64-apple-darwin`: full API support (`patchcode` / `instrument` / `instrument_no_original` / `inline_hook`)
- `aarch64-unknown-linux-gnu`: full API support (`patchcode` / `instrument` / `instrument_no_original` / `inline_hook`)
- `x86_64-unknown-linux-gnu`: full API support; CI smoke validates `patchcode` / `instrument` / `instrument_no_original` / `inline_hook` examples
- single-thread model (`static mut` internal state)

## Installation

```toml
[dependencies]
sighook = "0.3.1"
```

## Quick Start

### 1) BRK instrumentation (execute original opcode)

```rust,no_run
use sighook::{instrument, HookContext};

extern "C" fn on_hit(_address: u64, ctx: *mut HookContext) {
    let _ = ctx;
}

let target_instruction = 0x1000_0000_u64;
let _original = instrument(target_instruction, on_hit)?;
# Ok::<(), sighook::SigHookError>(())
```

### 2) BRK instrumentation (do not execute original opcode)

```rust,no_run
use sighook::{instrument_no_original, HookContext};

extern "C" fn replace_logic(_address: u64, ctx: *mut HookContext) {
    let _ = ctx;
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

## Example Loading Model

The examples are `cdylib` payloads that auto-run hook install logic via constructor sections:

- macOS uses `__DATA,__mod_init_func` + `DYLD_INSERT_LIBRARIES`
- Linux uses `.init_array` + `LD_PRELOAD`

When your preload library resolves symbols from the target executable via `dlsym`, compile the target executable with `-rdynamic` on Linux.

## Linux AArch64 Patchpoint Note

For AArch64 Linux examples, `calc`-based demos export a dedicated `calc_add_insn` symbol and patch that symbol directly. This avoids brittle fixed-offset assumptions in toolchain-generated function layout.

## API Notes

- `instrument(...)` executes original instruction through an internal trampoline.
- `instrument_no_original(...)` skips original instruction unless callback changes control-flow register (`pc`/`rip`).
- `inline_hook(...)` uses architecture-specific near jump first, then far-jump fallback.

## Safety Notes

This crate performs runtime code patching and raw context mutation.

- Ensure target addresses are valid runtime addresses.
- Ensure callback logic preserves ABI expectations.
- Test on disposable binaries first.

## License

GPL-2.0-only
