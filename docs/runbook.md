# Grammar compiler runbook

## Verify a source checkout

1. Run `cargo build --release --locked` at the repository root.
2. Run `cargo test --locked`.
3. Create the Python environment with the [README commands](../README.md#interface).
4. Activate the environment and run `make check`.
5. Run `make demo`. Expect the letters `n` and `y` after the opening quote.

For a native-only installation, run `cargo install --path . --locked`. This installs the `grammar-decode` executable in the Cargo binary directory.

## Check a CLI request

The CLI reads a schema file and consumes standard input until EOF. The `accepts` operation consumes raw bytes. The `mask` operation expects a JSON object with three fields.

| Mask field | Value |
| --- | --- |
| `prefix` | Array of byte values from 0 to 255. |
| `vocabulary` | Object mapping unsigned 32-bit token IDs to byte arrays. |
| `eos_id` | Reserved unsigned 32-bit end token ID. |

| Exit status | Meaning | Next action |
| --- | --- | --- |
| 0 | Accepted input or successful mask calculation. | Read the Boolean or sorted ID array from standard output. |
| 1 | The input bytes fail the complete-match check. | Inspect the exact bytes, property order, and trailing newline. |
| 2 | The command, schema, file, or mask request is invalid. | Read the diagnostic on standard error. |

## Reproduce benchmark evidence

1. Run `cargo bench --locked --bench masks` for the native mask workload.
2. Activate the Python environment.
3. Run the following command for the Python workloads.

```sh
python -m grammar_decode.benchmark --repeats 7 --iterations 10 --vocab-size 1024 > measurements.json
```

Record the revision, CPU, Rust version, Python version, vocabulary size, and case. Preserve each batch timing. Compare matching cases with identical allowed-token counts. The [operating guide](operations.md#reproduce-a-cpu-benchmark) explains how to interpret the measurements.
