//! Check prefix viability, EOS and ordered-schema rules without Python.
use grammar_decode::Grammar;
use serde_json::json;

#[test]
fn arrays_and_numbers() {
    let grammar = Grammar::compile(&json!({"type":"array","items":{"type":"number"}})).unwrap();
    for output in ["[]", "[-12e+2]", "[1.5,0,-0]"] {
        assert!(grammar.accepts(output.as_bytes()), "{output}");
        let mut state = grammar.prefix(b"");
        for b in output.bytes() {
            assert!(state.permits(&[b]));
            state = state.advance(&[b]);
        }
        assert!(state.complete());
    }
    for output in ["[,", "[1,]", "[01", "[0e."] {
        assert!(!grammar.prefix(b"").permits(output.as_bytes()), "{output}");
    }
    assert!(!grammar.accepts(b"[1e"));
    assert!(grammar.prefix(b"[1e").permits(b"-"));
}

#[test]
fn unicode_and_enum_literals() {
    let grammar = Grammar::compile(&json!({"type":"string","enum":["😀","\u{7f}"]})).unwrap();
    assert!(grammar.accepts(br#""\ud83d\ude00""#));
    assert!(grammar.accepts(br#""\u007f""#));
    let text = Grammar::compile(&json!({"type":"string"})).unwrap();
    let prefix = [b'"', 0xe0];
    assert!(!text.prefix(&prefix).permits(&[0x90]));
    assert!(text.prefix(&prefix).permits(&[0xa0]));
    assert!(!text.prefix(&prefix).complete());
}

#[test]
fn field_order_and_local_definitions() {
    let schema = json!({"$defs":{"t":{"type":"boolean"}}, "type":"object",
        "properties":{"z":{"$ref":"#/$defs/t"}, "a":{"type":"integer"}}});
    let grammar = Grammar::compile(&schema).unwrap();
    assert!(grammar.accepts(br#"{"z":true,"a":0}"#));
    assert!(!grammar.accepts(br#"{"a":0,"z":true}"#));
    assert!(
        Grammar::compile(&json!({"$ref":"#/$defs/a", "$defs":{"a":{"$ref":"#/$defs/a"}}})).is_err()
    );
    assert!(Grammar::compile(&json!({"type":"integer","minimum":0})).is_err());
}

#[test]
fn eos_requires_a_complete_prefix() {
    let g = Grammar::compile(&json!({"type":"integer"})).unwrap();
    let vocab: [(u32, &[u8]); 4] = [(7, b"x"), (1, b"1"), (2, b""), (3, b"9")];
    assert_eq!(g.allowed_ids(b"-", vocab, 7), [1, 3]);
    assert_eq!(g.allowed_ids(b"0", vocab, 7), [7]);
    assert!(g.allowed_ids(b"00", vocab, 7).is_empty());
}
