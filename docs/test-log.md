# Test log

## 2026-11-27 · native compiler and Python binding

Tested source revision `d245d22` on macOS arm64 with Rust 1.95.0 and Python 3.12.12. The Python environment contained the native extension and Pydantic v2.

| Check | Observed result |
| --- | --- |
| `make check PYTHON=.venv/bin/python` | Rust formatting passed. Four Rust integration tests and seven Python tests passed. |
| `.venv/bin/python -m grammar_decode` | Returned the allowed characters `["n","y"]` and the decoded string `"yes"`. |
| Native CLI examples in the README | Integer `-21` returned `true`. The mask after a minus sign returned `[0]`. |
| Native mask benchmark | The 1,024-token vocabulary admitted 64 IDs for the partial UTF-8 prefix. |

The native benchmark ran seven batches of 100 mask calculations. The samples below give mean microseconds per calculation within each batch.

```text
1.23917  1.18208  1.18292  1.18416  1.17917  1.18209  1.17958
```

The samples characterize the local machine and this single workload. The hosted check and CPU benchmark also passed on the same revision. [Hosted conformance evidence](https://github.com/asigdel29/grammar-constrained-decoding/actions/runs/46058052992) · [Hosted benchmark evidence](https://github.com/asigdel29/grammar-constrained-decoding/actions/runs/46058052981)
