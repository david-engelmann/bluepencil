"""BP-13 to BP-17, the diff for a human.

The golden rendering is written out in full below:
the claim is that a proposal file renders as exactly
this text, no rule and no file line the proposals do
not name, and that any disagreement between the
proposal file, its inventory, and the rules file as
parsed is a named refusal before anything renders.
"""
import hashlib
import json
from pathlib import Path

import pytest

from bluepencil.cli import main
from bluepencil.diff import (DiffError, check_target,
                             render_diff)
from bluepencil.inventory import parse_rules
from bluepencil.observations import load_observations
from bluepencil.proposals import build_document

FIX = Path(__file__).parent / "fixtures"
RULES = (FIX / "rules_synthetic.md").read_bytes()
SHA = hashlib.sha256(RULES).hexdigest()

GOLDEN = """bluepencil diff, 3 proposals (delete 1, rewrite 1, promote 1)
target sha256 {sha}

## rewrite rule-1 (Setup rules)
- - Always run the local gate before pushing
+ - Run the local gate and see it pass before any push happens
  evidence, misreading, "agent ran the gate after pushing, reading before as eventually"

## delete rule-2 (Setup rules)
- - Never force push to main
  evidence, ignored, "session 12 force pushed after a rebase tangle"
  evidence, ignored, "session 19 force pushed over a stale branch"

## promote rule-7 (Setup rules / Style to Setup rules)
- - Keep functions short
  evidence, followed, "session 22 split the parser before review"
  evidence, followed, "session 23 kept the new module under review size"
""".format(sha=SHA)


def build():
    observations, rewrites = load_observations(
        FIX / "observations_synthetic.json")
    return build_document(RULES, parse_rules(
        RULES.decode())[0], observations, rewrites)


def render(document, rules=None, text=None):
    if rules is None:
        rules = parse_rules(RULES.decode())[0]
    if text is None:
        text = RULES.decode()
    return render_diff(text.splitlines(), rules, document)


def test_the_golden_diff():
    assert render(build()) == GOLDEN


def test_the_cli_prints_the_same_diff(tmp_path, capsys):
    proposals = tmp_path / "proposals.json"
    proposals.write_text(json.dumps(build()), encoding="utf-8")
    rules = tmp_path / "rules.md"
    rules.write_bytes(RULES)
    assert main(["diff", str(rules), str(proposals)]) == 0
    assert capsys.readouterr().out == GOLDEN


def test_a_different_rules_file_is_refused():
    with pytest.raises(DiffError, match="different rules file"):
        check_target(RULES + b"\n", build())


def test_a_proposal_disagreeing_with_its_inventory():
    document = build()
    document["proposals"][0]["rule_text"] = "Edited by hand"
    with pytest.raises(DiffError, match="rule-1"):
        render(document)


def test_a_proposal_disagreeing_with_the_parsed_file():
    other = parse_rules("# Setup rules\n\n"
                        "- A different first rule\n")[0]
    with pytest.raises(DiffError, match="rule-1"):
        render(build(), rules=other,
               text="# Setup rules\n\n- A different first rule\n")


def test_a_quote_with_a_newline_is_flattened():
    document = build()
    document["proposals"][1]["evidence"][0]["quote"] = (
        "line one\nline two")
    text = render(document)
    assert "line one line two" in text
    assert text.count("\n") == GOLDEN.count("\n")


def test_the_empty_proposal_list_renders_its_header(
        tmp_path, capsys):
    rules = parse_rules(RULES.decode())[0]
    document = build_document(RULES, rules, [], [])
    proposals = tmp_path / "proposals.json"
    proposals.write_text(json.dumps(document),
                         encoding="utf-8")
    rules_path = tmp_path / "rules.md"
    rules_path.write_bytes(RULES)
    assert main(["diff", str(rules_path),
                 str(proposals)]) == 0
    out = capsys.readouterr().out
    assert out == ("bluepencil diff, 0 proposals "
                   "(delete 0, rewrite 0, promote 0)\n"
                   f"target sha256 {SHA}\n")


def test_the_cli_refuses_to_write_onto_its_inputs(
        tmp_path, capsys):
    proposals = tmp_path / "proposals.json"
    proposals.write_text(json.dumps(build()), encoding="utf-8")
    rules = tmp_path / "rules.md"
    rules.write_bytes(RULES)
    assert main(["diff", str(rules), str(proposals),
                 "-o", str(rules)]) == 1
    assert "never writes to its inputs" in \
        capsys.readouterr().err
    assert main(["diff", str(rules), str(proposals),
                 "-o", str(proposals)]) == 1
    assert "never writes to its inputs" in \
        capsys.readouterr().err


def test_a_write_failure_is_a_named_error(tmp_path, capsys):
    proposals = tmp_path / "proposals.json"
    proposals.write_text(json.dumps(build()), encoding="utf-8")
    rules = tmp_path / "rules.md"
    rules.write_bytes(RULES)
    missing = tmp_path / "no-such-dir" / "diff.txt"
    assert main(["diff", str(rules), str(proposals),
                 "-o", str(missing)]) == 1
    assert "cannot write the diff file" in \
        capsys.readouterr().err


def test_malformed_documents_refuse_by_name():
    import copy

    from bluepencil.diff import load_proposals

    def forged(mutate):
        document = copy.deepcopy(build())
        mutate(document)
        return json.dumps(document).encode()

    cases = [
        lambda d: d["proposals"][0].pop("hunk"),
        lambda d: d["proposals"][0].pop("replacement"),
        lambda d: d["proposals"].__setitem__(0, "not an object"),
        lambda d: d["proposals"][1]["evidence"][0].update(
            kind="hallucinated"),
        lambda d: d["proposals"][1]["evidence"][0].update(
            kind="ignored\n## delete rule-9"),
        lambda d: d["inventory"][0].pop("id"),
        lambda d: d.update(target={"sha256": 42}),
        lambda d: d["proposals"][0].update(
            rule_id="rule-1\n## delete rule-9"),
    ]
    for mutate in cases:
        with pytest.raises(DiffError):
            load_proposals(forged(mutate))


def test_a_forged_hunk_refuses_by_name():
    import copy
    document = copy.deepcopy(build())
    document["proposals"][2]["hunk"]["to_section"] = ["Nowhere"]
    with pytest.raises(DiffError, match="rule-7"):
        render(document)
    document = copy.deepcopy(build())
    document["proposals"][1]["hunk"]["text"] = "Forged text"
    with pytest.raises(DiffError, match="rule-2"):
        render(document)
    document = copy.deepcopy(build())
    document["proposals"][0]["hunk"]["new"] = "Forged rewrite"
    with pytest.raises(DiffError, match="rule-1"):
        render(document)


def test_the_inventory_check_stands_alone():
    import copy
    document = copy.deepcopy(build())
    document["inventory"][1]["text"] = "Only the inventory moved"
    with pytest.raises(DiffError, match="rule-2"):
        render(document)


def test_the_section_check_stands_alone():
    import copy
    rules = copy.deepcopy(parse_rules(RULES.decode())[0])
    rules[1]["section"] = ["Elsewhere"]
    with pytest.raises(DiffError, match="rule-2"):
        render(build(), rules=rules)


def test_the_cli_refusal_arms(tmp_path, capsys):
    rules = tmp_path / "rules.md"
    rules.write_bytes(RULES)
    proposals = tmp_path / "proposals.json"

    proposals.write_bytes(b"this is not json")
    assert main(["diff", str(rules), str(proposals)]) == 1
    assert "not JSON" in capsys.readouterr().err

    import copy
    document = copy.deepcopy(build())
    document["proposals"][0]["evidence"] = []
    proposals.write_text(json.dumps(document), encoding="utf-8")
    assert main(["diff", str(rules), str(proposals)]) == 1
    assert "rests on no observation" in \
        capsys.readouterr().err

    proposals.write_text(json.dumps(build()), encoding="utf-8")
    rules.write_bytes(RULES + b"\n")
    assert main(["diff", str(rules), str(proposals)]) == 1
    assert "different rules file" in capsys.readouterr().err

    bad_rules = tmp_path / "bad.md"
    bad_rules.write_text("- Rule before any heading\n",
                         encoding="utf-8")
    bad_document = copy.deepcopy(build())
    bad_document["target"]["sha256"] = hashlib.sha256(
        bad_rules.read_bytes()).hexdigest()
    proposals.write_text(json.dumps(bad_document),
                         encoding="utf-8")
    assert main(["diff", str(bad_rules),
                 str(proposals)]) == 1
    assert "outside any section" in capsys.readouterr().err


CRAFTED = """# S

1. First numbered rule
   with a second line
- [ ] Boxed rule to drop
- [x] Boxed rule to rewrite
"""


def test_spans_markers_and_checkboxes_render():
    rules_bytes = CRAFTED.encode()
    rules = parse_rules(CRAFTED)[0]
    observations = [
        {"rule": "First numbered rule\nwith a second line",
         "kind": "ignored", "quote": "q1"},
        {"rule": "First numbered rule\nwith a second line",
         "kind": "ignored", "quote": "q2"},
        {"rule": "Boxed rule to drop", "kind": "ignored",
         "quote": "q3"},
        {"rule": "Boxed rule to drop", "kind": "ignored",
         "quote": "q4"},
        {"rule": "Boxed rule to rewrite", "kind": "misreading",
         "quote": "read it as already done"},
    ]
    rewrites = [{"rule": "Boxed rule to rewrite",
                 "misreading_quote": "read it as already done",
                 "replacement": "Boxed rule, rewritten"}]
    document = build_document(rules_bytes, rules,
                              observations, rewrites)
    sha = hashlib.sha256(rules_bytes).hexdigest()
    expected = f"""bluepencil diff, 3 proposals (delete 2, rewrite 1, promote 0)
target sha256 {sha}

## delete rule-1 (S)
- 1. First numbered rule
-    with a second line
  evidence, ignored, "q1"
  evidence, ignored, "q2"

## delete rule-2 (S)
- - [ ] Boxed rule to drop
  evidence, ignored, "q3"
  evidence, ignored, "q4"

## rewrite rule-3 (S)
- - [x] Boxed rule to rewrite
+ - Boxed rule, rewritten
  evidence, misreading, "read it as already done"
"""
    assert render_diff(CRAFTED.splitlines(), rules,
                       document) == expected


def test_a_hardlink_output_is_refused(tmp_path, capsys):
    import os
    rules = tmp_path / "rules.md"
    rules.write_bytes(RULES)
    proposals = tmp_path / "proposals.json"
    proposals.write_text(json.dumps(build()), encoding="utf-8")
    alias = tmp_path / "alias.md"
    os.link(rules, alias)
    assert main(["diff", str(rules), str(proposals),
                 "-o", str(alias)]) == 1
    assert "never writes to its inputs" in \
        capsys.readouterr().err
    assert rules.read_bytes() == RULES
