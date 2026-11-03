//! BP-5, BP-5a, BP-6, BP-7. Inventory and evidence
//! join, ported from the Python suite file for
//! file.

use std::path::PathBuf;

use bluepencil::failure::Failure;
use bluepencil::inventory::{parse_rules, Rule};
use bluepencil::observations::{load_observations, Observation};
use bluepencil::proposals::build_document;
use bluepencil::value::{Text, Value};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn rules_bytes() -> Vec<u8> {
    std::fs::read(fixture("rules_synthetic.md")).expect("the rules fixture")
}

fn rules_text(bytes: &[u8]) -> Text {
    Text::from_str(&bluepencil::observations::decode_utf8_sig(bytes).expect("fixture is utf-8"))
}

fn parse(text: &str) -> (Vec<Rule>, Vec<Text>) {
    parse_rules(&Text::from_str(text)).expect("parses")
}

fn texts(rules: &[Rule]) -> Vec<String> {
    rules
        .iter()
        .map(|rule| rule.text.as_str().expect("valid text"))
        .collect()
}

fn section(rule: &Rule) -> Vec<String> {
    rule.section
        .iter()
        .map(|name| name.as_str().expect("valid text"))
        .collect()
}

fn get<'a>(value: &'a Value, key: &str) -> &'a Value {
    value.get(key).expect("the field stands")
}

fn as_text(value: &Value) -> String {
    match value {
        Value::Str(text) => text.as_str().expect("valid text"),
        _ => panic!("not a string value"),
    }
}

fn as_array(value: &Value) -> &Vec<Value> {
    value.as_array().expect("an array value")
}

fn counts(document: &Value, id: &str) -> (i64, i64, i64, i64) {
    for entry in as_array(get(document, "inventory")) {
        if as_text(get(entry, "id")) == id {
            let counts = get(entry, "counts");
            let num = |key: &str| match get(counts, key) {
                Value::Int(digits) => digits.parse::<i64>().expect("a count"),
                _ => panic!("not an integer"),
            };
            return (
                num("misreading"),
                num("ignored"),
                num("followed"),
                num("exercised"),
            );
        }
    }
    panic!("no inventory entry {id}");
}

fn obs(rule: &str, kind: &str, quote: &str) -> Observation {
    Observation {
        rule: Text::from_str(rule),
        kind: Text::from_str(kind),
        quote: Text::from_str(quote),
    }
}

fn named(failure: Failure) -> String {
    match failure {
        Failure::Named(message) => message.render_stderr(),
        Failure::Crash => panic!("a crash, not a named refusal"),
    }
}

#[test]
fn every_rule_is_inventoried_with_position_section_text_span() {
    let bytes = rules_bytes();
    let (rules, _) = parse_rules(&rules_text(&bytes)).expect("parses");
    let ids: Vec<String> = rules
        .iter()
        .map(|rule| rule.id.as_str().expect("an id"))
        .collect();
    assert_eq!(
        ids,
        (1..=7).map(|i| format!("rule-{i}")).collect::<Vec<_>>()
    );
    assert_eq!(section(&rules[0]), ["Setup rules"]);
    assert_eq!(
        rules[0].text.as_str().expect("text"),
        "Always run the local gate before pushing"
    );
    assert_eq!(section(&rules[3]), ["Setup rules", "Testing"]);
    assert_eq!(rules[3].line_span, [11, 12]);
    assert_eq!(section(&rules[6]), ["Setup rules", "Style"]);
}

#[test]
fn prose_and_fenced_lines_are_never_rules() {
    let bytes = rules_bytes();
    let (rules, _) = parse_rules(&rules_text(&bytes)).expect("parses");
    let texts = texts(&rules);
    assert!(!texts
        .iter()
        .any(|text| text.contains("fossil") || text.contains("inside a fence")));
    assert!(!texts.iter().any(|text| text.contains("Read these before")));
}

#[test]
fn a_continuation_line_joins_its_rules_text() {
    let bytes = rules_bytes();
    let (rules, _) = parse_rules(&rules_text(&bytes)).expect("parses");
    assert_eq!(
        rules[3].text.as_str().expect("text"),
        "Run the full suite locally, never trust CI first\nincluding the slow integration half"
    );
}

#[test]
fn a_rule_before_any_heading_refuses_by_name() {
    let failure = parse_rules(&Text::from_str(
        "- A rule with no home\n\n# Later\n\n- Fine\n",
    ))
    .expect_err("refused");
    assert!(named(failure).contains("outside any section"));
}

#[test]
fn numbered_lines_are_rules_and_nesting_is_kept_flat_in_order() {
    let (rules, _) = parse("# S\n\n1. First\n2. Second\n   - Nested third\n");
    assert_eq!(texts(&rules), ["First", "Second", "Nested third"]);
}

#[test]
fn observations_join_by_exact_text_and_unmatched_are_reported() {
    let bytes = rules_bytes();
    let (rules, warnings) = parse_rules(&rules_text(&bytes)).expect("parses");
    let (observations, _) =
        load_observations(&fixture("observations_synthetic.json")).expect("loads");
    let document = build_document(&bytes, &rules, &observations, &[], &warnings).expect("builds");
    let unmatched = as_array(get(&document, "unmatched_observations"));
    assert_eq!(unmatched.len(), 1);
    assert!(as_text(get(&unmatched[0], "rule")).starts_with("A rule that was edited away"));
    assert_eq!(counts(&document, "rule-2"), (0, 2, 0, 2));
    assert_eq!(counts(&document, "rule-1").3, 2);
}

#[test]
fn the_counting_method_is_stated_in_the_output() {
    let bytes = rules_bytes();
    let (rules, warnings) = parse_rules(&rules_text(&bytes)).expect("parses");
    let document = build_document(&bytes, &rules, &[], &[], &warnings).expect("builds");
    assert!(as_text(get(&document, "counting_method")).contains("exact rule text"));
}

#[test]
fn a_duplicate_text_is_flagged_and_counts_attach_to_the_first() {
    let text = "# S\n\n- Same words\n- Same words\n";
    let (rules, warnings) = parse(text);
    let document = build_document(
        text.as_bytes(),
        &rules,
        &[obs("Same words", "ignored", "q")],
        &[],
        &warnings,
    )
    .expect("builds");
    let inventory = as_array(get(&document, "inventory"));
    let first_counts = get(&inventory[0], "counts");
    let second_counts = get(&inventory[1], "counts");
    assert_eq!(get(first_counts, "ignored"), &Value::Int("1".to_string()));
    assert_eq!(get(second_counts, "ignored"), &Value::Int("0".to_string()));
    assert_eq!(as_text(get(&inventory[1], "duplicate_of")), "rule-1");
}

#[test]
fn malformed_observations_refuse_by_name() {
    let dir = std::env::temp_dir().join(format!("bp-inv-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a temp dir");
    let bad = dir.join("bad.json");
    std::fs::write(
        &bad,
        "{\"observations\": [{\"rule\": \"x\", \"kind\": \"shrugged\", \"quote\": \"q\"}]}",
    )
    .expect("writes");
    let failure = load_observations(&bad).expect_err("refused");
    assert!(named(failure).contains("unknown kind"));
}

#[test]
fn bullets_inside_an_html_comment_are_not_rules() {
    let (rules, _) = parse(
        "# S\n\n<!--\n- A phantom rule from an example\n- Another phantom\n-->\n\n- A real rule\n",
    );
    assert_eq!(texts(&rules), ["A real rule"]);
}

#[test]
fn an_empty_marker_refuses_by_name() {
    let failure = parse_rules(&Text::from_str("# S\n\n-\n")).expect_err("refused");
    assert!(named(failure).contains("empty rule at line 3"));
}

#[test]
fn a_fence_closes_only_on_its_own_character_and_length() {
    let (rules, _) =
        parse("# S\n\n````\n```\n- Inside the long fence\n~~~\n````\n\n- After the fence\n");
    assert_eq!(texts(&rules), ["After the fence"]);
}

#[test]
fn an_unclosed_fence_warns_and_swallows_nothing_silently() {
    let (rules, warnings) = parse("# S\n\n- Kept\n\n```\n- Lost\n");
    assert_eq!(texts(&rules), ["Kept"]);
    assert!(!warnings.is_empty());
    assert!(warnings[0]
        .as_str()
        .expect("a warning")
        .contains("unclosed fence at line 5"));
}

#[test]
fn setext_headings_open_sections() {
    let (rules, _) =
        parse("Top section\n===========\n\n- Rule one\n\nSub section\n-----------\n\n- Rule two\n");
    assert_eq!(section(&rules[0]), ["Top section"]);
    assert_eq!(section(&rules[1]), ["Top section", "Sub section"]);
}

#[test]
fn closing_hashes_are_not_part_of_a_section_name() {
    let (rules, _) = parse("## Style ##\n\n- A rule\n");
    assert_eq!(section(&rules[0]), ["Style"]);
}

#[test]
fn a_task_list_checkbox_is_stripped_from_the_rule_text() {
    let (rules, _) = parse("# S\n\n- [ ] Do the thing\n- [x] Done thing\n");
    assert_eq!(texts(&rules), ["Do the thing", "Done thing"]);
}

#[test]
fn a_continuation_after_a_blank_line_still_joins_its_rule() {
    let (rules, _) = parse("# S\n\n- A rule that\n\n  continues here\n- Next\n");
    assert_eq!(
        rules[0].text.as_str().expect("text"),
        "A rule that\ncontinues here"
    );
    assert_eq!(rules[0].line_span, [3, 5]);
}

#[test]
fn a_near_miss_observation_is_unmatched_not_attached() {
    let (rules, warnings) = parse("# S\n\n- Keep functions short\n");
    let document = build_document(
        b"-",
        &rules,
        &[obs("keep functions short", "ignored", "lowercase differs")],
        &[],
        &warnings,
    )
    .expect("builds");
    assert_eq!(as_array(get(&document, "unmatched_observations")).len(), 1);
    let inventory = as_array(get(&document, "inventory"));
    assert_eq!(
        get(get(&inventory[0], "counts"), "exercised"),
        &Value::Int("0".to_string())
    );
}
