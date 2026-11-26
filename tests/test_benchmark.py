"""Verify that timing samples correspond to valid mask checkpoints."""

import unittest

from grammar_decode.benchmark import benchmark


class BenchmarkTests(unittest.TestCase):
    """Check the report contract and reject unusable workloads."""

    def test_samples(self):
        """Each checkpoint must provide a candidate and the requested samples."""
        report = benchmark(2, 1, 257)
        self.assertEqual(len(report["cases"]), 3)
        for case in report["cases"]:
            self.assertEqual(len(case["warm_compile_us"]), 2)
            for mask in case["masks"]:
                self.assertEqual(len(mask["samples_us"]), 2)
                self.assertGreater(mask["allowed_count"], 0)
        self.assertEqual(report["cases"][0]["masks"][-1]["allowed_count"], 1)

    def test_size(self):
        """The byte vocabulary needs an entry for every possible byte."""
        with self.assertRaises(ValueError):
            benchmark(1, 1, 255)
