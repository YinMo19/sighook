# instrument_with_original

Demonstrates `sighook::instrument`.

The callback updates registers before original opcode executes:

- set `x8 = 40`
- set `x9 = 2`

Then original `add w0, w8, w9` runs through trampoline.

## Run (from repository root)

```bash
cc -O0 -fno-inline examples/instrument_with_original/target.c -o examples/instrument_with_original/app
cargo build --example instrument_with_original
DYLD_INSERT_LIBRARIES="$PWD/target/debug/examples/libinstrument_with_original.dylib" examples/instrument_with_original/app
```

Expected output:

```text
calc(1, 2) = 42
```
