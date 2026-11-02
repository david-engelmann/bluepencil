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
