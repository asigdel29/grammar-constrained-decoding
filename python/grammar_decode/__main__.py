"""Run a mask example or measure the reference implementation."""

import argparse
import json
from timeit import repeat

from . import compile_schema


def main():
    """Run the fixed byte vocabulary workload and print the result."""
    parser = argparse.ArgumentParser()
    parser.add_argument("--bench", action="store_true")
    args = parser.parse_args()
    schema = {"type": "string", "enum": ["yes", "no"]}
    grammar = compile_schema(schema)
    vocabulary = {i: bytes([i]) for i in range(256)}
    if args.bench:
        def micros(fn):
            """Return raw microseconds per call for each repeated trial."""
            return [s * 10000 for s in repeat(fn, number=100, repeat=5)]
        result = {"vocabulary_size": 256, "compile_us": micros(lambda: compile_schema(schema)),
                  "mask_us": micros(lambda: grammar.allowed_ids(b'"', vocabulary, 256))}
    else:
        result = {"allowed_after_quote": [chr(i) for i in sorted(grammar.allowed_ids(b'"', vocabulary, 256))],
                  "decoded": grammar.finish(b'"yes"')}
    print(json.dumps(result))


if __name__ == "__main__":
    main()
