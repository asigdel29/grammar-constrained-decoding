# System design

The package constrains a decoder to a subset of JSON. It runs in the caller's process. The caller owns the model, tokenizer, logits, and output buffer.

## Components

```mermaid
classDiagram
    class Grammar {
        +expression
        +model
        +accepts(data) bool
        +allowed_ids(prefix, vocabulary, eos_id) set
        +finish(data)
    }
    class Pattern {
        +fullmatch(data, partial)
    }
    class BaseModel {
        +model_json_schema()
        +model_validate_json(data)
    }
    class SchemaError
    class ValueError
    Grammar --> Pattern : holds compiled expression
    Grammar --> BaseModel : optional model class
    SchemaError --|> ValueError
```

`compile_schema` copies the root schema and reads its local definitions. It translates each reachable node into a byte pattern. It returns a frozen `Grammar`.
`compile_model` uses the model's exported schema. It retains the model class for final validation.
Each request can reuse the grammar and vocabulary. Each output stream needs its own prefix.

## Compile contract

| Schema construct | Output rule |
| --- | --- |
| Object | Emit all declared properties once, in declaration order. Emit compact separators. |
| Array | Emit zero or more values of the declared item type. |
| String | Accept JSON escapes and valid raw UTF-8 sequences. |
| String enum | Match the exact ASCII-escaped JSON encoding of a listed value. |
| Number or integer | Use the JSON number syntax for the requested type. |
| Boolean or null | Match the lowercase JSON literal. |
| Local reference | Expand a standalone reference to a named root definition. Reject cycles and missing targets. |

The object rule applies even when the schema allows extra fields or omits a field from `required`.
Annotations such as `default` do not insert values. Unsupported keywords on a compiled node raise `SchemaError`.

## Generation sequence

```mermaid
sequenceDiagram
    actor Caller
    participant G as Grammar
    participant R as Compiled byte pattern
    participant M as Pydantic model
    loop Each decoding step
        Caller->>G: allowed_ids(prefix, vocabulary, eos_id)
        loop Each nonempty text token
            G->>R: fullmatch(prefix + token bytes, partial=True)
            R-->>G: Complete, partial, or absent
        end
        G->>R: fullmatch(prefix)
        G-->>Caller: Allowed IDs, including EOS for a complete prefix
        Caller->>Caller: Select an allowed ID and append text bytes or stop
    end
    Caller->>G: finish(prefix)
    alt Model supplied
        G->>M: model_validate_json(prefix)
        M-->>Caller: Validated model via Grammar, or validation error
    else Schema supplied
        G-->>Caller: Parsed JSON value
    end
```

The end token has no text bytes. A complete numeric prefix can permit both the end token and another digit.
A partial match tests the grammar over bytes. A tokenizer with limited coverage can still reach a prefix with no available continuation.

## Cost and limits

Mask construction scans the vocabulary for each output token. It builds a candidate byte string and matches from the start for each candidate.
Longer prefixes increase copy and match costs. The result set grows with the number of allowed IDs.
The caller applies this set to logits. The package provides no model server or mutable decoder state.

The benchmark measures complete mask construction at fixed checkpoints. Its warm compile path includes regex cache effects.
Use the operating guide to reproduce a run and interpret its records. [Operating guide](operations.md)
