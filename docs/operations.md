# Operating guide

## Set up the reference implementation

Use Python 3.10 or later. Run the following commands from the repository root.

```sh
python3 -m venv .venv
. .venv/bin/activate
python -m pip install '.[pydantic]'
make check
make demo
```

## Integrate a decoder

1. Compile the schema once.
2. Map each text token ID to its context-independent bytes.
3. Reserve an ID for the end token.
4. Start with an empty output prefix.
5. Request the allowed IDs before each sample.
6. Exclude all other IDs from the sampler.
7. Append each selected text token to the prefix.
8. Call `finish` when the sampler selects the end token.

This example uses predetermined samples to show the byte-boundary contract.

```python
from grammar_decode import compile_schema

grammar = compile_schema({"type": "array", "items": {"type": "integer"}})
vocab = {0: b"[", 1: b"12", 2: b",", 3: b"-3]", 4: b"oops"}
eos = 5
prefix = b""
for token in (0, 1, 2, 3, eos):
    assert token in grammar.allowed_ids(prefix, vocab, eos)
    if token == eos:
        break
    prefix += vocab[token]

assert grammar.finish(prefix) == [12, -3]
```

Use raw token bytes for strings with multibyte characters. Some tokens contain only the first bytes of a character.
An adapter that decodes each piece to text can discard these bytes.

## Diagnose a rejected output

| Symptom | Next check |
| --- | --- |
| Compilation raises `SchemaError` | Inspect the reported schema path. Check constraints, local reference targets, and supported types. |
| The allowed set is empty | Inspect `prefix.hex()` and the exact bytes of candidate IDs. Check field order and vocabulary coverage. |
| A complete number permits another token | Let the sampler choose between a valid continuation and the end token. |
| A custom model validator rejects the result | Inspect the Pydantic validation error at finalization. Custom Python predicates run after the structural check. |
| A length limit interrupts generation | Test `accepts(prefix)` and report incomplete output when it returns false. |

## Reproduce a CPU benchmark

Use the same CPU, Python version, regex version, and vocabulary size for comparisons.

```sh
python -m pip install . 'regex==2024.11.6'
python -m grammar_decode.benchmark --repeats 7 --iterations 10 --vocab-size 1024 > measurements.json
```

Each sample is the mean time per call within one batch, in microseconds. Compare the same case and prefix across runs.
Use `allowed_count` to check that both runs permit the same number of IDs.
The report records the schema, prefix bytes, raw samples, and environment. CI adds the installed package list and CPU description to the artifact.
