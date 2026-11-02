"""BP-8 to BP-12. Proposals, generation, and the output."""
import hashlib
import json
from pathlib import Path

import pytest

from bluepencil.cli import main
from bluepencil.inventory import parse_rules
from bluepencil.observations import load_observations
from bluepencil.proposals import (ProposalError, build_document,
                                  validate_proposals)

FIX = Path(__file__).parent / "fixtures"
RULES = (FIX / "rules_synthetic.md").read_bytes()


def build():
    observations, rewrites = load_observations(
        FIX / "observations_synthetic.json")
    return build_document(RULES, parse_rules(RULES.decode()),
                          observations, rewrites)


def test_the_document_names_its_target_by_sha256():
    document = build()
    assert document["target"]["sha256"] == hashlib.sha256(
        RULES).hexdigest()
    assert document["proposal_version"] == "1.0"


def test_delete_and_promote_generate_at_their_thresholds():
    proposals = build()["proposals"]
    by_kind = {(p["kind"], p["rule_id"]) for p in proposals}
    assert ("delete", "rule-2") in by_kind
    assert ("promote", "rule-7") in by_kind
    delete = next(p for p in proposals if p["kind"] == "delete")
    assert len(delete["evidence"]) == 2
    assert delete["hunk"]["action"] == "delete"
    promote = next(p for p in proposals if p["kind"] == "promote")
    assert promote["hunk"]["to_section"] == ["Setup rules"]
    assert promote["hunk"]["from_section"] == ["Setup rules", "Style"]


def test_below_threshold_no_proposal():
    rules = parse_rules("# S\n\n- Lonely rule\n")
    document = build_document(b"-", rules, [
        {"rule": "Lonely rule", "kind": "ignored", "quote": "once"}],
        [])
    assert document["proposals"] == []


def test_a_followed_observation_blocks_a_delete():
    rules = parse_rules("# S\n\n- Mixed rule\n")
    obs = [{"rule": "Mixed rule", "kind": "ignored", "quote": "a"},
           {"rule": "Mixed rule", "kind": "ignored", "quote": "b"},
           {"rule": "Mixed rule", "kind": "followed", "quote": "c"}]
    assert build_document(b"-", rules, obs, [])["proposals"] == []


def test_promote_needs_the_second_half():
    text = "# A\n\n- r1\n- r2\n- r3\n- r4\n"
    rules = parse_rules(text)
    obs = [{"rule": "r2", "kind": "followed", "quote": "a"},
           {"rule": "r2", "kind": "followed", "quote": "b"}]
    # r2 sits at index 1 of 4, 2*1 < 4, first half: no promote.
    assert build_document(b"-", rules, obs, [])["proposals"] == []


def test_the_supplied_rewrite_is_carried_with_its_misreading():
    rewrite = next(p for p in build()["proposals"]
                   if p["kind"] == "rewrite")
    assert rewrite["rule_id"] == "rule-1"
    assert rewrite["misreading_quote"].startswith("agent ran the gate")
    assert rewrite["replacement"].startswith("Run the local gate")
    assert len(rewrite["evidence"]) == 1


def test_a_rewrite_quoting_an_unobserved_misreading_is_refused():
    rules = parse_rules("# S\n\n- Some rule\n")
    with pytest.raises(ProposalError, match="not among its"):
        build_document(b"-", rules, [], [
            {"rule": "Some rule", "misreading_quote": "invented",
             "replacement": "new text"}])


def test_a_rewrite_for_a_missing_rule_is_refused():
    rules = parse_rules("# S\n\n- Some rule\n")
    with pytest.raises(ProposalError, match="not in the file"):
        build_document(b"-", rules, [], [
            {"rule": "Other rule", "misreading_quote": "q",
             "replacement": "new text"}])


def test_a_proposal_resting_on_no_observation_is_named():
    document = {"target": {"sha256": "a" * 64}, "proposals": [
        {"rule_id": "rule-9", "kind": "delete", "evidence": []}]}
    with pytest.raises(ProposalError, match="rule-9"):
        validate_proposals(document)


def test_output_is_deterministic(tmp_path):
    out_a, out_b = tmp_path / "a.json", tmp_path / "b.json"
    argv = ["propose", str(FIX / "rules_synthetic.md"),
            str(FIX / "observations_synthetic.json")]
    assert main(argv + ["-o", str(out_a)]) == 0
    assert main(argv + ["-o", str(out_b)]) == 0
    assert out_a.read_bytes() == out_b.read_bytes()


def test_the_cli_refuses_to_write_onto_its_inputs(tmp_path):
    rules = tmp_path / "rules.md"
    rules.write_bytes(RULES)
    code = main(["propose", str(rules),
                 str(FIX / "observations_synthetic.json"),
                 "-o", str(rules)])
    assert code == 1
    assert rules.read_bytes() == RULES


def test_the_cli_reports_missing_inputs_by_name(capsys):
    assert main(["propose", "/nonexistent.md", "/none.json"]) == 1
    assert "bluepencil: error:" in capsys.readouterr().err
