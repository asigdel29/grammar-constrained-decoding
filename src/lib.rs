//! Compile canonical JSON schemas into byte-level finite-state token filters.
use regex_automata::dfa::{dense, Automaton};
use regex_automata::util::{primitives::StateID, syntax};
use regex_automata::{Anchored, Input};
use serde_json::Value;

#[cfg(feature = "python")]
mod python;
mod schema;

/// Immutable byte automaton shared across output streams.
pub struct Grammar {
    dfa: dense::DFA<Vec<u32>>,
    start: StateID,
}

/// A prefix state associated with its originating grammar.
#[derive(Clone, Copy)]
pub struct Prefix<'a> {
    grammar: &'a Grammar,
    state: StateID,
}

impl Grammar {
    /// Compile an ordered JSON schema and resolve acyclic local definitions.
    pub fn compile(schema: &Value) -> Result<Self, String> {
        let pattern = schema::pattern(schema)?;
        let dfa = dense::Builder::new()
            .configure(
                dense::Config::new()
                    .dfa_size_limit(Some(32 << 20))
                    .determinize_size_limit(Some(32 << 20)),
            )
            .syntax(syntax::Config::new().unicode(false).utf8(false))
            .build(&format!(r"\A(?:{pattern})\z"))
            .map_err(|e| format!("Cannot compile byte grammar: {e}"))?;
        let start = dfa
            .start_state_forward(&Input::new(b"").anchored(Anchored::Yes))
            .map_err(|e| e.to_string())?;
        Ok(Self { dfa, start })
    }

    /// Consume the existing bytes once before checking vocabulary entries.
    pub fn prefix(&self, bytes: &[u8]) -> Prefix<'_> {
        Prefix {
            grammar: self,
            state: self.start,
        }
        .advance(bytes)
    }

    /// Check whether a complete output can terminate at its final byte.
    pub fn accepts(&self, bytes: &[u8]) -> bool {
        self.prefix(bytes).complete()
    }

    /// Return token IDs whose byte pieces can extend the given prefix.
    pub fn allowed_ids<'a>(
        &self,
        prefix: &[u8],
        vocabulary: impl IntoIterator<Item = (u32, &'a [u8])>,
        eos_id: u32,
    ) -> Vec<u32> {
        let state = self.prefix(prefix);
        let mut ids = Vec::new();
        for (id, bytes) in vocabulary {
            if id != eos_id && state.permits(bytes) {
                ids.push(id);
            }
        }
        if state.complete() {
            ids.push(eos_id);
        }
        ids
    }
}

impl Prefix<'_> {
    /// Advance a copy of this state over one token or byte span.
    pub fn advance(mut self, bytes: &[u8]) -> Self {
        let dfa = &self.grammar.dfa;
        for &byte in bytes {
            self.state = dfa.next_state(self.state, byte);
            if dfa.is_dead_state(self.state) {
                break;
            }
        }
        self
    }

    /// Accept EOS only if this position completes the anchored expression.
    pub fn complete(self) -> bool {
        let dfa = &self.grammar.dfa;
        dfa.is_match_state(dfa.next_eoi_state(self.state))
    }

    /// Test if a nonempty piece leaves the prefix viable.
    pub fn permits(self, piece: &[u8]) -> bool {
        !piece.is_empty() && !self.grammar.dfa.is_dead_state(self.advance(piece).state)
    }
}
