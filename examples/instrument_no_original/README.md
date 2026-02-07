# instrument_no_original

Demonstrates `sighook::instrument_no_original`.

The callback writes `x0 = 99`, and original opcode is skipped.

## Run (from repository root)

```bash
cc -O0 -fno-inline examples/instrument_no_original/target.c -o examples/instrument_no_original/app
cargo build --example instrument_no_original
DYLD_INSERT_LIBRARIES="$PWD/target/debug/examples/libinstrument_no_original.dylib" examples/instrument_no_original/app
```

Expected output:

```text
calc(4, 5) = 99
```
