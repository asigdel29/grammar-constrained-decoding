# Token eligibility and completion

Supply each token as bytes with a stable ID. A token can span JSON syntax, part of a string, or part of one UTF-8 scalar. Preserve the original bytes even when one token cannot decode as standalone text.

1. Advance the automaton over the existing output prefix.
2. Test each nonempty text token from that state.
3. Include its ID if the token ends in a live state.
4. Include the reserved EOS ID if the prefix can end at this position.

The reserved ID contributes no bytes. Exclude other control tokens from the vocabulary. An empty allowed set can indicate missing vocabulary coverage or a previously invalid prefix.

| Integer prefix | Candidate | Decision |
| --- | --- | --- |
| `-` | `1` | Allow. The next prefix is a complete integer. |
| `-` | EOS | Reject. A digit must follow the sign. |
| `12` | EOS | Allow. The prefix is complete. |
| `12` | `3` | Allow. The integer can grow. |
| `0` | `3` | Reject. The new integer would have a leading zero. |

The Rust API can retain a prefix cursor per output. Its `advance` method returns a new cursor. The Python API accepts the complete prefix on each mask call. Both interfaces use the same native compiler.

At EOS, Python `finish` verifies completion and parses the JSON. A grammar from `compile_model` also runs the model's Pydantic validators. A custom validator can reject structurally valid bytes.

See the [decoder loop](../operations.md#integrate-a-decoder) for a runnable integration.
