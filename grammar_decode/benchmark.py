"""Measure the cost of schema compilation and full vocabulary masks."""

import argparse
from importlib.metadata import version
import json
import os
import platform
from statistics import median
from time import perf_counter_ns

from . import compile_schema


def workloads():
    """Return schemas and valid checkpoints in their compact JSON output."""
    enum = {"type": "string", "enum": ["yes", "no"]}
    record = {"type": "object", "properties": {
        "answer": enum, "scores": {"type": "array", "items": {"type": "number"}}}}
    return [
        ("enum", enum, [b"", b'"', b'"yes"']),
        ("nested_record", record, [b'{"', b'{"answer":"yes","scores":[1,', b'{"answer":"no","scores":[]}']),
        ("long_utf8_string", {"type": "string"}, [b'"' + b"x" * 128 + b"\xe2"]),
    ]


def measure(operation, iterations):
    """Return one batch mean in microseconds per call."""
    start = perf_counter_ns()
    for _ in range(iterations):
        operation()
    return (perf_counter_ns() - start) / (iterations * 1000)


def benchmark(repeats, iterations, vocab_size):
    """Measure warmed paths and preserve raw observations for each checkpoint."""
    if min(repeats, iterations) < 1 or vocab_size < 256 or vocab_size > 65792:
        raise ValueError("Use positive sample counts and a vocabulary size from 256 to 65792")
    vocab = {index: bytes([index]) for index in range(256)}
    vocab.update({index: (index - 256).to_bytes(2, "big") for index in range(256, vocab_size)})
    results = []
    for name, schema, prefixes in workloads():
        grammar = compile_schema(schema)
        compile_samples = [measure(lambda: compile_schema(schema), iterations) for _ in range(repeats)]
        masks = []
        for prefix in prefixes:
            def mask():
                """Build one allowed-token set for the current checkpoint."""
                return grammar.allowed_ids(prefix, vocab, vocab_size)

            allowed = mask()
            if not allowed:
                raise ValueError(f"Benchmark checkpoint has no continuation: {name}/{prefix!r}")
            samples = [measure(mask, iterations) for _ in range(repeats)]
            masks.append({"prefix_hex": prefix.hex(), "allowed_count": len(allowed),
                          "median_us": median(samples), "samples_us": samples})
        results.append({"name": name, "schema": schema, "warm_compile_us": compile_samples, "masks": masks})
    return {"format_version": 1, "python": platform.python_version(), "platform": platform.platform(),
            "machine": platform.machine(), "regex": version("regex"), "git_sha": os.getenv("GITHUB_SHA"),
            "iterations": iterations, "repeats": repeats, "vocabulary_size": vocab_size, "cases": results}


def main():
    """Write a JSON report to standard output."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repeats", type=int, default=7)
    parser.add_argument("--iterations", type=int, default=10)
    parser.add_argument("--vocab-size", type=int, default=1024)
    args = parser.parse_args()
    try:
        report = benchmark(args.repeats, args.iterations, args.vocab_size)
    except ValueError as error:
        parser.error(str(error))
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
