//! Translate the supported JSON Schema subset to a byte regular expression.
use serde_json::{Map, Value};
use std::fmt::Write;

const STRING: &str = r#""(?:[\x20-\x21\x23-\x5B\x5D-\x7F]|\\(?:["\\/bfnrt]|u[0-9a-fA-F]{4})|[\xC2-\xDF][\x80-\xBF]|\xE0[\xA0-\xBF][\x80-\xBF]|[\xE1-\xEC\xEE-\xEF][\x80-\xBF]{2}|\xED[\x80-\x9F][\x80-\xBF]|\xF0[\x90-\xBF][\x80-\xBF]{2}|[\xF1-\xF3][\x80-\xBF]{3}|\xF4[\x80-\x8F][\x80-\xBF]{2})*""#;
const INTEGER: &str = r"-?(?:0|[1-9][0-9]*)";
const MAX_PATTERN_BYTES: usize = 1 << 20;

/// Append a pattern fragment within the compiler's bounded workspace.
fn append(dst: &mut String, src: &str) -> Result<(), String> {
    if src.len() > MAX_PATTERN_BYTES.saturating_sub(dst.len()) {
        return Err("Expanded schema exceeds the pattern size limit".into());
    }
    dst.push_str(src);
    Ok(())
}

/// Encode an exact JSON string with Python-compatible ASCII escapes.
fn literal(s: &str, dst: &mut String) -> Result<(), String> {
    let mut encoded = String::from("\"");
    for ch in s.chars() {
        match ch {
            '"' => encoded.push_str("\\\""),
            '\\' => encoded.push_str("\\\\"),
            '\n' => encoded.push_str("\\n"),
            '\r' => encoded.push_str("\\r"),
            '\t' => encoded.push_str("\\t"),
            '\u{8}' => encoded.push_str("\\b"),
            '\u{c}' => encoded.push_str("\\f"),
            ' '..='~' => encoded.push(ch),
            _ => {
                for unit in ch.encode_utf16(&mut [0; 2]) {
                    write!(encoded, "\\u{unit:04x}").unwrap();
                }
            }
        }
        if encoded.len() > MAX_PATTERN_BYTES / 4 {
            return Err("JSON literal exceeds the pattern size limit".into());
        }
    }
    encoded.push('"');
    for byte in encoded.bytes() {
        append(dst, &format!("\\x{byte:02X}"))?;
    }
    Ok(())
}

/// Build a compact byte pattern, preserving property insertion order.
pub(crate) fn pattern(root: &Value) -> Result<String, String> {
    let empty = Map::new();
    let defs = match root.get("$defs") {
        None => &empty,
        Some(value) => value.as_object().ok_or("Expected an object in $defs")?,
    };
    let mut result = String::new();
    node(root, defs, "$", &mut Vec::new(), &mut result, 0)?;
    Ok(result)
}

fn node(
    schema: &Value,
    defs: &Map<String, Value>,
    path: &str,
    active: &mut Vec<String>,
    dst: &mut String,
    depth: usize,
) -> Result<(), String> {
    if depth > 128 {
        return Err(format!("{path}: schema nesting exceeds 128 levels"));
    }
    let obj = schema
        .as_object()
        .ok_or_else(|| format!("{path}: expected a schema object"))?;
    let kind = obj.get("type").and_then(Value::as_str);
    let allowed: &[&str] = if obj.contains_key("$ref") {
        &["$ref"]
    } else {
        match kind {
            Some("array") => &["type", "items"],
            Some("object") => &["type", "properties", "required", "additionalProperties"],
            Some("string") => &["type", "enum"],
            _ => &["type"],
        }
    };
    for key in obj.keys() {
        if !(allowed.contains(&key.as_str())
            || matches!(key.as_str(), "title" | "description" | "default")
            || (depth == 0 && key == "$defs"))
        {
            return Err(format!("{path}: unsupported keyword {key}"));
        }
    }
    if let Some(reference) = obj.get("$ref") {
        let name = reference
            .as_str()
            .and_then(|r| r.strip_prefix("#/$defs/"))
            .ok_or_else(|| format!("{path}: expected a standalone local reference"))?;
        let target = defs
            .get(name)
            .ok_or_else(|| format!("{path}: unknown reference {name}"))?;
        if active.iter().any(|entry| entry == name) {
            return Err(format!("{path}: cyclic reference {name}"));
        }
        active.push(name.to_owned());
        node(
            target,
            defs,
            &format!("#/$defs/{name}"),
            active,
            dst,
            depth + 1,
        )?;
        active.pop();
        return Ok(());
    }
    match kind {
        Some("string") if obj.contains_key("enum") => {
            let list = obj["enum"]
                .as_array()
                .filter(|a| !a.is_empty())
                .ok_or_else(|| format!("{path}: expected a nonempty string enum"))?;
            append(dst, "(?:")?;
            for (i, value) in list.iter().enumerate() {
                if i > 0 {
                    append(dst, "|")?;
                }
                literal(
                    value
                        .as_str()
                        .ok_or_else(|| format!("{path}: expected a string enum"))?,
                    dst,
                )?;
            }
            append(dst, ")")
        }
        Some("string") => append(dst, STRING),
        Some("integer") => append(dst, INTEGER),
        Some("number") => {
            append(dst, INTEGER)?;
            append(dst, r"(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?")
        }
        Some("null") => append(dst, "null"),
        Some("boolean") => append(dst, "(?:true|false)"),
        Some("array") => {
            let item = obj
                .get("items")
                .ok_or_else(|| format!("{path}: missing items"))?;
            let mut child = String::new();
            node(
                item,
                defs,
                &format!("{path}.items"),
                active,
                &mut child,
                depth + 1,
            )?;
            append(dst, r"\[(?:")?;
            append(dst, &child)?;
            append(dst, "(?:,")?;
            append(dst, &child)?;
            append(dst, r")*)?\]")
        }
        Some("object") => {
            let empty = Map::new();
            let properties = match obj.get("properties") {
                None => &empty,
                Some(v) => v
                    .as_object()
                    .ok_or_else(|| format!("{path}: expected named properties"))?,
            };
            if obj
                .get("additionalProperties")
                .is_some_and(|v| !v.is_boolean())
            {
                return Err(format!("{path}: additionalProperties must be boolean"));
            }
            if let Some(req) = obj.get("required") {
                let names = req
                    .as_array()
                    .ok_or_else(|| format!("{path}: required must be an array"))?;
                if names
                    .iter()
                    .any(|k| !k.as_str().is_some_and(|k| properties.contains_key(k)))
                {
                    return Err(format!("{path}: unknown required property"));
                }
            }
            append(dst, r"\{")?;
            for (i, (name, value)) in properties.iter().enumerate() {
                if i > 0 {
                    append(dst, ",")?;
                }
                literal(name, dst)?;
                append(dst, ":")?;
                node(
                    value,
                    defs,
                    &format!("{path}.{name}"),
                    active,
                    dst,
                    depth + 1,
                )?;
            }
            append(dst, r"\}")
        }
        _ => Err(format!("{path}: unsupported schema type {kind:?}")),
    }
}
