"""Compile a JSON Schema subset into a byte-level token mask."""

from dataclasses import dataclass
import json

import regex


class SchemaError(ValueError):
    """Identify unsupported or invalid schema constraints before generation."""


# Each branch accepts one UTF-8 scalar, one ASCII character, or one JSON escape.
_STRING = (
    rb'"(?:[\x20-\x21\x23-\x5b\x5d-\x7f]|\\(?:["\\/bfnrt]|u[0-9a-fA-F]{4})'
    rb'|[\xc2-\xdf][\x80-\xbf]|\xe0[\xa0-\xbf][\x80-\xbf]'
    rb'|[\xe1-\xec\xee-\xef][\x80-\xbf]{2}|\xed[\x80-\x9f][\x80-\xbf]'
    rb'|\xf0[\x90-\xbf][\x80-\xbf]{2}|[\xf1-\xf3][\x80-\xbf]{3}'
    rb'|\xf4[\x80-\x8f][\x80-\xbf]{2})*"'
)
_INTEGER = rb"-?(?:0|[1-9][0-9]*)"
_SCALARS = {
    "string": _STRING,
    "integer": _INTEGER,
    "number": _INTEGER + rb"(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?",
    "boolean": rb"(?:true|false)",
    "null": rb"null",
}


def _literal(value):
    """Encode one JSON literal as an exact byte pattern."""
    return regex.escape(json.dumps(value, ensure_ascii=True).encode("ascii"))


def _pattern(schema, definitions, path, active):
    """Translate one supported node and reject unhandled constraints."""
    if not isinstance(schema, dict):
        raise SchemaError(f"{path}: expected a schema object")
    node = schema.copy()
    for key in ("title", "description", "default"):
        node.pop(key, None)
    if "$ref" in node:
        ref = node.pop("$ref")
        if node or not isinstance(ref, str) or not ref.startswith("#/$defs/"):
            raise SchemaError(f"{path}: expected a local standalone reference")
        name = ref[len("#/$defs/"):]
        if name in active or name not in definitions:
            raise SchemaError(f"{path}: cyclic or unknown reference {ref}")
        return _pattern(definitions[name], definitions, ref, active | {name})

    kind = node.pop("type", None)
    if kind == "string" and "enum" in node:
        values = node.pop("enum")
        if not isinstance(values, list) or not values or not all(isinstance(v, str) for v in values):
            raise SchemaError(f"{path}: expected a nonempty string enum")
        pattern = b"(?:" + b"|".join(_literal(v) for v in values) + b")"
    elif isinstance(kind, str) and kind in _SCALARS:
        pattern = _SCALARS[kind]
    elif kind == "array":
        item = _pattern(node.pop("items", None), definitions, path + ".items", active)
        # Each item must precede the closing bracket or a comma and another item.
        pattern = rb"\[(?:" + item + rb"(?:(?=\])|,(?!\])))*\]"
    elif kind == "object":
        properties = node.pop("properties", {})
        required = node.pop("required", [])
        extra = node.pop("additionalProperties", True)
        if not isinstance(properties, dict) or not isinstance(extra, bool):
            raise SchemaError(f"{path}: expected named properties")
        if not isinstance(required, list) or not all(isinstance(k, str) and k in properties for k in required):
            raise SchemaError(f"{path}: unknown required property")
        fields = []
        for name, child in properties.items():
            if not isinstance(name, str):
                raise SchemaError(f"{path}: expected a string property name")
            fields.append(_literal(name) + b":" + _pattern(child, definitions, f"{path}.{name}", active))
        pattern = rb"\{" + b",".join(fields) + rb"\}"
    else:
        raise SchemaError(f"{path}: unsupported type {kind!r}")
    if node:
        raise SchemaError(f"{path}: unsupported keywords {sorted(node)}")
    return pattern


@dataclass(frozen=True)
class Grammar:
    """Recognize complete outputs and valid continuations of one schema."""

    expression: regex.Pattern
    model: object = None

    def accepts(self, data: bytes) -> bool:
        """Return whether a stream may end at this byte boundary."""
        return self.expression.fullmatch(data) is not None

    def allowed_ids(self, prefix: bytes, vocabulary: dict[int, bytes], eos_id: int) -> set[int]:
        """Return token IDs that preserve a possible completion of the prefix."""
        allowed = {
            token_id for token_id, piece in vocabulary.items()
            if token_id != eos_id and piece
            and self.expression.fullmatch(prefix + piece, partial=True) is not None
        }
        if self.accepts(prefix):
            allowed.add(eos_id)
        return allowed

    def finish(self, data: bytes):
        """Parse a complete stream and run the model validators when configured."""
        if not self.accepts(data):
            raise ValueError("Output does not match the compiled schema")
        if self.model is not None:
            return self.model.model_validate_json(data)
        return json.loads(data)


def compile_schema(schema: dict) -> Grammar:
    """Compile a JSON Schema document into its canonical generation grammar."""
    if not isinstance(schema, dict):
        raise SchemaError("Expected a schema object")
    root = schema.copy()
    definitions = root.pop("$defs", {})
    if not isinstance(definitions, dict):
        raise SchemaError("Expected an object in $defs")
    return Grammar(regex.compile(_pattern(root, definitions, "$", set())))


def compile_model(model) -> Grammar:
    """Use the JSON Schema and final validators of a Pydantic v2 model."""
    return Grammar(compile_schema(model.model_json_schema()).expression, model)
