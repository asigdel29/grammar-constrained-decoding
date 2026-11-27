//! Emit raw batch timings for Rust schema compilation and token masks.
use grammar_decode::Grammar;
use serde_json::json;
use std::{hint::black_box, time::Instant};

fn main() {
    let schema = json!({"type":"array","items":{"type":"string"}});
    let vocab: Vec<Vec<u8>> = (0..1024u16)
        .map(|i| {
            if i < 256 {
                vec![i as u8]
            } else {
                (i - 256).to_be_bytes().to_vec()
            }
        })
        .collect();
    let grammar = Grammar::compile(&schema).unwrap();
    let mut prefix = vec![b'[', b'"'];
    prefix.extend([b'x'; 128]);
    prefix.push(0xe2);
    let mask = || {
        grammar.allowed_ids(
            &prefix,
            vocab
                .iter()
                .enumerate()
                .map(|(i, b)| (i as u32, b.as_slice())),
            1024,
        )
    };
    let allowed_count = mask().len();
    assert_eq!(allowed_count, 64);
    let mut samples = Vec::new();
    for _ in 0..7 {
        let start = Instant::now();
        for _ in 0..100 {
            black_box(mask());
        }
        samples.push(start.elapsed().as_secs_f64() * 1e6 / 100.0);
    }
    println!(
        "{}",
        json!({"case":"array_string_split_utf8","iterations":100,"samples_us":samples,"vocab_size":1024,"allowed_count":allowed_count})
    );
}
