# Examples

Each subdirectory demonstrates one `sighook` API as a Cargo example target.

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
