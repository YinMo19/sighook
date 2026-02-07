# instrument_with_original

Demonstrates `sighook::instrument`.

The callback updates registers before original opcode executes:

- `aarch64`: set `x8 = 40`, `x9 = 2`
- `linux x86_64`: set `rdx = 40`, `rax = 2`

Then original add instruction runs through trampoline.

## Run (from repository root)

macOS:

```bash
cc -O0 -fno-inline examples/instrument_with_original/target.c -o examples/instrument_with_original/app
cargo build --example instrument_with_original
DYLD_INSERT_LIBRARIES="$PWD/target/debug/examples/libinstrument_with_original.dylib" examples/instrument_with_original/app
```

Linux:

```bash
cc -O0 -fno-inline -rdynamic examples/instrument_with_original/target.c -o examples/instrument_with_original/app
cargo build --example instrument_with_original
LD_PRELOAD="$PWD/target/debug/examples/libinstrument_with_original.so" examples/instrument_with_original/app
```

Expected output:

```text
calc(1, 2) = 42
```
