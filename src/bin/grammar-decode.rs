//! Validate stdin or compute a token mask using a schema file.
use grammar_decode::Grammar;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    env, fs,
    io::{self, Read},
    process::ExitCode,
};

/// Read raw output bytes for accepts, or a JSON byte-vocabulary request for mask.
fn run() -> Result<ExitCode, String> {
    let args: Vec<_> = env::args().skip(1).collect();
    if args.len() != 2 || !matches!(args[0].as_str(), "accepts" | "mask") {
        return Err("usage: grammar-decode <accepts|mask> SCHEMA.json < input".into());
    }
    let source = fs::read(&args[1]).map_err(|e| e.to_string())?;
    let schema = serde_json::from_slice(&source).map_err(|e| e.to_string())?;
    let grammar = Grammar::compile(&schema)?;
    let mut input = Vec::new();
    io::stdin()
        .read_to_end(&mut input)
        .map_err(|e| e.to_string())?;
    if args[0] == "accepts" {
        let accepted = grammar.accepts(&input);
        println!("{accepted}");
        return Ok(if accepted {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(1)
        });
    }
    let request: Value = serde_json::from_slice(&input).map_err(|e| e.to_string())?;
    let prefix: Vec<u8> =
        serde_json::from_value(request["prefix"].clone()).map_err(|e| e.to_string())?;
    let vocab: BTreeMap<u32, Vec<u8>> =
        serde_json::from_value(request["vocabulary"].clone()).map_err(|e| e.to_string())?;
    let eos: u32 = serde_json::from_value(request["eos_id"].clone()).map_err(|e| e.to_string())?;
    let mut allowed = grammar.allowed_ids(
        &prefix,
        vocab.iter().map(|(&id, bytes)| (id, bytes.as_slice())),
        eos,
    );
    allowed.sort_unstable();
    println!("{}", serde_json::json!(allowed));
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    run().unwrap_or_else(|error| {
        eprintln!("{error}");
        ExitCode::from(2)
    })
}
