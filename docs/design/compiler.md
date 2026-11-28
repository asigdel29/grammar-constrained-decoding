# Compiler contract

The compiler translates a schema into a byte pattern and builds an anchored deterministic finite automaton. A complete match consumes the entire byte string.

| Input | Canonical language |
| --- | --- |
| String | A quoted JSON string with legal escape sequences and UTF-8 byte sequences. |
| String enum | The exact ASCII-escaped JSON spelling of each listed value. |
| Number | JSON numeric syntax, including an optional fraction and exponent. Integers accept only the integer part. |
| Array | Brackets around zero or more items from one schema, separated by commas. |
| Object | Braces around all declared fields in property order, with compact separators. |
| Local reference | A standalone reference to a schema in the root `$defs` object. |
| Annotations | `title`, `description`, and `default` do not change the byte language. |

For properties `z` and `a` with integer values, `{"z":1,"a":2}` can pass. Reversing the fields, omitting either field, or adding exterior whitespace fails the structural check. This canonical output rule also applies when the input schema marks a field optional.

The compiler rejects unsupported keywords such as `minimum`, cyclic references, and missing targets. Python exposes these errors as `SchemaError`. Inspect the failing path before changing the schema or sampling code.

The compiler limits nesting to 128 levels and the expanded pattern to one MiB. The DFA builder limits its construction workspace and final table to 32 MiB each. Large schemas can reach these limits even when the source JSON is small.

See the [schema translator](../../src/schema.rs) and [compiler implementation](../../src/lib.rs).
