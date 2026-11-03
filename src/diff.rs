//! The diff command's document side (BP-13 to
//! BP-15, BP-24): load a proposal file, shape-check
//! every field the renderer touches, check its
//! target against the rules file at hand, and
//! render the diff for a human. The diff applies
//! nothing.

use sha2::Digest;

use crate::failure::Failure;
use crate::inventory::Rule;
use crate::observations::decode_utf8_sig;
use crate::util::Msg;
use crate::value::{Text, Value};

const KINDS: [&str; 3] = ["delete", "rewrite", "promote"];
const EVIDENCE_KINDS: [&str; 3] = ["misreading", "ignored", "followed"];

fn get_str<'a>(value: &'a Value, key: &str) -> Option<&'a Text> {
    match value.get(key) {
        Some(Value::Str(text)) => Some(text),
        _ => None,
    }
}

fn get_section(value: &Value, key: &str) -> Option<Vec<Text>> {
    match value.get(key) {
        Some(Value::Array(items)) => {
            let mut section = Vec::new();
            for item in items {
                match item {
                    Value::Str(name) => section.push(name.clone()),
                    _ => return None,
                }
            }
            Some(section)
        }
        _ => None,
    }
}

fn kind_of(proposal: &Value) -> Option<&'static str> {
    let kind = get_str(proposal, "kind")?;
    KINDS
        .iter()
        .find(|known| kind == &Text::from_str(known))
        .copied()
}

fn named_with_id(prefix: &str, rule_id: &Text, suffix: &str) -> Failure {
    let mut message = Msg::str(prefix);
    message.push_text(rule_id);
    message.push_str(suffix);
    Failure::named_msg(message)
}

fn check_proposal(proposal: &Value) -> Result<(), Failure> {
    if !matches!(proposal, Value::Object(_)) {
        return Err(Failure::named("a proposal is not an object"));
    }
    let rule_id = match get_str(proposal, "rule_id") {
        Some(id)
            if !id.units().is_empty()
                && !id.units().contains(&0x0A)
                && !id.units().contains(&0x0D) =>
        {
            id.clone()
        }
        _ => return Err(Failure::named("a proposal names no rule id")),
    };
    if get_str(proposal, "rule_text").is_none() {
        return Err(named_with_id(
            "proposal for ",
            &rule_id,
            " carries no rule text",
        ));
    }
    let kind = match kind_of(proposal) {
        Some(kind) => kind,
        None => {
            let kind = proposal.get("kind").cloned().unwrap_or(Value::Null);
            let mut message = Msg::str("proposal for ");
            message.push_text(&rule_id);
            message.push_str(" has unknown kind ");
            message.push_str(&crate::pyrepr::repr_value(&kind));
            return Err(Failure::named_msg(message));
        }
    };
    let evidence = match proposal.get("evidence") {
        Some(Value::Array(items)) => items,
        _ => {
            return Err(named_with_id(
                "proposal for ",
                &rule_id,
                " carries no evidence list",
            ))
        }
    };
    for item in evidence {
        let shaped = matches!(item, Value::Object(_))
            && matches!(get_str(item, "kind"), Some(kind)
                if EVIDENCE_KINDS.iter().any(|known| kind == &Text::from_str(known)))
            && get_str(item, "quote").is_some();
        if !shaped {
            return Err(named_with_id(
                "proposal for ",
                &rule_id,
                " cites evidence that is not a quoted observation",
            ));
        }
    }
    let hunk = match proposal.get("hunk") {
        Some(hunk @ Value::Object(_)) => hunk,
        _ => return Err(named_with_id("proposal for ", &rule_id, " carries no hunk")),
    };
    let shaped = match kind {
        "delete" => {
            get_str(hunk, "action") == Some(&Text::from_str("delete"))
                && get_section(hunk, "section").is_some()
                && get_str(hunk, "text").is_some()
        }
        "rewrite" => {
            get_str(hunk, "action") == Some(&Text::from_str("rewrite"))
                && get_section(hunk, "section").is_some()
                && get_str(hunk, "old").is_some()
                && get_str(hunk, "new").is_some()
                && get_str(proposal, "replacement").is_some()
        }
        _ => {
            get_str(hunk, "action") == Some(&Text::from_str("move"))
                && get_section(hunk, "from_section").is_some()
                && get_section(hunk, "to_section").is_some()
                && get_str(hunk, "text").is_some()
        }
    };
    if !shaped {
        return Err(named_with_id(
            "proposal for ",
            &rule_id,
            " carries a hunk of the wrong shape",
        ));
    }
    Ok(())
}

/// BP-13a: the document the renderer stands over.
fn check_shape(document: &Value) -> Result<(), Failure> {
    if !matches!(document, Value::Object(_)) {
        return Err(Failure::Crash);
    }
    let target = match document.get("target") {
        Some(target @ Value::Object(_)) => target,
        _ => return Err(Failure::named("the proposal file names no target")),
    };
    let sha_ok = match get_str(target, "sha256") {
        Some(sha) => {
            crate::util::scalar_count(sha.units()) == 64
                && crate::util::scalars_of(sha.units())
                    .iter()
                    .all(|scalar| matches!(scalar, 0x30..=0x39 | 0x61..=0x66))
        }
        None => false,
    };
    if !sha_ok {
        return Err(Failure::named("the proposal file names no target sha256"));
    }
    let inventory = match document.get("inventory") {
        Some(Value::Array(items)) => items,
        _ => return Err(Failure::Crash),
    };
    for entry in inventory {
        let shaped = matches!(entry, Value::Object(_))
            && get_str(entry, "id").is_some()
            && get_str(entry, "text").is_some()
            && get_section(entry, "section").is_some();
        if !shaped {
            return Err(Failure::named("an inventory entry is not a rule"));
        }
        let span_ok = match entry.get("line_span") {
            Some(Value::Array(parts)) => {
                parts.len() == 2 && parts.iter().all(|part| matches!(part, Value::Int(_)))
            }
            _ => false,
        };
        if !span_ok {
            return Err(Failure::named("an inventory entry carries no line span"));
        }
    }
    let proposals = match document.get("proposals") {
        Some(Value::Array(items)) => items,
        _ => return Err(Failure::Crash),
    };
    for proposal in proposals {
        check_proposal(proposal)?;
    }
    Ok(())
}

/// Decode and shape-check a proposal file (BP-13).
pub fn load_proposals(data: &[u8]) -> Result<Value, Failure> {
    let text = match decode_utf8_sig(data) {
        Some(text) => text,
        None => return Err(Failure::named("the proposal file is not utf-8")),
    };
    let scalars: Vec<u32> = text.chars().map(|ch| ch as u32).collect();
    let document = match crate::json::parse(&scalars) {
        Ok(document) => document,
        Err(_) => return Err(Failure::named("the proposal file is not JSON")),
    };
    let shaped = matches!(document, Value::Object(_))
        && matches!(document.get("proposals"), Some(Value::Array(_)))
        && matches!(document.get("inventory"), Some(Value::Array(_)))
        && matches!(document.get("target"), Some(Value::Object(_)));
    if !shaped {
        return Err(Failure::named(
            "the proposal file is not a proposal document",
        ));
    }
    check_shape(&document)?;
    Ok(document)
}

/// BP-13: the proposal file must name this rules
/// file by the SHA-256 of its bytes.
pub fn check_target(rules_bytes: &[u8], document: &Value) -> Result<(), Failure> {
    let digest = sha2::Sha256::digest(rules_bytes);
    let sha: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    let named = document
        .get("target")
        .and_then(|target| get_str(target, "sha256"));
    if named != Some(&Text::from_str(&sha)) {
        return Err(Failure::named(
            "the proposal file names a different rules file",
        ));
    }
    Ok(())
}

fn check_hunk(proposal: &Value, entry: &Value, rules: &[Rule]) -> Result<(), Failure> {
    let kind = kind_of(proposal).expect("a shaped kind");
    let rule_id = get_str(proposal, "rule_id").expect("a shaped id").clone();
    let rule_text = get_str(proposal, "rule_text").expect("a shaped text");
    let hunk = proposal.get("hunk").expect("a shaped hunk");
    let entry_section = get_section(entry, "section").expect("a shaped section");
    let agree = match kind {
        "delete" => {
            get_section(hunk, "section") == Some(entry_section)
                && get_str(hunk, "text") == Some(rule_text)
        }
        "rewrite" => {
            get_section(hunk, "section") == Some(entry_section)
                && get_str(hunk, "old") == Some(rule_text)
                && get_str(hunk, "new") == get_str(proposal, "replacement")
        }
        _ => {
            let first_section = rules.first().map(|rule| rule.section.clone());
            get_section(hunk, "from_section") == Some(entry_section)
                && get_str(hunk, "text") == Some(rule_text)
                && match &first_section {
                    Some(first) => get_section(hunk, "to_section").as_ref() == Some(first),
                    None => false,
                }
        }
    };
    if !agree {
        return Err(named_with_id(
            "proposal for ",
            &rule_id,
            " carries a hunk that does not agree with its rule",
        ));
    }
    Ok(())
}

/// A span integer as an index quantity: Python ints
/// are unbounded, so a literal past i128 reads as
/// its sign's infinity for slicing.
fn span_value(value: &Value) -> i128 {
    match value {
        Value::Int(digits) => digits.parse::<i128>().unwrap_or_else(|_| {
            if digits.starts_with('-') {
                i128::MIN
            } else {
                i128::MAX
            }
        }),
        _ => 0,
    }
}

/// Python's slice of the lines for a span.
fn slice_lines<'a>(lines: &'a [Text], span: &[Value]) -> Vec<&'a Text> {
    let total = lines.len() as i128;
    let mut start = span_value(&span[0]) - 1;
    let mut end = span_value(&span[1]);
    if start < 0 {
        start += total;
        if start < 0 {
            start = 0;
        }
    } else if start > total {
        start = total;
    }
    if end < 0 {
        end += total;
        if end < 0 {
            end = 0;
        }
    } else if end > total {
        end = total;
    }
    if end < start {
        return Vec::new();
    }
    lines[start as usize..end as usize].iter().collect()
}

/// The marker prefix of a raw rule line: leading
/// whitespace, the marker, and the whitespace run
/// after it, where the line carries a marker.
fn marker_prefix(line: &Text) -> Option<Text> {
    let scalars = crate::util::scalars_of(line.units());
    let mut at = 0;
    while at < scalars.len() && crate::util::py_is_space(scalars[at]) {
        at += 1;
    }
    let marker_end = if at < scalars.len() && matches!(scalars[at], 0x2D | 0x2A | 0x2B) {
        at + 1
    } else {
        let mut end = at;
        while end < scalars.len() && crate::util::decimal_digit(scalars[end]).is_some() {
            end += 1;
        }
        if end > at && end < scalars.len() && matches!(scalars[end], 0x2E | 0x29) {
            end + 1
        } else {
            return None;
        }
    };
    if marker_end >= scalars.len() || !crate::util::py_is_space(scalars[marker_end]) {
        return None;
    }
    let mut end = marker_end;
    while end < scalars.len() && crate::util::py_is_space(scalars[end]) {
        end += 1;
    }
    let unit_at = crate::util::unit_offset(line.units(), end);
    Some(Text::from_units(line.units()[..unit_at].to_vec()))
}

fn flat(text: &Text) -> Text {
    let parts = crate::util::py_splitlines(text.units());
    let mut out: Vec<u16> = Vec::new();
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            out.push(0x20);
        }
        out.extend_from_slice(part.units());
    }
    Text::from_units(out)
}

fn join_section(section: &[Text], out: &mut Msg) {
    for (index, name) in section.iter().enumerate() {
        if index > 0 {
            out.push_str(" / ");
        }
        out.push_text(name);
    }
}

/// The diff text for one rules file and one
/// proposal file, cross-checked before anything
/// renders (BP-14).
pub fn render_diff(lines: &[Text], rules: &[Rule], document: &Value) -> Result<Text, Failure> {
    check_shape(document)?;
    crate::proposals::validate_proposals(document)?;
    let target = get_str(document.get("target").expect("a shaped target"), "sha256")
        .expect("a shaped sha")
        .clone();
    let inventory = document
        .get("inventory")
        .and_then(Value::as_array)
        .expect("a shaped inventory");
    let proposals = document
        .get("proposals")
        .and_then(Value::as_array)
        .expect("shaped proposals");
    let mut counts = [0i64; 3];
    let mut blocks: Vec<Msg> = Vec::new();
    for proposal in proposals {
        let kind = kind_of(proposal).expect("a shaped kind");
        counts[KINDS
            .iter()
            .position(|known| *known == kind)
            .expect("a kind")] += 1;
        let rule_id = get_str(proposal, "rule_id").expect("a shaped id").clone();
        let rule_text = get_str(proposal, "rule_text").expect("a shaped text");
        let entry = inventory
            .iter()
            .rev()
            .find(|entry| get_str(entry, "id") == Some(&rule_id));
        let entry = match entry {
            Some(entry) if get_str(entry, "text") == Some(rule_text) => entry,
            _ => {
                return Err(named_with_id(
                    "proposal for ",
                    &rule_id,
                    " does not match the inventory it travels with",
                ))
            }
        };
        let rule = rules.iter().rev().find(|rule| rule.id == rule_id);
        let entry_section = get_section(entry, "section").expect("a shaped section");
        let rule = match rule {
            Some(rule) if rule.text == *rule_text && rule.section == entry_section => rule,
            _ => {
                return Err(named_with_id(
                    "proposal for ",
                    &rule_id,
                    " does not match the rules file as parsed",
                ))
            }
        };
        check_hunk(proposal, entry, rules)?;
        let span = vec![
            Value::Int(rule.line_span[0].to_string()),
            Value::Int(rule.line_span[1].to_string()),
        ];
        let raw = slice_lines(lines, &span);
        let hunk = proposal.get("hunk").expect("a shaped hunk");
        let mut block = Msg::new();
        if kind == "promote" {
            block.push_str("## promote ");
            block.push_text(&rule_id);
            block.push_str(" (");
            join_section(
                &get_section(hunk, "from_section").expect("shaped"),
                &mut block,
            );
            block.push_str(" to ");
            join_section(
                &get_section(hunk, "to_section").expect("shaped"),
                &mut block,
            );
            block.push_str(")");
        } else {
            block.push_str("## ");
            block.push_str(kind);
            block.push_str(" ");
            block.push_text(&rule_id);
            block.push_str(" (");
            join_section(&entry_section, &mut block);
            block.push_str(")");
        }
        for line in &raw {
            block.push_str("\n- ");
            block.push_text(line);
        }
        if kind == "rewrite" {
            let first = match raw.first() {
                Some(first) => *first,
                None => return Err(Failure::Crash),
            };
            let prefix = marker_prefix(first).unwrap_or_else(|| Text::from_str("- "));
            let replacement = get_str(proposal, "replacement").expect("a shaped replacement");
            let mut lines_out = crate::util::py_splitlines(replacement.units());
            if lines_out.is_empty() {
                lines_out.push(Text::from_str(""));
            }
            block.push_str("\n+ ");
            block.push_text(&prefix);
            block.push_text(&lines_out[0]);
            for line in &lines_out[1..] {
                block.push_str("\n+ ");
                block.push_text(line);
            }
        }
        let evidence = proposal
            .get("evidence")
            .and_then(Value::as_array)
            .expect("shaped evidence");
        for item in evidence {
            block.push_str("\n  evidence, ");
            block.push_text(get_str(item, "kind").expect("a shaped kind"));
            block.push_str(", \"");
            block.push_text(&flat(get_str(item, "quote").expect("a shaped quote")));
            block.push_str("\"");
        }
        blocks.push(block);
    }
    let mut out = Msg::new();
    out.push_str(&format!(
        "bluepencil diff, {} proposals (delete {}, rewrite {}, promote {})\ntarget sha256 ",
        proposals.len(),
        counts[0],
        counts[1],
        counts[2]
    ));
    out.push_text(&target);
    if blocks.is_empty() {
        out.push_str("\n");
    } else {
        for block in &blocks {
            out.push_str("\n\n");
            out.push_msg(block);
        }
        out.push_str("\n");
    }
    Ok(Text::from_units(out.0))
}
