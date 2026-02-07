# instrument_no_original

Demonstrates `sighook::instrument_no_original`.

The callback overwrites return register and skips original instruction:

- `aarch64`: write `x0 = 99`
- `linux x86_64`: write `rax = 99`

## Run (from repository root)

macOS:

```bash
cc -O0 -fno-inline examples/instrument_no_original/target.c -o examples/instrument_no_original/app
cargo build --example instrument_no_original
DYLD_INSERT_LIBRARIES="$PWD/target/debug/examples/libinstrument_no_original.dylib" examples/instrument_no_original/app
```

Linux:

```bash
cc -O0 -fno-inline -rdynamic examples/instrument_no_original/target.c -o examples/instrument_no_original/app
cargo build --example instrument_no_original
LD_PRELOAD="$PWD/target/debug/examples/libinstrument_no_original.so" examples/instrument_no_original/app
```

Expected output:

```text
calc(4, 5) = 99
```
