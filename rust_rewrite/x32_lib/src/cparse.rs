
use crate::sprint::{xsprint_f, xsprint_i, xsprint_s};

pub fn xcparse(input: &str) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    let mut parts = input.splitn(2, |c| c == ' ' || c == ',');

    let path = parts.next().ok_or("Empty command")?.trim();
    if path.is_empty() {
        return Err("Empty command".to_string());
    }
    xsprint_s(&mut buf, path);

    let remaining = match parts.next() {
        Some(s) => s.trim_start(),
        None => return Ok(buf),
    };

    let type_tags = if input.contains(',') {
        let comma_pos = input.find(',').ok_or("Invalid format: expected comma for type tags")?;
        let tags_part = input[comma_pos..].split_whitespace().next().ok_or("Invalid format: expected comma for type tags")?;
        if !tags_part.starts_with(',') {
            return Err("Invalid format: expected comma for type tags".to_string());
        }
        tags_part
    } else {
        return Ok(buf);
    };

    xsprint_s(&mut buf, type_tags);

    let mut args_str = &input[input.find(type_tags).ok_or("Type tags not found")? + type_tags.len()..];

    for tag in type_tags.chars().skip(1) {
        args_str = args_str.trim_start();
        if args_str.is_empty() {
            return Err(format!("Missing argument for type tag '{}'", tag));
        }

        match tag {
            's' => {
                let (arg, rest) = parse_string(args_str)?;
                xsprint_s(&mut buf, arg);
                args_str = rest;
            }
            'i' => {
                let (arg_str, rest) = args_str.split_at(
                    args_str.find(char::is_whitespace).unwrap_or(args_str.len())
                );
                let i: i32 = arg_str.parse().map_err(|e| format!("Invalid integer '{}': {}", arg_str, e))?;
                xsprint_i(&mut buf, i);
                args_str = rest;
            }
            'f' => {
                let (arg_str, rest) = args_str.split_at(
                    args_str.find(char::is_whitespace).unwrap_or(args_str.len())
                );
                let f: f32 = arg_str.parse().map_err(|e| format!("Invalid float '{}': {}", arg_str, e))?;
                xsprint_f(&mut buf, f);
                args_str = rest;
            }
            _ => return Err(format!("Unsupported type tag '{}'", tag)),
        }
    }

    Ok(buf)
}

fn parse_string(input: &str) -> Result<(&str, &str), String> {
    if input.starts_with('"') || input.starts_with('\'') {
        let quote = input.chars().next().ok_or("Empty string")?;
        let end_quote_pos = input[1..].find(quote).ok_or_else(|| format!("Unmatched quote in string: {}", input))?;
        let arg = &input[1..end_quote_pos + 1];
        let rest = &input[end_quote_pos + 2..];
        Ok((arg, rest))
    } else {
        let end_space_pos = input.find(char::is_whitespace).unwrap_or(input.len());
        let arg = &input[..end_space_pos];
        let rest = &input[end_space_pos..];
        Ok((arg, rest))
    }
}
