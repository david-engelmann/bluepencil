//! BP-13 to BP-17, the diff for a human, ported
//! from the Python suite file for file. The golden
//! rendering is written out in full below.

use std::path::PathBuf;

use bluepencil::diff::{check_target, load_proposals, render_diff};
use bluepencil::failure::Failure;
use bluepencil::inventory::parse_rules;
use bluepencil::observations::{load_observations, Observation, Rewrite};
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

fn sha_of(bytes: &[u8]) -> String {
    use sha2::Digest;
    sha2::Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn golden() -> String {
    format!(
        "bluepencil diff, 3 proposals (delete 1, rewrite 1, promote 1)\n\
         target sha256 {}\n\
         \n\
         ## rewrite rule-1 (Setup rules)\n\
         - - Always run the local gate before pushing\n\
         + - Run the local gate and see it pass before any push happens\n\
         \x20 evidence, misreading, \"agent ran the gate after pushing, reading before as eventually\"\n\
         \n\
         ## delete rule-2 (Setup rules)\n\
         - - Never force push to main\n\
         \x20 evidence, ignored, \"session 12 force pushed after a rebase tangle\"\n\
         \x20 evidence, ignored, \"session 19 force pushed over a stale branch\"\n\
         \n\
         ## promote rule-7 (Setup rules / Style to Setup rules)\n\
         - - Keep functions short\n\
         \x20 evidence, followed, \"session 22 split the parser before review\"\n\
         \x20 evidence, followed, \"session 23 kept the new module under review size\"\n",
        sha_of(&rules_bytes())
    )
}

fn rules_text(bytes: &[u8]) -> Text {
    Text::from_str(&bluepencil::observations::decode_utf8_sig(bytes).expect("fixture is utf-8"))
}

fn build() -> Value {
    let bytes = rules_bytes();
    let (rules, warnings) = parse_rules(&rules_text(&bytes)).expect("parses");
    let (observations, rewrites) =
        load_observations(&fixture("observations_synthetic.json")).expect("loads");
    build_document(&bytes, &rules, &observations, &rewrites, &warnings).expect("builds")
}

fn render(document: &Value) -> Result<Text, Failure> {
    let bytes = rules_bytes();
    let text = rules_text(&bytes);
    let (rules, _) = parse_rules(&text).expect("parses");
    let lines = bluepencil::util::py_splitlines(text.units());
    render_diff(&lines, &rules, document)
}

type Mutator = Box<dyn Fn(&mut Value)>;

fn named(failure: Failure) -> String {
    match failure {
        Failure::Named(message) => message.render_stderr(),
        Failure::Crash => panic!("a crash, not a named refusal"),
    }
}

/// Set a field on the value at a path of keys and
/// indices, the test's deepcopy-and-mutate.
fn set_path(value: &mut Value, path: &[&str], replacement: Value) {
    if path.len() == 1 {
        match value {
            Value::Object(entries) => {
                for (key, slot) in entries.iter_mut() {
                    if key.as_str().as_deref() == Some(path[0]) {
                        *slot = replacement;
                        return;
                    }
                }
                entries.push((Text::from_str(path[0]), replacement));
            }
            _ => panic!("not an object"),
        }
        return;
    }
    match value {
        Value::Object(entries) => {
            for (key, slot) in entries.iter_mut() {
                if key.as_str().as_deref() == Some(path[0]) {
                    set_path(slot, &path[1..], replacement);
                    return;
                }
            }
            panic!("no such key {}", path[0]);
        }
        _ => panic!("not an object"),
    }
}

fn proposal_at(document: &mut Value, index: usize) -> &mut Value {
    match document {
        Value::Object(entries) => {
            for (key, slot) in entries.iter_mut() {
                if key.as_str().as_deref() == Some("proposals") {
                    match slot {
                        Value::Array(items) => return &mut items[index],
                        _ => panic!("not an array"),
                    }
                }
            }
            panic!("no proposals");
        }
        _ => panic!("not an object"),
    }
}

fn inventory_at(document: &mut Value, index: usize) -> &mut Value {
    match document {
        Value::Object(entries) => {
            for (key, slot) in entries.iter_mut() {
                if key.as_str().as_deref() == Some("inventory") {
                    match slot {
                        Value::Array(items) => return &mut items[index],
                        _ => panic!("not an array"),
                    }
                }
            }
            panic!("no inventory");
        }
        _ => panic!("not an object"),
    }
}

fn document_bytes(document: &Value) -> Vec<u8> {
    bluepencil::emit::dumps_document(document, bluepencil::emit::Mode::Strict)
        .expect("the document renders")
}

fn tmpdir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "bp-diff-{}-{}-{}",
        std::process::id(),
        tag,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("a clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("a temp dir");
    dir
}

struct Run {
    code: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn run(args: &[&str]) -> Run {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_bluepencil"))
        .args(args)
        .output()
        .expect("the binary runs");
    Run {
        code: output.status.code().expect("an exit code"),
        stdout: output.stdout,
        stderr: output.stderr,
    }
}

fn stderr_text(run: &Run) -> String {
    String::from_utf8_lossy(&run.stderr).into_owned()
}

#[test]
fn the_golden_diff() {
    let rendered = render(&build()).expect("renders");
    assert_eq!(rendered.as_str().expect("valid text"), golden());
}

#[test]
fn the_cli_prints_the_same_diff() {
    let dir = tmpdir("golden");
    let proposals = dir.join("proposals.json");
    std::fs::write(&proposals, document_bytes(&build())).expect("writes");
    let rules = dir.join("rules.md");
    std::fs::write(&rules, rules_bytes()).expect("writes");
    let run = run(&[
        "diff",
        rules.to_str().expect("a path"),
        proposals.to_str().expect("a path"),
    ]);
    assert_eq!(run.code, 0);
    assert_eq!(String::from_utf8(run.stdout).expect("utf-8"), golden());
}

#[test]
fn a_different_rules_file_is_refused() {
    let mut bytes = rules_bytes();
    bytes.push(b'\n');
    let failure = check_target(&bytes, &build()).expect_err("refused");
    assert!(named(failure).contains("different rules file"));
}

#[test]
fn a_proposal_disagreeing_with_its_inventory() {
    let mut document = build();
    set_path(
        proposal_at(&mut document, 0),
        &["rule_text"],
        Value::Str(Text::from_str("Edited by hand")),
    );
    let failure = render(&document).expect_err("refused");
    assert!(named(failure).contains("rule-1"));
}

#[test]
fn a_proposal_disagreeing_with_the_parsed_file() {
    let document = build();
    let other_text = "# Setup rules\n\n- A different first rule\n";
    let (other, _) = parse_rules(&Text::from_str(other_text)).expect("parses");
    let lines = bluepencil::util::py_splitlines(Text::from_str(other_text).units());
    let failure = render_diff(&lines, &other, &document).expect_err("refused");
    assert!(named(failure).contains("rule-1"));
}

#[test]
fn a_quote_with_a_newline_is_flattened() {
    let mut document = build();
    let evidence = match proposal_at(&mut document, 1) {
        Value::Object(entries) => entries
            .iter_mut()
            .find(|(key, _)| key.as_str().as_deref() == Some("evidence"))
            .map(|(_, slot)| slot)
            .expect("evidence"),
        _ => panic!("not an object"),
    };
    let first = match evidence {
        Value::Array(items) => &mut items[0],
        _ => panic!("not an array"),
    };
    set_path(
        first,
        &["quote"],
        Value::Str(Text::from_str("line one\nline two")),
    );
    let rendered = render(&document).expect("renders");
    let text = rendered.as_str().expect("valid text");
    assert!(text.contains("line one line two"));
    assert_eq!(text.matches('\n').count(), golden().matches('\n').count());
}

#[test]
fn the_empty_proposal_list_renders_its_header() {
    let dir = tmpdir("empty");
    let bytes = rules_bytes();
    let (rules, warnings) = parse_rules(&rules_text(&bytes)).expect("parses");
    let document = build_document(&bytes, &rules, &[], &[], &warnings).expect("builds");
    let proposals = dir.join("proposals.json");
    std::fs::write(&proposals, document_bytes(&document)).expect("writes");
    let rules_path = dir.join("rules.md");
    std::fs::write(&rules_path, &bytes).expect("writes");
    let run = run(&[
        "diff",
        rules_path.to_str().expect("a path"),
        proposals.to_str().expect("a path"),
    ]);
    assert_eq!(run.code, 0);
    let expected = format!(
        "bluepencil diff, 0 proposals (delete 0, rewrite 0, promote 0)\ntarget sha256 {}\n",
        sha_of(&bytes)
    );
    assert_eq!(String::from_utf8(run.stdout).expect("utf-8"), expected);
}

#[test]
fn the_cli_refuses_to_write_onto_its_inputs() {
    let dir = tmpdir("guard");
    let proposals = dir.join("proposals.json");
    std::fs::write(&proposals, document_bytes(&build())).expect("writes");
    let rules = dir.join("rules.md");
    std::fs::write(&rules, rules_bytes()).expect("writes");
    for out in [&rules, &proposals] {
        let run = run(&[
            "diff",
            rules.to_str().expect("a path"),
            proposals.to_str().expect("a path"),
            "-o",
            out.to_str().expect("a path"),
        ]);
        assert_eq!(run.code, 1);
        assert!(stderr_text(&run).contains("never writes to its inputs"));
    }
}

#[test]
fn a_write_failure_is_a_named_error() {
    let dir = tmpdir("writefail");
    let proposals = dir.join("proposals.json");
    std::fs::write(&proposals, document_bytes(&build())).expect("writes");
    let rules = dir.join("rules.md");
    std::fs::write(&rules, rules_bytes()).expect("writes");
    let missing = dir.join("no-such-dir").join("diff.txt");
    let run = run(&[
        "diff",
        rules.to_str().expect("a path"),
        proposals.to_str().expect("a path"),
        "-o",
        missing.to_str().expect("a path"),
    ]);
    assert_eq!(run.code, 1);
    assert!(stderr_text(&run).contains("cannot write the diff file"));
}

#[test]
fn malformed_documents_refuse_by_name() {
    // Each case forges the built document the way the
    // Python suite does and expects load_proposals to
    // refuse it by name.
    let cases: Vec<Mutator> = vec![
        Box::new(|document| {
            let proposal = proposal_at(document, 0);
            if let Value::Object(entries) = proposal {
                entries.retain(|(key, _)| key.as_str().as_deref() != Some("hunk"));
            }
        }),
        Box::new(|document| {
            let proposal = proposal_at(document, 0);
            if let Value::Object(entries) = proposal {
                entries.retain(|(key, _)| key.as_str().as_deref() != Some("replacement"));
            }
        }),
        Box::new(|document| {
            let proposal = proposal_at(document, 0);
            *proposal = Value::Str(Text::from_str("not an object"));
        }),
        Box::new(|document| {
            let proposal = proposal_at(document, 1);
            set_path(proposal, &["evidence"], Value::Array(vec![]));
            let proposal = proposal_at(document, 1);
            if let Value::Object(entries) = proposal {
                for (key, slot) in entries.iter_mut() {
                    if key.as_str().as_deref() == Some("evidence") {
                        *slot = Value::Array(vec![Value::Object(vec![
                            (
                                Text::from_str("kind"),
                                Value::Str(Text::from_str("hallucinated")),
                            ),
                            (Text::from_str("quote"), Value::Str(Text::from_str("q"))),
                            (Text::from_str("rule"), Value::Str(Text::from_str("r"))),
                        ])]);
                    }
                }
            }
        }),
        Box::new(|document| {
            let proposal = proposal_at(document, 1);
            if let Value::Object(entries) = proposal {
                for (key, slot) in entries.iter_mut() {
                    if key.as_str().as_deref() == Some("evidence") {
                        *slot = Value::Array(vec![Value::Object(vec![
                            (
                                Text::from_str("kind"),
                                Value::Str(Text::from_str("ignored\n## delete rule-9")),
                            ),
                            (Text::from_str("quote"), Value::Str(Text::from_str("q"))),
                            (Text::from_str("rule"), Value::Str(Text::from_str("r"))),
                        ])]);
                    }
                }
            }
        }),
        Box::new(|document| {
            let entry = inventory_at(document, 0);
            if let Value::Object(entries) = entry {
                entries.retain(|(key, _)| key.as_str().as_deref() != Some("id"));
            }
        }),
        Box::new(|document| {
            set_path(
                document,
                &["target"],
                Value::Object(vec![(
                    Text::from_str("sha256"),
                    Value::Int("42".to_string()),
                )]),
            );
        }),
        Box::new(|document| {
            let proposal = proposal_at(document, 0);
            set_path(
                proposal,
                &["rule_id"],
                Value::Str(Text::from_str("rule-1\n## delete rule-9")),
            );
        }),
    ];
    for mutate in cases {
        let mut document = build();
        mutate(&mut document);
        let failure = load_proposals(&document_bytes(&document)).expect_err("refused");
        named(failure);
    }
}

#[test]
fn a_forged_hunk_refuses_by_name() {
    let mut document = build();
    set_path(
        proposal_at(&mut document, 2),
        &["hunk", "to_section"],
        Value::Array(vec![Value::Str(Text::from_str("Nowhere"))]),
    );
    // set_path walks objects only; the hunk path is
    // two objects deep, handled below.
    let failure = render(&document).expect_err("refused");
    assert!(named(failure).contains("rule-7"));

    let mut document = build();
    set_hunk_field(
        &mut document,
        1,
        "text",
        Value::Str(Text::from_str("Forged text")),
    );
    let failure = render(&document).expect_err("refused");
    assert!(named(failure).contains("rule-2"));

    let mut document = build();
    set_hunk_field(
        &mut document,
        0,
        "new",
        Value::Str(Text::from_str("Forged rewrite")),
    );
    let failure = render(&document).expect_err("refused");
    assert!(named(failure).contains("rule-1"));
}

fn set_hunk_field(document: &mut Value, index: usize, field: &str, replacement: Value) {
    let proposal = proposal_at(document, index);
    if let Value::Object(entries) = proposal {
        for (key, slot) in entries.iter_mut() {
            if key.as_str().as_deref() == Some("hunk") {
                if let Value::Object(hunk) = slot {
                    for (hkey, hslot) in hunk.iter_mut() {
                        if hkey.as_str().as_deref() == Some(field) {
                            *hslot = replacement;
                            return;
                        }
                    }
                    hunk.push((Text::from_str(field), replacement));
                    return;
                }
            }
        }
    }
    panic!("no hunk");
}

#[test]
fn the_inventory_check_stands_alone() {
    let mut document = build();
    set_path(
        inventory_at(&mut document, 1),
        &["text"],
        Value::Str(Text::from_str("Only the inventory moved")),
    );
    let failure = render(&document).expect_err("refused");
    assert!(named(failure).contains("rule-2"));
}

#[test]
fn the_section_check_stands_alone() {
    let document = build();
    let bytes = rules_bytes();
    let text = rules_text(&bytes);
    let (mut rules, _) = parse_rules(&text).expect("parses");
    rules[1].section = vec![Text::from_str("Elsewhere")];
    let lines = bluepencil::util::py_splitlines(text.units());
    let failure = render_diff(&lines, &rules, &document).expect_err("refused");
    assert!(named(failure).contains("rule-2"));
}

#[test]
fn the_cli_refusal_arms() {
    let dir = tmpdir("refusals");
    let rules = dir.join("rules.md");
    std::fs::write(&rules, rules_bytes()).expect("writes");
    let proposals = dir.join("proposals.json");

    std::fs::write(&proposals, b"this is not json").expect("writes");
    let outcome = run(&[
        "diff",
        rules.to_str().expect("a path"),
        proposals.to_str().expect("a path"),
    ]);
    assert_eq!(outcome.code, 1);
    assert!(stderr_text(&outcome).contains("not JSON"));

    let mut document = build();
    set_path(
        proposal_at(&mut document, 0),
        &["evidence"],
        Value::Array(vec![]),
    );
    std::fs::write(&proposals, document_bytes(&document)).expect("writes");
    let outcome = run(&[
        "diff",
        rules.to_str().expect("a path"),
        proposals.to_str().expect("a path"),
    ]);
    assert_eq!(outcome.code, 1);
    assert!(stderr_text(&outcome).contains("rests on no observation"));

    std::fs::write(&proposals, document_bytes(&build())).expect("writes");
    let mut longer = rules_bytes();
    longer.push(b'\n');
    std::fs::write(&rules, &longer).expect("writes");
    let outcome = run(&[
        "diff",
        rules.to_str().expect("a path"),
        proposals.to_str().expect("a path"),
    ]);
    assert_eq!(outcome.code, 1);
    assert!(stderr_text(&outcome).contains("different rules file"));

    let bad_rules = dir.join("bad.md");
    std::fs::write(&bad_rules, "- Rule before any heading\n").expect("writes");
    let mut bad_document = build();
    let bad_sha = sha_of(&std::fs::read(&bad_rules).expect("reads"));
    set_path(
        &mut bad_document,
        &["target"],
        Value::Object(vec![(
            Text::from_str("sha256"),
            Value::Str(Text::from_str(&bad_sha)),
        )]),
    );
    std::fs::write(&proposals, document_bytes(&bad_document)).expect("writes");
    let outcome = run(&[
        "diff",
        bad_rules.to_str().expect("a path"),
        proposals.to_str().expect("a path"),
    ]);
    assert_eq!(outcome.code, 1);
    assert!(stderr_text(&outcome).contains("outside any section"));
}

const CRAFTED: &str = "# S\n\n1. First numbered rule\n   with a second line\n- [ ] Boxed rule to drop\n- [x] Boxed rule to rewrite\n";

#[test]
fn spans_markers_and_checkboxes_render() {
    let bytes = CRAFTED.as_bytes();
    let (rules, warnings) = parse_rules(&Text::from_str(CRAFTED)).expect("parses");
    let observations = vec![
        Observation {
            rule: Text::from_str("First numbered rule\nwith a second line"),
            kind: Text::from_str("ignored"),
            quote: Text::from_str("q1"),
        },
        Observation {
            rule: Text::from_str("First numbered rule\nwith a second line"),
            kind: Text::from_str("ignored"),
            quote: Text::from_str("q2"),
        },
        Observation {
            rule: Text::from_str("Boxed rule to drop"),
            kind: Text::from_str("ignored"),
            quote: Text::from_str("q3"),
        },
        Observation {
            rule: Text::from_str("Boxed rule to drop"),
            kind: Text::from_str("ignored"),
            quote: Text::from_str("q4"),
        },
        Observation {
            rule: Text::from_str("Boxed rule to rewrite"),
            kind: Text::from_str("misreading"),
            quote: Text::from_str("read it as already done"),
        },
    ];
    let rewrites = vec![Rewrite {
        rule: Text::from_str("Boxed rule to rewrite"),
        misreading_quote: Text::from_str("read it as already done"),
        replacement: Text::from_str("Boxed rule, rewritten"),
    }];
    let document =
        build_document(bytes, &rules, &observations, &rewrites, &warnings).expect("builds");
    let sha = sha_of(bytes);
    let expected = format!(
        "bluepencil diff, 3 proposals (delete 2, rewrite 1, promote 0)\n\
         target sha256 {sha}\n\
         \n\
         ## delete rule-1 (S)\n\
         - 1. First numbered rule\n\
         -    with a second line\n\
         \x20 evidence, ignored, \"q1\"\n\
         \x20 evidence, ignored, \"q2\"\n\
         \n\
         ## delete rule-2 (S)\n\
         - - [ ] Boxed rule to drop\n\
         \x20 evidence, ignored, \"q3\"\n\
         \x20 evidence, ignored, \"q4\"\n\
         \n\
         ## rewrite rule-3 (S)\n\
         - - [x] Boxed rule to rewrite\n\
         + - Boxed rule, rewritten\n\
         \x20 evidence, misreading, \"read it as already done\"\n"
    );
    let lines = bluepencil::util::py_splitlines(Text::from_str(CRAFTED).units());
    let rendered = render_diff(&lines, &rules, &document).expect("renders");
    assert_eq!(rendered.as_str().expect("valid text"), expected);
}

#[test]
fn a_hardlink_output_is_refused() {
    let dir = tmpdir("hardlink");
    let rules = dir.join("rules.md");
    std::fs::write(&rules, rules_bytes()).expect("writes");
    let proposals = dir.join("proposals.json");
    std::fs::write(&proposals, document_bytes(&build())).expect("writes");
    let alias = dir.join("alias.md");
    std::fs::hard_link(&rules, &alias).expect("a hardlink");
    let run = run(&[
        "diff",
        rules.to_str().expect("a path"),
        proposals.to_str().expect("a path"),
        "-o",
        alias.to_str().expect("a path"),
    ]);
    assert_eq!(run.code, 1);
    assert!(stderr_text(&run).contains("never writes to its inputs"));
    assert_eq!(std::fs::read(&rules).expect("the file"), rules_bytes());
}

#[test]
fn the_json_loaders_refuse_every_decode_failure() {
    let big_int = vec![b'1'; 5000];
    let failure = load_proposals(&big_int).expect_err("refused");
    assert!(named(failure).contains("not JSON"));
    let mut deep = vec![b'['; 20000];
    deep.extend_from_slice(&vec![b']'; 20000]);
    let failure = load_proposals(&deep).expect_err("refused");
    assert!(named(failure).contains("not JSON"));
}

#[test]
fn the_observations_loader_refuses_them_too() {
    let dir = tmpdir("obsdepth");
    let path = dir.join("observations.json");
    std::fs::write(&path, vec![b'1'; 5000]).expect("writes");
    assert!(load_observations(&path).is_err());
}
