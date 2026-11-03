//! BP-8 to BP-12. Proposals, generation, and the
//! output, ported from the Python suite file for
//! file. The CLI cases drive the compiled binary.

use std::path::PathBuf;

use bluepencil::failure::Failure;
use bluepencil::inventory::parse_rules;
use bluepencil::observations::{load_observations, Observation, Rewrite};
use bluepencil::proposals::{build_document, validate_proposals};
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

fn build() -> Value {
    let bytes = rules_bytes();
    let text = Text::from_str(
        &bluepencil::observations::decode_utf8_sig(&bytes).expect("fixture is utf-8"),
    );
    let (rules, warnings) = parse_rules(&text).expect("parses");
    let (observations, rewrites) =
        load_observations(&fixture("observations_synthetic.json")).expect("loads");
    build_document(&bytes, &rules, &observations, &rewrites, &warnings).expect("builds")
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

fn obs(rule: &str, kind: &str, quote: &str) -> Observation {
    Observation {
        rule: Text::from_str(rule),
        kind: Text::from_str(kind),
        quote: Text::from_str(quote),
    }
}

fn rewrite(rule: &str, quote: &str, replacement: &str) -> Rewrite {
    Rewrite {
        rule: Text::from_str(rule),
        misreading_quote: Text::from_str(quote),
        replacement: Text::from_str(replacement),
    }
}

fn named(failure: Failure) -> String {
    match failure {
        Failure::Named(message) => message.render_stderr(),
        Failure::Crash => panic!("a crash, not a named refusal"),
    }
}

fn parse(text: &str) -> (Vec<bluepencil::inventory::Rule>, Vec<Text>) {
    parse_rules(&Text::from_str(text)).expect("parses")
}

fn build_small(text: &str, observations: &[Observation], rewrites: &[Rewrite]) -> Value {
    let (rules, warnings) = parse(text);
    build_document(b"-", &rules, observations, rewrites, &warnings).expect("builds")
}

fn tmpdir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "bp-prop-{}-{}-{}",
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
fn the_document_names_its_target_by_sha256() {
    let document = build();
    let expected = {
        use sha2::Digest;
        let digest = sha2::Sha256::digest(rules_bytes());
        digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    assert_eq!(as_text(get(get(&document, "target"), "sha256")), expected);
    assert_eq!(as_text(get(&document, "proposal_version")), "1.0");
}

#[test]
fn delete_and_promote_generate_at_their_thresholds() {
    let document = build();
    let proposals = as_array(get(&document, "proposals"));
    let kinds: Vec<(String, String)> = proposals
        .iter()
        .map(|p| (as_text(get(p, "kind")), as_text(get(p, "rule_id"))))
        .collect();
    assert!(kinds.contains(&("delete".to_string(), "rule-2".to_string())));
    assert!(kinds.contains(&("promote".to_string(), "rule-7".to_string())));
    let delete = proposals
        .iter()
        .find(|p| as_text(get(p, "kind")) == "delete")
        .expect("a delete");
    assert_eq!(as_array(get(delete, "evidence")).len(), 2);
    assert_eq!(as_text(get(get(delete, "hunk"), "action")), "delete");
    let promote = proposals
        .iter()
        .find(|p| as_text(get(p, "kind")) == "promote")
        .expect("a promote");
    let to_section = get(get(promote, "hunk"), "to_section");
    assert_eq!(
        as_array(to_section).iter().map(as_text).collect::<Vec<_>>(),
        ["Setup rules"]
    );
    let from_section = get(get(promote, "hunk"), "from_section");
    assert_eq!(
        as_array(from_section)
            .iter()
            .map(as_text)
            .collect::<Vec<_>>(),
        ["Setup rules", "Style"]
    );
}

#[test]
fn below_threshold_no_proposal() {
    let document = build_small(
        "# S\n\n- Lonely rule\n",
        &[obs("Lonely rule", "ignored", "once")],
        &[],
    );
    assert!(as_array(get(&document, "proposals")).is_empty());
}

#[test]
fn a_followed_observation_blocks_a_delete() {
    let document = build_small(
        "# S\n\n- Mixed rule\n",
        &[
            obs("Mixed rule", "ignored", "a"),
            obs("Mixed rule", "ignored", "b"),
            obs("Mixed rule", "followed", "c"),
        ],
        &[],
    );
    assert!(as_array(get(&document, "proposals")).is_empty());
}

#[test]
fn promote_needs_the_second_half() {
    let document = build_small(
        "# A\n\n- r1\n- r2\n- r3\n- r4\n",
        &[obs("r2", "followed", "a"), obs("r2", "followed", "b")],
        &[],
    );
    assert!(as_array(get(&document, "proposals")).is_empty());
}

#[test]
fn the_supplied_rewrite_is_carried_with_its_misreading() {
    let document = build();
    let rewrite = as_array(get(&document, "proposals"))
        .iter()
        .find(|p| as_text(get(p, "kind")) == "rewrite")
        .expect("a rewrite")
        .clone();
    assert_eq!(as_text(get(&rewrite, "rule_id")), "rule-1");
    assert!(as_text(get(&rewrite, "misreading_quote")).starts_with("agent ran the gate"));
    assert!(as_text(get(&rewrite, "replacement")).starts_with("Run the local gate"));
    assert_eq!(as_array(get(&rewrite, "evidence")).len(), 1);
}

#[test]
fn a_rewrite_quoting_an_unobserved_misreading_is_refused() {
    let (rules, warnings) = parse("# S\n\n- Some rule\n");
    let failure = build_document(
        b"-",
        &rules,
        &[],
        &[rewrite("Some rule", "invented", "new text")],
        &warnings,
    )
    .expect_err("refused");
    assert!(named(failure).contains("not among its"));
}

#[test]
fn a_rewrite_for_a_missing_rule_is_refused() {
    let (rules, warnings) = parse("# S\n\n- Some rule\n");
    let failure = build_document(
        b"-",
        &rules,
        &[],
        &[rewrite("Other rule", "q", "new text")],
        &warnings,
    )
    .expect_err("refused");
    assert!(named(failure).contains("not in the file"));
}

#[test]
fn a_proposal_resting_on_no_observation_is_named() {
    let document = Value::Object(vec![
        (
            Text::from_str("target"),
            Value::Object(vec![(
                Text::from_str("sha256"),
                Value::Str(Text::from_str(&"a".repeat(64))),
            )]),
        ),
        (
            Text::from_str("proposals"),
            Value::Array(vec![Value::Object(vec![
                (
                    Text::from_str("rule_id"),
                    Value::Str(Text::from_str("rule-9")),
                ),
                (Text::from_str("kind"), Value::Str(Text::from_str("delete"))),
                (Text::from_str("evidence"), Value::Array(vec![])),
            ])]),
        ),
    ]);
    let failure = validate_proposals(&document).expect_err("refused");
    assert!(named(failure).contains("rule-9"));
}

#[test]
fn output_is_deterministic() {
    let dir = tmpdir("deterministic");
    let out_a = dir.join("a.json");
    let out_b = dir.join("b.json");
    let rules = fixture("rules_synthetic.md");
    let observations = fixture("observations_synthetic.json");
    let first = run(&[
        "propose",
        rules.to_str().expect("a path"),
        observations.to_str().expect("a path"),
        "-o",
        out_a.to_str().expect("a path"),
    ]);
    let second = run(&[
        "propose",
        rules.to_str().expect("a path"),
        observations.to_str().expect("a path"),
        "-o",
        out_b.to_str().expect("a path"),
    ]);
    assert_eq!(first.code, 0);
    assert_eq!(second.code, 0);
    assert_eq!(
        std::fs::read(&out_a).expect("a file"),
        std::fs::read(&out_b).expect("a file")
    );
}

#[test]
fn the_cli_refuses_to_write_onto_its_inputs() {
    let dir = tmpdir("guard");
    let rules = dir.join("rules.md");
    std::fs::write(&rules, rules_bytes()).expect("writes");
    let observations = fixture("observations_synthetic.json");
    let run = run(&[
        "propose",
        rules.to_str().expect("a path"),
        observations.to_str().expect("a path"),
        "-o",
        rules.to_str().expect("a path"),
    ]);
    assert_eq!(run.code, 1);
    assert_eq!(std::fs::read(&rules).expect("the file"), rules_bytes());
}

#[test]
fn the_cli_reports_missing_inputs_by_name() {
    let run = run(&["propose", "/nonexistent.md", "/none.json"]);
    assert_eq!(run.code, 1);
    assert!(stderr_text(&run).contains("error, no such rules file"));
}

#[test]
fn promote_fires_exactly_on_the_second_half_boundary() {
    let document = build_small(
        "# A\n\n- r1\n- r2\n\n## B\n\n- r3\n- r4\n",
        &[obs("r3", "followed", "a"), obs("r3", "followed", "b")],
        &[],
    );
    let ids: Vec<String> = as_array(get(&document, "proposals"))
        .iter()
        .map(|p| as_text(get(p, "rule_id")))
        .collect();
    assert_eq!(ids, ["rule-3"]);
}

#[test]
fn one_followed_observation_does_not_promote() {
    let document = build_small(
        "# A\n\n- r1\n- r2\n\n## B\n\n- r3\n- r4\n",
        &[obs("r4", "followed", "only once")],
        &[],
    );
    assert!(as_array(get(&document, "proposals")).is_empty());
}

#[test]
fn the_first_section_guard_blocks_a_promote() {
    let document = build_small(
        "# A\n\n- r1\n- r2\n- r3\n- r4\n",
        &[obs("r4", "followed", "a"), obs("r4", "followed", "b")],
        &[],
    );
    assert!(as_array(get(&document, "proposals")).is_empty());
}

#[test]
fn generated_evidence_is_filtered_by_kind() {
    let document = build();
    let proposals = as_array(get(&document, "proposals"));
    let delete = proposals
        .iter()
        .find(|p| as_text(get(p, "kind")) == "delete")
        .expect("a delete");
    assert!(as_array(get(delete, "evidence"))
        .iter()
        .all(|o| as_text(get(o, "kind")) == "ignored"));
    let promote = proposals
        .iter()
        .find(|p| as_text(get(p, "kind")) == "promote")
        .expect("a promote");
    assert!(as_array(get(promote, "evidence"))
        .iter()
        .all(|o| as_text(get(o, "kind")) == "followed"));
}

#[test]
fn the_validator_refuses_an_unknown_kind_and_unquoted_evidence() {
    let make = |kind: &str, evidence: Value| {
        Value::Object(vec![
            (
                Text::from_str("target"),
                Value::Object(vec![(
                    Text::from_str("sha256"),
                    Value::Str(Text::from_str(&"a".repeat(64))),
                )]),
            ),
            (
                Text::from_str("proposals"),
                Value::Array(vec![Value::Object(vec![
                    (
                        Text::from_str("rule_id"),
                        Value::Str(Text::from_str("rule-1")),
                    ),
                    (Text::from_str("kind"), Value::Str(Text::from_str(kind))),
                    (Text::from_str("evidence"), evidence),
                ])]),
            ),
        ])
    };
    let unknown = make(
        "shred",
        Value::Array(vec![Value::Object(vec![(
            Text::from_str("quote"),
            Value::Str(Text::from_str("q")),
        )])]),
    );
    let failure = validate_proposals(&unknown).expect_err("refused");
    assert!(named(failure).contains("unknown kind"));
    let unquoted = make(
        "delete",
        Value::Array(vec![Value::Object(vec![(
            Text::from_str("kind"),
            Value::Str(Text::from_str("ignored")),
        )])]),
    );
    let failure = validate_proposals(&unquoted).expect_err("refused");
    assert!(named(failure).contains("no quote"));
}

#[test]
fn a_write_failure_is_a_named_error_with_exit_1() {
    let dir = tmpdir("writefail");
    let out = dir.join("missing").join("out.json");
    let rules = fixture("rules_synthetic.md");
    let observations = fixture("observations_synthetic.json");
    let run = run(&[
        "propose",
        rules.to_str().expect("a path"),
        observations.to_str().expect("a path"),
        "-o",
        out.to_str().expect("a path"),
    ]);
    assert_eq!(run.code, 1);
    assert!(stderr_text(&run).contains("cannot write the proposal file"));
}

#[test]
fn stdout_mode_writes_the_document() {
    let rules = fixture("rules_synthetic.md");
    let observations = fixture("observations_synthetic.json");
    let run = run(&[
        "propose",
        rules.to_str().expect("a path"),
        observations.to_str().expect("a path"),
    ]);
    assert_eq!(run.code, 0);
    let scalars: Vec<u32> = String::from_utf8(run.stdout)
        .expect("utf-8 stdout")
        .chars()
        .map(|ch| ch as u32)
        .collect();
    let document = bluepencil::json::parse(&scalars).expect("the document parses");
    assert_eq!(as_text(get(&document, "proposal_version")), "1.0");
}

#[test]
fn a_first_half_rule_in_a_later_section_does_not_promote() {
    let document = build_small(
        "# A\n\n- r1\n\n## B\n\n- r2\n- r3\n- r4\n- r5\n- r6\n",
        &[obs("r2", "followed", "a"), obs("r2", "followed", "b")],
        &[],
    );
    assert!(as_array(get(&document, "proposals")).is_empty());
}

#[test]
fn a_rules_file_with_a_bom_still_parses() {
    let dir = tmpdir("bom");
    let rules = dir.join("rules.md");
    let mut bytes = vec![0xEF, 0xBB, 0xBF];
    bytes.extend_from_slice(b"# S\n\n- A rule\n");
    std::fs::write(&rules, &bytes).expect("writes");
    let empty = dir.join("empty.json");
    std::fs::write(&empty, "{\"observations\": []}").expect("writes");
    let out = dir.join("out.json");
    let run = run(&[
        "propose",
        rules.to_str().expect("a path"),
        empty.to_str().expect("a path"),
        "-o",
        out.to_str().expect("a path"),
    ]);
    assert_eq!(run.code, 0);
    let text = std::fs::read_to_string(&out).expect("the output");
    assert!(text.contains("\"A rule\""));
}

#[test]
fn an_unencodable_quote_is_a_named_write_error() {
    let dir = tmpdir("unencodable");
    let obs_path = dir.join("obs.json");
    std::fs::write(
        &obs_path,
        "{\"observations\": [\
         {\"rule\": \"Never force push to main\", \"kind\": \"ignored\", \"quote\": \"bad \\ud800 quote\"}, \
         {\"rule\": \"Never force push to main\", \"kind\": \"ignored\", \"quote\": \"fine quote\"}]}",
    )
    .expect("writes");
    let rules = fixture("rules_synthetic.md");
    let out = dir.join("out.json");
    let run = run(&[
        "propose",
        rules.to_str().expect("a path"),
        obs_path.to_str().expect("a path"),
        "-o",
        out.to_str().expect("a path"),
    ]);
    assert_eq!(run.code, 1);
    assert!(stderr_text(&run).contains("cannot write the proposal file"));
}

#[test]
fn propose_refuses_a_hardlink_output() {
    let dir = tmpdir("hardlink");
    let rules = dir.join("rules.md");
    std::fs::write(&rules, rules_bytes()).expect("writes");
    let alias = dir.join("alias.md");
    std::fs::hard_link(&rules, &alias).expect("a hardlink");
    let observations = fixture("observations_synthetic.json");
    let run = run(&[
        "propose",
        rules.to_str().expect("a path"),
        observations.to_str().expect("a path"),
        "-o",
        alias.to_str().expect("a path"),
    ]);
    assert_eq!(run.code, 1);
    assert!(stderr_text(&run).contains("never writes to its inputs"));
    assert_eq!(std::fs::read(&rules).expect("the file"), rules_bytes());
}
