# Examples

Each subdirectory demonstrates one `sighook` API as a Cargo example target.

Current supported host targets:

- `aarch64-apple-darwin`
- `aarch64-unknown-linux-gnu`
- `x86_64-unknown-linux-gnu`

Build all examples from repository root:

```bash
cargo build --examples
```

Artifacts are generated under the root `target/debug/examples/` directory.

Available examples:

- `patchcode_add_to_mul`: patch one opcode directly
- `instrument_with_original`: BRK instrumentation + execute original opcode
- `instrument_no_original`: BRK instrumentation + skip original opcode
- `inline_hook_far`: function-entry detour with inline hook

## Coverage matrix

- `aarch64-apple-darwin`: `patchcode` / `instrument` / `instrument_no_original` / `inline_hook`
- `aarch64-unknown-linux-gnu`: runtime smoke coverage for all 4 examples (CI)
- `x86_64-unknown-linux-gnu`: API compile coverage for all examples, runtime smoke coverage for `instrument*` and `inline_hook`; `patchcode_add_to_mul` remains AArch64-opcode specific demo

## Notes by architecture

- On `aarch64`, `calc` examples use fixed asm layout so `ADD_INSN_OFFSET=0x14` and opcode patching remain stable across macOS/Linux CI.
- On `x86_64`, examples compile for API smoke-check; runtime offsets/opcodes are architecture-specific and need per-binary recalculation before real testing.
