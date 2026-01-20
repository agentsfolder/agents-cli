use crate::model::StampMethod;

use super::encoding::{
    encode_stamp_meta_json, COMMENT_STAMP_PREFIX, COMMENT_STAMP_SUFFIX, FRONTMATTER_STAMP_KEY,
    JSON_STAMP_FIELD,
};
use super::types::{Stamp, StampMeta};

#[derive(Debug, thiserror::Error)]
pub enum StampError {
    #[error("invalid stamp json: {0}")]
    InvalidStampJson(#[from] serde_json::Error),

    #[error("json_field stamping requires a top-level object")]
    JsonFieldNotObject,
}

pub fn parse_stamp(content: &str) -> Option<Stamp> {
    if let Some(meta) = parse_comment_stamp(content) {
        return Some(Stamp {
            method: StampMethod::Comment,
            meta,
        });
    }

    if let Some(meta) = parse_frontmatter_stamp(content) {
        return Some(Stamp {
            method: StampMethod::Frontmatter,
            meta,
        });
    }

    if let Some(meta) = parse_json_field_stamp(content) {
        return Some(Stamp {
            method: StampMethod::JsonField,
            meta,
        });
    }

    None
}

pub fn strip_existing_stamp(content: &str) -> (String, Option<Stamp>) {
    if let Some(meta) = parse_comment_stamp(content) {
        let (_, rest) = split_first_line(content);
        return (
            rest.to_string(),
            Some(Stamp {
                method: StampMethod::Comment,
                meta,
            }),
        );
    }

    if let Some((stripped, meta)) = strip_frontmatter_stamp(content) {
        return (
            stripped,
            Some(Stamp {
                method: StampMethod::Frontmatter,
                meta,
            }),
        );
    }

    if let Some((stripped, meta)) = strip_json_field_stamp(content) {
        return (
            stripped,
            Some(Stamp {
                method: StampMethod::JsonField,
                meta,
            }),
        );
    }

    (content.to_string(), None)
}

pub fn apply_stamp(
    content_without_stamp: &str,
    meta: &StampMeta,
    method: StampMethod,
) -> Result<String, StampError> {
    match method {
        StampMethod::Comment => {
            let json = encode_stamp_meta_json(meta)?;
            Ok(format!(
                "{}{}{}\n{}",
                COMMENT_STAMP_PREFIX, json, COMMENT_STAMP_SUFFIX, content_without_stamp
            ))
        }
        StampMethod::Frontmatter => {
            let json = encode_stamp_meta_json(meta)?;
            let stamp_line = format!("{}: {}\n", FRONTMATTER_STAMP_KEY, json);

            if content_without_stamp.starts_with("---\n") {
                // Insert stamp line immediately after frontmatter start delimiter.
                Ok(format!(
                    "---\n{}{}",
                    stamp_line,
                    &content_without_stamp[4..]
                ))
            } else {
                Ok(format!("---\n{}---\n{}", stamp_line, content_without_stamp))
            }
        }
        StampMethod::JsonField => {
            let json = encode_stamp_meta_json(meta)?;
            apply_json_field_stamp(content_without_stamp, &json)
        }
    }
}

pub fn stamp_rendered_output(
    content: &str,
    meta: &StampMeta,
    method: StampMethod,
) -> Result<String, StampError> {
    let (stripped, _) = strip_existing_stamp(content);
    apply_stamp(&stripped, meta, method)
}

fn split_first_line(s: &str) -> (&str, &str) {
    match s.find('\n') {
        Some(i) => (&s[..i], &s[i + 1..]),
        None => (s, ""),
    }
}

fn parse_comment_stamp(content: &str) -> Option<StampMeta> {
    let (line, _) = split_first_line(content);
    if !line.starts_with(COMMENT_STAMP_PREFIX) {
        return None;
    }
    if !line.ends_with(COMMENT_STAMP_SUFFIX) {
        return None;
    }

    let json = &line[COMMENT_STAMP_PREFIX.len()..line.len() - COMMENT_STAMP_SUFFIX.len()];
    serde_json::from_str::<StampMeta>(json).ok()
}

fn frontmatter_bounds(content: &str) -> Option<(usize, usize)> {
    if !content.starts_with("---\n") {
        return None;
    }

    let rest = &content[4..];
    let end_rel = rest.find("\n---\n")?;

    let fm_start = 4;
    let fm_end = 4 + end_rel + 1; // include the leading '\n' of the terminator
    Some((fm_start, fm_end))
}

fn parse_frontmatter_stamp(content: &str) -> Option<StampMeta> {
    let (fm_start, fm_end) = frontmatter_bounds(content)?;
    let fm = &content[fm_start..fm_end];

    for line in fm.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with(FRONTMATTER_STAMP_KEY) {
            continue;
        }
        let rest = trimmed.strip_prefix(FRONTMATTER_STAMP_KEY)?;
        let rest = rest.trim_start();
        if !rest.starts_with(':') {
            continue;
        }
        let json = rest[1..].trim();
        return serde_json::from_str::<StampMeta>(json).ok();
    }

    None
}

fn strip_frontmatter_stamp(content: &str) -> Option<(String, StampMeta)> {
    let (fm_start, fm_end) = frontmatter_bounds(content)?;

    let fm = &content[fm_start..fm_end];
    let mut found: Option<(StampMeta, usize, usize)> = None;

    // Find the exact byte range of the line containing `x_generated:`.
    let mut line_start = fm_start;
    for line in fm.split_inclusive('\n') {
        let line_end = line_start + line.len();
        let trimmed = line.trim();
        if trimmed.starts_with(FRONTMATTER_STAMP_KEY) {
            let rest = trimmed.strip_prefix(FRONTMATTER_STAMP_KEY)?;
            let rest = rest.trim_start();
            if rest.starts_with(':') {
                let json = rest[1..].trim();
                if let Ok(meta) = serde_json::from_str::<StampMeta>(json) {
                    found = Some((meta, line_start, line_end));
                    break;
                }
            }
        }
        line_start = line_end;
    }

    let (meta, rm_start, rm_end) = found?;

    let mut new_fm = String::new();
    new_fm.push_str(&content[fm_start..rm_start]);
    new_fm.push_str(&content[rm_end..fm_end]);

    // If frontmatter becomes empty/whitespace-only after removing stamp, drop it entirely.
    let only_ws = new_fm.trim().is_empty();
    if only_ws {
        let after = &content[fm_end + 4..]; // skip "---\n" terminator (4 bytes)
        return Some((after.to_string(), meta));
    }

    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(new_fm.trim_end_matches('\n'));
    out.push_str("\n---\n");
    out.push_str(&content[fm_end + 4..]);
    Some((out, meta))
}

fn apply_json_field_stamp(content: &str, meta_json: &str) -> Result<String, StampError> {
    let bytes = content.as_bytes();

    let mut open_idx = 0usize;
    open_idx = skip_ws_and_jsonc_comments(bytes, open_idx).unwrap_or(open_idx);
    if bytes.get(open_idx) != Some(&b'{') {
        return Err(StampError::JsonFieldNotObject);
    }

    let after_open = open_idx + 1;

    // Find the first non-ws/comment token after `{`.
    let mut i = after_open;
    i = skip_ws_and_jsonc_comments(bytes, i).unwrap_or(i);

    // Empty object case.
    if i < bytes.len() && bytes[i] == b'}' {
        let prefix = &content[..open_idx];
        return Ok(format!(
            "{}{{\n  \"{}\": {}\n}}",
            prefix, JSON_STAMP_FIELD, meta_json
        ));
    }

    // Multi-line insertion if object starts with `{\n`.
    if bytes.get(after_open) == Some(&(b'\n')) {
        // Detect field indent from first existing field.
        let mut j = after_open + 1;
        while j < bytes.len() {
            let c = bytes[j] as char;
            if c != ' ' && c != '\t' {
                break;
            }
            j += 1;
        }
        let indent = &content[after_open + 1..j];

        let mut out = String::new();
        out.push_str(&content[..after_open + 1]); // include "{\n"
        out.push_str(indent);
        out.push_str("\"");
        out.push_str(JSON_STAMP_FIELD);
        out.push_str("\": ");
        out.push_str(meta_json);
        out.push_str(",\n");
        out.push_str(&content[after_open + 1..]);
        return Ok(out);
    }

    // Single-line insertion.
    let rest = &content[after_open..];
    let need_space = rest
        .as_bytes()
        .first()
        .is_some_and(|b| !(*b as char).is_whitespace());

    let mut out = String::new();
    out.push_str(&content[..after_open]);
    out.push_str("\"");
    out.push_str(JSON_STAMP_FIELD);
    out.push_str("\": ");
    out.push_str(meta_json);
    out.push(',');
    if need_space {
        out.push(' ');
    }
    out.push_str(rest);
    Ok(out)
}

fn skip_ws_and_jsonc_comments(bytes: &[u8], mut i: usize) -> Option<usize> {
    while i < bytes.len() {
        match bytes[i] {
            b' ' | b'\t' | b'\n' | b'\r' => {
                i += 1;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                i += 2;
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                i += 2;
                while i + 1 < bytes.len() {
                    if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
            }
            _ => return Some(i),
        }
    }

    None
}

fn parse_json_field_stamp(content: &str) -> Option<StampMeta> {
    let (_, meta) = strip_json_field_stamp(content)?;
    Some(meta)
}

fn strip_json_field_stamp(content: &str) -> Option<(String, StampMeta)> {
    // Very small JSON scanner: find a top-level `"x_generated"` field and remove it.
    let bytes = content.as_bytes();
    let mut i = 0usize;

    // Find opening '{'
    while i < bytes.len() && (bytes[i] as char).is_whitespace() {
        i += 1;
    }
    if i >= bytes.len() || bytes[i] != b'{' {
        return None;
    }

    let mut depth = 0i32;
    let mut in_str = false;
    let mut esc = false;

    let mut key_start: Option<usize> = None;
    let mut key_end: Option<usize> = None;

    for idx in i..bytes.len() {
        let c = bytes[idx] as char;

        if in_str {
            if esc {
                esc = false;
                continue;
            }
            if c == '\\' {
                esc = true;
                continue;
            }
            if c == '"' {
                in_str = false;
                if depth == 1 {
                    // potential key
                    if let Some(ks) = key_start {
                        key_end = Some(idx + 1);
                        let key = &content[ks..idx + 1];
                        if key == format!("\"{}\"", JSON_STAMP_FIELD) {
                            // keep key_end; we will validate ':' etc below
                            break;
                        }
                    }
                }
                key_start = None;
            }
            continue;
        }

        match c {
            '"' => {
                in_str = true;
                if depth == 1 {
                    key_start = Some(idx);
                }
            }
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }
    }

    let ks = key_start?;
    let ke = key_end?;

    // Validate that this string is used as a key (followed by optional ws + ':')
    let mut j = ke;
    while j < bytes.len() && (bytes[j] as char).is_whitespace() {
        j += 1;
    }
    if j >= bytes.len() || bytes[j] != b':' {
        return None;
    }

    // Parse value bounds (assume JSON object for our stamp)
    j += 1;
    while j < bytes.len() && (bytes[j] as char).is_whitespace() {
        j += 1;
    }
    if j >= bytes.len() || bytes[j] != b'{' {
        return None;
    }

    let val_start = j;
    let mut k = j;
    depth = 0;
    in_str = false;
    esc = false;
    for idx in j..bytes.len() {
        let c = bytes[idx] as char;
        if in_str {
            if esc {
                esc = false;
                continue;
            }
            if c == '\\' {
                esc = true;
                continue;
            }
            if c == '"' {
                in_str = false;
            }
            continue;
        }

        match c {
            '"' => in_str = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    k = idx + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    let val_end = k;

    let meta_json = &content[val_start..val_end];
    let meta = serde_json::from_str::<StampMeta>(meta_json).ok()?;

    // Remove the whole `"x_generated": {..}` property including any surrounding comma.
    // Find start (include preceding comma if present)
    let mut rm_start = ks;
    while rm_start > 0 {
        let prev = bytes[rm_start - 1] as char;
        if prev == ',' {
            rm_start -= 1;
            break;
        }
        if !prev.is_whitespace() {
            break;
        }
        rm_start -= 1;
    }

    // Find end (include trailing comma if this was first property)
    let mut rm_end = val_end;
    while rm_end < bytes.len() {
        let c = bytes[rm_end] as char;
        if c == ',' {
            rm_end += 1;
            break;
        }
        if !c.is_whitespace() {
            break;
        }
        rm_end += 1;
    }

    let mut out = String::new();
    out.push_str(&content[..rm_start]);
    out.push_str(&content[rm_end..]);

    Some((out, meta))
}
