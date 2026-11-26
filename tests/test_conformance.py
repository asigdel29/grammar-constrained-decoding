"""Check conformance at schema, token, and final validation boundaries."""

import unittest

from pydantic import BaseModel, field_validator
from grammar_decode import SchemaError, compile_model, compile_schema


class Row(BaseModel):
    """Require a nonempty label after decoding."""

    label: str

    @field_validator("label")
    @classmethod
    def nonempty(cls, value):
        """Reject an empty label."""
        if not value:
            raise ValueError("Empty label")
        return value


class Batch(BaseModel):
    """Wrap nested rows to exercise local schema references."""

    rows: list[Row]


class Conformance(unittest.TestCase):
    """Check canonical output and the exact token boundary contract."""

    def test_token_boundaries(self):
        grammar = compile_model(Batch)
        pieces = {0: b'{"ro', 1: b'ws":[{"label":"', 2: b'\xf0\x9f', 3: b'\x98\x80"}]}', 4: b"", 5: b"x"}
        data = b""
        for token_id in range(4):
            allowed = grammar.allowed_ids(data, pieces, 100)
            self.assertIn(token_id, allowed)
            self.assertNotIn(100, allowed)
            self.assertNotIn(4, allowed)
            data += pieces[token_id]
        self.assertEqual(grammar.allowed_ids(data, pieces, 100), {100})
        self.assertEqual(grammar.finish(data).rows[0].label, "😀")

    def test_invalid_unicode_and_escape(self):
        grammar = compile_schema({"type": "string"})
        pieces = {1: b"\x80", 2: b"\xc0\xaf", 3: b"\xed\xa0", 4: b"\\x", 5: b'\\u00', 6: b'\n'}
        self.assertEqual(grammar.allowed_ids(b'"', pieces, 9), {5})

    def test_pydantic_validator(self):
        grammar = compile_model(Row)
        data = b'{"label":""}'
        self.assertTrue(grammar.accepts(data))
        with self.assertRaises(ValueError):
            grammar.finish(data)

    def test_enum_and_numbers(self):
        grammar = compile_schema({"type": "array", "items": {"type": "string", "enum": ["a.", "b"]}})
        self.assertEqual(grammar.finish(b'["a.","b"]'), ["a.", "b"])
        self.assertEqual(grammar.finish(b"[]"), [])
        for invalid in (b'["aa"]', b'["a.",]', b'[true]'):
            self.assertFalse(grammar.accepts(invalid))
        number = compile_schema({"type": "number"})
        for valid in (b"0", b"-0.5e+09", b"28"):
            self.assertTrue(number.accepts(valid))
        for invalid in (b"02", b".3", b"3e", b"+8"):
            self.assertFalse(number.accepts(invalid))

    def test_unsupported_and_cycles(self):
        for schema in (
            {"type": "integer", "minimum": 3},
            {"type": "array"},
            {"$ref": "https://example.com/schema.json"},
            {"$defs": {"x": {"$ref": "#/$defs/x"}}, "$ref": "#/$defs/x"},
            {"anyOf": [{"type": "null"}, {"type": "integer"}]},
        ):
            with self.subTest(schema=schema), self.assertRaises(SchemaError):
                compile_schema(schema)
