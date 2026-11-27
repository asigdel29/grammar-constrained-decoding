# Grammar-constrained decoding

Use a JSON Schema or a Pydantic v2 model to restrict the next token. Rust compiles the schema and computes masks. Python bindings support Pydantic v2.

## Quickstart

Install stable Rust and Python 3.10 or later.

```sh
python3 -m venv .venv
. .venv/bin/activate
python -m pip install '.[pydantic]'
make check
make demo
make bench
```

The demo lists the allowed next characters for a two-value enumeration.

## Native library and CLI

Install the CLI with `cargo install --path . --locked`. It works without Python.
Use `accepts` to validate the exact bytes from standard input.

```sh
printf '%s' '{"type":"integer"}' > /tmp/grammar-integer.json
printf '%s' '-21' | grammar-decode accepts /tmp/grammar-integer.json
printf '%s' '{"prefix":[45],"vocabulary":{"0":[49],"1":[120]},"eos_id":2}' | grammar-decode mask /tmp/grammar-integer.json
```

The two commands print `true` and `[0]`. The validation command returns 1 for invalid output. Invalid schemas and malformed requests return 2.

Use `Grammar::compile` from Rust. Use `Grammar::prefix` to obtain a reusable cursor. Each call to `advance` returns a new cursor.

## CI and benchmarks

The Check workflow builds a wheel and tests it outside the checkout on Python 3.10, 3.12, and 3.13.

The CPU benchmark runs on each push to main. You can also start it from the Actions tab. Each artifact contains raw timing batches, package versions, and CPU details.

To reproduce the benchmark, use Python 3.12 and the locked Rust dependencies.

```sh
python -m pip install .
cargo bench --locked --bench masks
python -m grammar_decode.benchmark --repeats 7 --iterations 10 --vocab-size 1024 > measurements.json
```

The workloads cover a string enumeration, a nested record, and a partial UTF-8 character after a long prefix. Each batch reports mean microseconds per call. Each mask record includes the median across batches, every raw sample, and the number of allowed tokens. Compile measurements include construction of a new automaton.

The synthetic vocabulary includes every single byte. Extra tokens use successive two-byte values in big-endian order. The workload measures full mask construction for a fixed prefix. Hosted runners can differ in CPU and load. Compare results on the same machine for performance analysis.

## Interface

```python
from grammar_decode import compile_schema

grammar = compile_schema({"type": "string", "enum": ["yes", "no"]})
vocabulary = {0: b'"y', 1: b'es"', 2: b'"no"', 3: b'bad'}
allowed = grammar.allowed_ids(b"", vocabulary, eos_id=4)
assert allowed == {0, 2}
assert grammar.finish(b'"yes"') == "yes"
```

Map each text token to its exact bytes. Exclude control tokens from the vocabulary. Use a unique ID for the end token. Each token must have the same bytes in every context. Text from standalone token decoding can lose partial UTF-8 sequences.

Add each selected token to the byte prefix. The mask admits the end token once the grammar accepts that prefix. Call `compile_model(Model)` to use a Pydantic model. Call `finish` to apply its Python validators.

## Schema support

- Use JSON strings, integers, numbers, booleans, null, and string enumerations.
- Use homogeneous arrays and objects with named properties.
- Use acyclic local references of the form `#/$defs/name`.
- Use `title`, `description`, and `default` as annotations.
- The grammar emits compact JSON with all declared fields in schema order. It omits extra fields and optional whitespace.
- Unsupported constraints raise `SchemaError` during compilation. Python validators can reject a structurally valid result.

The compiler builds an anchored byte automaton. The mask advances through the prefix once. Each candidate starts at this saved state.

## License

Apache License 2.0.

## Documentation

- [System design and UML diagrams](docs/system-design.md)
- [Operating guide and decoder integration](docs/operations.md)
- [Repository instructions](AGENTS.md)

## Graphify source map

Read the [source map report](graphify-out/GRAPH_REPORT.md) or query the [graph data](graphify-out/graph.json). Open the [interactive map](graphify-out/graph.html) locally in a browser.

Install Graphify 0.9.76 with `uv tool install graphifyy==0.9.76`. Run these commands from the repository root.

```sh
make graph
graphify query "compile_schema"
```

The build reads tracked Python and Rust files from the working tree, including tests. Stage new source files before you refresh the map. Graphify uses local syntax analysis with model backends disabled. The system design diagrams describe runtime behavior.
