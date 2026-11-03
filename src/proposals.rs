//! Join evidence to rules, generate and validate
//! proposals, assemble the proposal file (BP-7 to
//! BP-11, BP-23).

use sha2::Digest;

use crate::failure::Failure;
use crate::inventory::Rule;
use crate::observations::{Observation, Rewrite};
use crate::util::Msg;
use crate::value::{Text, Value};

pub const COUNTING_METHOD: &str = "counts are observations per rule by kind, matched by \
exact rule text; a rule edited since an observation was taken is a new rule and the \
observation is unmatched; an observation naming a text carried by two rules attaches \
to the first, and the later rule is flagged a duplicate";

fn kind_index(kind: &Text) -> usize {
    crate::observations::kind_index(kind).expect("an observation kind is one of the three")
}

fn section_value(section: &[Text]) -> Value {
    Value::Array(
        section
            .iter()
            .map(|name| Value::Str(name.clone()))
            .collect(),
    )
}

fn int(value: i64) -> Value {
    Value::Int(value.to_string())
}

/// A rule joined with its observations and counts.
pub struct Joined {
    pub id: Text,
    pub section: Vec<Text>,
    pub text: Text,
    pub line_span: [i64; 2],
    pub observations: Vec<Observation>,
    /// [misreading, ignored, followed, exercised]
    pub counts: [i64; 4],
    pub duplicate_of: Option<Text>,
}

/// Attach observations to rules by exact text
/// (BP-6): an observation attaches to the first
/// rule carrying its text, and a later rule with
/// the same text is flagged a duplicate.
pub fn join(rules: &[Rule], observations: &[Observation]) -> (Vec<Joined>, Vec<Observation>) {
    let mut first_position: Vec<(&Text, usize)> = Vec::new();
    for (position, rule) in rules.iter().enumerate() {
        if !first_position.iter().any(|(text, _)| *text == &rule.text) {
            first_position.push((&rule.text, position));
        }
    }
    let mut joined: Vec<Joined> = Vec::new();
    let mut seen_texts: Vec<(&Text, &Text)> = Vec::new();
    for rule in rules {
        let duplicate_of = match seen_texts.iter().find(|(text, _)| *text == &rule.text) {
            Some((_, id)) => Some((*id).clone()),
            None => {
                seen_texts.push((&rule.text, &rule.id));
                None
            }
        };
        joined.push(Joined {
            id: rule.id.clone(),
            section: rule.section.clone(),
            text: rule.text.clone(),
            line_span: rule.line_span,
            observations: Vec::new(),
            counts: [0; 4],
            duplicate_of,
        });
    }
    let mut unmatched = Vec::new();
    for observation in observations {
        match first_position
            .iter()
            .find(|(text, _)| *text == &observation.rule)
        {
            Some((_, position)) => {
                let entry = &mut joined[*position];
                entry.counts[kind_index(&observation.kind)] += 1;
                entry.counts[3] += 1;
                entry.observations.push(observation.clone());
            }
            None => unmatched.push(observation.clone()),
        }
    }
    (joined, unmatched)
}

fn evidence_values(observations: &[Observation]) -> Value {
    Value::Array(observations.iter().map(Observation::to_value).collect())
}

fn filtered<'a>(entry: &'a Joined, kind: &str) -> Vec<&'a Observation> {
    let wanted = Text::from_str(kind);
    entry
        .observations
        .iter()
        .filter(|observation| observation.kind == wanted)
        .collect()
}

fn proposal(kind: &str, entry: &Joined, fields: Vec<(Text, Value)>) -> Value {
    let mut entries = vec![
        (Text::from_str("kind"), Value::Str(Text::from_str(kind))),
        (Text::from_str("rule_id"), Value::Str(entry.id.clone())),
        (Text::from_str("rule_text"), Value::Str(entry.text.clone())),
    ];
    entries.extend(fields);
    Value::Object(entries)
}

/// Mechanical generation (BP-10) plus supplied
/// rewrites carried and validated (BP-9), in the
/// Python's order: per rule, its delete, then its
/// rewrites in document order, then its promote.
pub fn generate(joined: &[Joined], rewrites: &[Rewrite]) -> Result<Vec<Value>, Failure> {
    let mut proposals = Vec::new();
    let total = joined.len();
    let first_section: Option<&Vec<Text>> = joined.first().map(|entry| &entry.section);
    let mut seen_rewrite_texts: Vec<&Text> = Vec::new();
    for (index, entry) in joined.iter().enumerate() {
        if entry.counts[1] >= 2 && entry.counts[2] == 0 {
            proposals.push(proposal(
                "delete",
                entry,
                vec![
                    (
                        Text::from_str("evidence"),
                        evidence_values(
                            &filtered(entry, "ignored")
                                .into_iter()
                                .cloned()
                                .collect::<Vec<_>>(),
                        ),
                    ),
                    (
                        Text::from_str("hunk"),
                        Value::Object(vec![
                            (
                                Text::from_str("action"),
                                Value::Str(Text::from_str("delete")),
                            ),
                            (Text::from_str("section"), section_value(&entry.section)),
                            (Text::from_str("text"), Value::Str(entry.text.clone())),
                        ]),
                    ),
                ],
            ));
        }
        if !seen_rewrite_texts.contains(&&entry.text) {
            for rewrite in rewrites.iter().filter(|rewrite| rewrite.rule == entry.text) {
                let matching: Vec<Observation> = entry
                    .observations
                    .iter()
                    .filter(|observation| {
                        observation.kind == Text::from_str("misreading")
                            && observation.quote == rewrite.misreading_quote
                    })
                    .cloned()
                    .collect();
                if matching.is_empty() {
                    let mut message = Msg::str("rewrite for ");
                    message.push_text(&entry.id);
                    message.push_str(" quotes a misreading that is not among its observations");
                    return Err(Failure::named_msg(message));
                }
                proposals.push(proposal(
                    "rewrite",
                    entry,
                    vec![
                        (Text::from_str("evidence"), evidence_values(&matching)),
                        (
                            Text::from_str("misreading_quote"),
                            Value::Str(rewrite.misreading_quote.clone()),
                        ),
                        (
                            Text::from_str("replacement"),
                            Value::Str(rewrite.replacement.clone()),
                        ),
                        (
                            Text::from_str("hunk"),
                            Value::Object(vec![
                                (
                                    Text::from_str("action"),
                                    Value::Str(Text::from_str("rewrite")),
                                ),
                                (Text::from_str("section"), section_value(&entry.section)),
                                (Text::from_str("old"), Value::Str(entry.text.clone())),
                                (
                                    Text::from_str("new"),
                                    Value::Str(rewrite.replacement.clone()),
                                ),
                            ]),
                        ),
                    ],
                ));
            }
        }
        seen_rewrite_texts.push(&entry.text);
        if 2 * index >= total && entry.counts[2] >= 2 && Some(&entry.section) != first_section {
            proposals.push(proposal(
                "promote",
                entry,
                vec![
                    (
                        Text::from_str("evidence"),
                        evidence_values(
                            &filtered(entry, "followed")
                                .into_iter()
                                .cloned()
                                .collect::<Vec<_>>(),
                        ),
                    ),
                    (
                        Text::from_str("hunk"),
                        Value::Object(vec![
                            (Text::from_str("action"), Value::Str(Text::from_str("move"))),
                            (
                                Text::from_str("from_section"),
                                section_value(&entry.section),
                            ),
                            (
                                Text::from_str("to_section"),
                                section_value(first_section.expect("a first rule stands")),
                            ),
                            (Text::from_str("text"), Value::Str(entry.text.clone())),
                        ]),
                    ),
                ],
            ));
        }
    }
    for rewrite in rewrites {
        if !joined.iter().any(|entry| entry.text == rewrite.rule) {
            let truncated = crate::util::truncate_text(&rewrite.rule, 60);
            let mut message = Msg::str("supplied rewrite names a rule that is not in the file: ");
            message.push_str(&crate::pyrepr::repr_text(&truncated));
            return Err(Failure::named_msg(message));
        }
    }
    Ok(proposals)
}

/// The Python's str of a value, for the phrases
/// that interpolate one raw: a string stands as
/// itself, anything else as its repr, and a missing
/// value as None.
fn py_str(value: Option<&Value>) -> Msg {
    match value {
        None => Msg::str("None"),
        Some(Value::Str(text)) => {
            let mut message = Msg::new();
            message.push_text(text);
            message
        }
        Some(other) => Msg::str(&crate::pyrepr::repr_value(other)),
    }
}

fn truthy(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) => false,
        Some(Value::Bool(flag)) => *flag,
        Some(Value::Int(digits)) => digits.bytes().any(|byte| (b'1'..=b'9').contains(&byte)),
        Some(Value::Float(number)) => *number != 0.0,
        Some(Value::Str(text)) => !text.units().is_empty(),
        Some(Value::Array(items)) => !items.is_empty(),
        Some(Value::Object(entries)) => !entries.is_empty(),
    }
}

/// BP-8: every proposal rests on at least one
/// quoted observation, and the target names the
/// rules file.
pub fn validate_proposals(document: &Value) -> Result<(), Failure> {
    if !matches!(document, Value::Object(_)) {
        return Err(Failure::Crash);
    }
    let sha_len_ok = match document.get("target") {
        None => false,
        Some(target @ Value::Object(_)) => match target.get("sha256") {
            None => false,
            Some(Value::Str(text)) => crate::util::scalar_count(text.units()) == 64,
            Some(Value::Array(items)) => items.len() == 64,
            Some(Value::Object(entries)) => entries.len() == 64,
            Some(_) => return Err(Failure::Crash),
        },
        Some(_) => return Err(Failure::Crash),
    };
    if !sha_len_ok {
        return Err(Failure::named("proposal file names no target sha256"));
    }
    let proposals = match document.get("proposals") {
        Some(Value::Array(items)) => items,
        _ => return Err(Failure::Crash),
    };
    for item in proposals {
        if !matches!(item, Value::Object(_)) {
            return Err(Failure::Crash);
        }
        let rule_id = py_str(item.get("rule_id"));
        let kind = item.get("kind").cloned().unwrap_or(Value::Null);
        let kind_ok = matches!(&kind, Value::Str(text)
            if text == &Text::from_str("delete")
                || text == &Text::from_str("rewrite")
                || text == &Text::from_str("promote"));
        if !kind_ok {
            let mut message = Msg::str("proposal for ");
            message.push_msg(&rule_id);
            message.push_str(" has unknown kind ");
            message.push_str(&crate::pyrepr::repr_value(&kind));
            return Err(Failure::named_msg(message));
        }
        if !truthy(item.get("evidence")) {
            let mut message = Msg::str("proposal for ");
            message.push_msg(&rule_id);
            message.push_str(" rests on no observation");
            return Err(Failure::named_msg(message));
        }
        let evidence = match item.get("evidence") {
            Some(Value::Array(items)) => items,
            _ => return Err(Failure::Crash),
        };
        for cited in evidence {
            if !matches!(cited, Value::Object(_)) {
                return Err(Failure::Crash);
            }
            if !truthy(cited.get("quote")) {
                let mut message = Msg::str("proposal for ");
                message.push_msg(&rule_id);
                message.push_str(" cites evidence with no quote");
                return Err(Failure::named_msg(message));
            }
        }
    }
    Ok(())
}

/// Assemble the proposal document, in its field
/// order, and validate it as the Python does.
pub fn build_document(
    rules_bytes: &[u8],
    rules: &[Rule],
    observations: &[Observation],
    rewrites: &[Rewrite],
    warnings: &[Text],
) -> Result<Value, Failure> {
    let (joined, unmatched) = join(rules, observations);
    let proposals = generate(&joined, rewrites)?;
    let digest = sha2::Sha256::digest(rules_bytes);
    let sha: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    let inventory: Vec<Value> = joined
        .iter()
        .map(|entry| {
            let mut fields = vec![
                (Text::from_str("id"), Value::Str(entry.id.clone())),
                (Text::from_str("section"), section_value(&entry.section)),
                (Text::from_str("text"), Value::Str(entry.text.clone())),
                (
                    Text::from_str("line_span"),
                    Value::Array(vec![int(entry.line_span[0]), int(entry.line_span[1])]),
                ),
                (
                    Text::from_str("counts"),
                    Value::Object(vec![
                        (Text::from_str("misreading"), int(entry.counts[0])),
                        (Text::from_str("ignored"), int(entry.counts[1])),
                        (Text::from_str("followed"), int(entry.counts[2])),
                        (Text::from_str("exercised"), int(entry.counts[3])),
                    ]),
                ),
            ];
            if let Some(duplicate) = &entry.duplicate_of {
                fields.push((
                    Text::from_str("duplicate_of"),
                    Value::Str(duplicate.clone()),
                ));
            }
            Value::Object(fields)
        })
        .collect();
    let document = Value::Object(vec![
        (
            Text::from_str("proposal_version"),
            Value::Str(Text::from_str("1.0")),
        ),
        (
            Text::from_str("target"),
            Value::Object(vec![(
                Text::from_str("sha256"),
                Value::Str(Text::from_str(&sha)),
            )]),
        ),
        (
            Text::from_str("counting_method"),
            Value::Str(Text::from_str(COUNTING_METHOD)),
        ),
        (Text::from_str("inventory"), Value::Array(inventory)),
        (
            Text::from_str("unmatched_observations"),
            evidence_values(&unmatched),
        ),
        (
            Text::from_str("warnings"),
            Value::Array(
                warnings
                    .iter()
                    .map(|warning| Value::Str(warning.clone()))
                    .collect(),
            ),
        ),
        (Text::from_str("proposals"), Value::Array(proposals)),
    ]);
    validate_proposals(&document)?;
    Ok(document)
}
