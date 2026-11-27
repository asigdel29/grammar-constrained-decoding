"""Python and Pydantic adapters for the Rust byte grammar."""

from dataclasses import dataclass
import json

from ._native import NativeGrammar


class SchemaError(ValueError):
    """Identify unsupported or invalid schema constraints before generation."""


@dataclass(frozen=True)
class Grammar:
    """Recognize complete outputs and valid continuations of one schema."""

    expression: NativeGrammar
    model: object = None

    def accepts(self, data: bytes) -> bool:
        """Return whether a stream may end at this byte boundary."""
        return self.expression.accepts(data)

    def allowed_ids(self, prefix: bytes, vocabulary: dict[int, bytes], eos_id: int) -> set[int]:
        """Return token IDs that preserve a possible completion of the prefix."""
        return self.expression.allowed_ids(prefix, vocabulary, eos_id)

    def finish(self, data: bytes):
        """Parse the complete output and apply the caller's model validators."""
        if not self.accepts(data):
            raise ValueError("Output does not match the compiled schema")
        if self.model is not None:
            return self.model.model_validate_json(data)
        return json.loads(data)


def compile_schema(schema: dict) -> Grammar:
    """Compile an ordered schema with the native compiler."""
    if not isinstance(schema, dict):
        raise SchemaError("Expected a schema object")
    try:
        return Grammar(NativeGrammar(json.dumps(schema)))
    except (ValueError, TypeError) as error:
        raise SchemaError(str(error)) from error


def compile_model(model) -> Grammar:
    """Retain a Pydantic model for final validation."""
    return Grammar(compile_schema(model.model_json_schema()).expression, model)
