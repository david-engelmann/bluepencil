//! Evidence enters as an observations file (BP-6,
//! BP-22). The loader reads its file as UTF-8, with
//! no BOM stripping at this surface, and a failure
//! to read, decode, or parse is reported with the
//! CPython exception text the Python embeds in its
//! named refusal.

use crate::failure::Failure;
use crate::util::Msg;
use crate::value::{Text, Value};

pub const KINDS: [&str; 3] = ["misreading", "ignored", "followed"];

/// A kind's position in the canonical KINDS order,
/// the order the proposal counts array reads in.
/// This is the one definition of the kind set's
/// order; validation and counting both read it, so
/// the two cannot drift apart.
pub fn kind_index(kind: &Text) -> Option<usize> {
    KINDS
        .iter()
        .position(|known| kind == &Text::from_str(known))
}

/// One observation, in the BP-6 shape: the rule's
/// exact text, a kind, and a quote. Unheard consumes
/// this shape, so its fields do not drift.
#[derive(Clone, Debug, PartialEq)]
pub struct Observation {
    pub rule: Text,
    pub kind: Text,
    pub quote: Text,
}

/// One supplied rewrite: the rule it answers, the
/// misreading quote it rests on, and its replacement.
#[derive(Clone, Debug, PartialEq)]
pub struct Rewrite {
    pub rule: Text,
    pub misreading_quote: Text,
    pub replacement: Text,
}

impl Observation {
    /// The observation as a JSON value, in the
    /// document's key order.
    pub fn to_value(&self) -> Value {
        Value::Object(vec![
            (Text::from_str("rule"), Value::Str(self.rule.clone())),
            (Text::from_str("kind"), Value::Str(self.kind.clone())),
            (Text::from_str("quote"), Value::Str(self.quote.clone())),
        ])
    }
}

/// A UTF-8 decoding failure in CPython's terms:
/// where it starts, where its span ends, and its
/// reason.
struct DecodeFailure {
    start: usize,
    end: usize,
    reason: &'static str,
    byte: u8,
}

impl DecodeFailure {
    fn describe(&self) -> String {
        if self.end - self.start == 1 {
            format!(
                "'utf-8' codec can't decode byte 0x{:02x} in position {}: {}",
                self.byte, self.start, self.reason
            )
        } else {
            format!(
                "'utf-8' codec can't decode bytes in position {}-{}: {}",
                self.start,
                self.end - 1,
                self.reason
            )
        }
    }
}

/// Decode bytes as CPython's UTF-8 decoder does,
/// reporting the first failure in its own terms.
fn decode_utf8(bytes: &[u8]) -> Result<String, DecodeFailure> {
    let mut out = String::new();
    let mut at = 0;
    while at < bytes.len() {
        let first = bytes[at];
        if first < 0x80 {
            out.push(first as char);
            at += 1;
            continue;
        }
        let (len, second_lo, second_hi): (usize, u8, u8) = match first {
            0xC2..=0xDF => (2, 0x80, 0xBF),
            0xE0 => (3, 0xA0, 0xBF),
            0xE1..=0xEC | 0xEE..=0xEF => (3, 0x80, 0xBF),
            0xED => (3, 0x80, 0x9F),
            0xF0 => (4, 0x90, 0xBF),
            0xF1..=0xF3 => (4, 0x80, 0xBF),
            0xF4 => (4, 0x80, 0x8F),
            _ => {
                return Err(DecodeFailure {
                    start: at,
                    end: at + 1,
                    reason: "invalid start byte",
                    byte: first,
                })
            }
        };
        if at + 1 >= bytes.len() {
            return Err(DecodeFailure {
                start: at,
                end: bytes.len(),
                reason: "unexpected end of data",
                byte: first,
            });
        }
        let second = bytes[at + 1];
        if !(second_lo..=second_hi).contains(&second) {
            return Err(DecodeFailure {
                start: at,
                end: at + 1,
                reason: "invalid continuation byte",
                byte: first,
            });
        }
        let mut scalar = (match len {
            2 => first & 0x1F,
            3 => first & 0x0F,
            _ => first & 0x07,
        }) as u32;
        scalar = (scalar << 6) | (second as u32 & 0x3F);
        for offset in 2..len {
            if at + offset >= bytes.len() {
                return Err(DecodeFailure {
                    start: at,
                    end: bytes.len(),
                    reason: "unexpected end of data",
                    byte: first,
                });
            }
            let next = bytes[at + offset];
            if !(0x80..=0xBF).contains(&next) {
                return Err(DecodeFailure {
                    start: at,
                    end: at + offset,
                    reason: "invalid continuation byte",
                    byte: first,
                });
            }
            scalar = scalar << 6 | (next as u32 & 0x3F);
        }
        out.push(char::from_u32(scalar).expect("a decoded scalar"));
        at += len;
    }
    Ok(out)
}

/// Decode bytes as utf-8-sig: one leading byte
/// order mark is stripped, the rest is strict
/// UTF-8. Used by the rules and proposal surfaces,
/// which refuse by name rather than by detail.
pub fn decode_utf8_sig(bytes: &[u8]) -> Option<String> {
    let body = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &bytes[3..]
    } else {
        bytes
    };
    String::from_utf8(body.to_vec()).ok()
}

/// Universal-newline translation over the decoded
/// text, matching the interpreter's text reading.
fn translate_newlines(scalars: &[u32]) -> Vec<u32> {
    let mut out = Vec::with_capacity(scalars.len());
    let mut index = 0;
    while index < scalars.len() {
        if scalars[index] == 0x0D {
            out.push(0x0A);
            if scalars.get(index + 1) == Some(&0x0A) {
                index += 1;
            }
        } else {
            out.push(scalars[index]);
        }
        index += 1;
    }
    out
}

fn cannot_read(path: &std::path::Path, detail: &str) -> Failure {
    let display = crate::util::fsdecode_units(&crate::util::os_bytes(path.as_os_str()));
    let mut message = Msg::str("cannot read observations ");
    message.push_units(&display);
    message.push_str(": ");
    message.push_str(detail);
    Failure::named_msg(message)
}

fn os_error_detail(path: &std::path::Path, error: &std::io::Error) -> String {
    let display = crate::util::fsdecode_units(&crate::util::os_bytes(path.as_os_str()));
    let repr = crate::pyrepr::repr_text(&Text::from_units(display));
    match error.raw_os_error() {
        Some(errno) => format!(
            "[Errno {errno}] {}: {repr}",
            crate::util::strerror_text(error)
        ),
        None => format!("{error}: {repr}"),
    }
}

fn text_field(entry: &Value, field: &str, label: &str) -> Result<Text, Failure> {
    match entry.get(field) {
        Some(Value::Str(text)) if !text.units().is_empty() => Ok(text.clone()),
        _ => Err(Failure::named(&format!("{label} has no {field}"))),
    }
}

fn entries<'a>(document: &'a Value, key: &str) -> Result<&'a Vec<Value>, Failure> {
    match document.get(key) {
        None => Ok(&EMPTY_LIST),
        Some(Value::Array(items)) => Ok(items),
        _ => Err(Failure::named(&format!("{key} must be a list"))),
    }
}

static EMPTY_LIST: Vec<Value> = Vec::new();

/// Load an observations file: (observations,
/// supplied rewrites), every entry validated by
/// name in document order.
pub fn load_observations(
    path: &std::path::Path,
) -> Result<(Vec<Observation>, Vec<Rewrite>), Failure> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => return Err(cannot_read(path, &os_error_detail(path, &error))),
    };
    let text = match decode_utf8(&bytes) {
        Ok(text) => text,
        Err(failure) => return Err(cannot_read(path, &failure.describe())),
    };
    let scalars: Vec<u32> = text.chars().map(|ch| ch as u32).collect();
    // read_text applies universal-newline
    // translation before json.loads runs: a CR LF
    // pair and a lone CR both become LF, and the
    // positions the interpreter reports, in its
    // syntax errors above all, are the translated
    // text's positions.
    let scalars = translate_newlines(&scalars);
    let document = match crate::json::parse(&scalars) {
        Ok(document) => document,
        Err(error) => return Err(cannot_read(path, &error.describe(&scalars))),
    };
    let observations_ok = matches!(document, Value::Object(_))
        && matches!(document.get("observations"), Some(Value::Array(_)));
    if !observations_ok {
        return Err(Failure::named(
            "observations file must be a JSON object with an observations list",
        ));
    }
    let mut observations = Vec::new();
    for (index, entry) in entries(&document, "observations")?.iter().enumerate() {
        if !matches!(entry, Value::Object(_)) {
            return Err(Failure::named(&format!(
                "observation {index} is not an object"
            )));
        }
        let label = format!("observation {index}");
        let kind = entry.get("kind").cloned().unwrap_or(Value::Null);
        let kind_text = match &kind {
            Value::Str(text) if KINDS.iter().any(|known| text == &Text::from_str(known)) => {
                text.clone()
            }
            _ => {
                return Err(Failure::named(&format!(
                    "{label} has unknown kind {}",
                    crate::pyrepr::repr_value(&kind)
                )))
            }
        };
        observations.push(Observation {
            rule: text_field(entry, "rule", &label)?,
            kind: kind_text,
            quote: text_field(entry, "quote", &label)?,
        });
    }
    let mut rewrites = Vec::new();
    for (index, entry) in entries(&document, "rewrites")?.iter().enumerate() {
        if !matches!(entry, Value::Object(_)) {
            return Err(Failure::named(&format!("rewrite {index} is not an object")));
        }
        let label = format!("rewrite {index}");
        rewrites.push(Rewrite {
            rule: text_field(entry, "rule", &label)?,
            misreading_quote: text_field(entry, "misreading_quote", &label)?,
            replacement: text_field(entry, "replacement", &label)?,
        });
    }
    Ok((observations, rewrites))
}
