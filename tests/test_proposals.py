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
    return build_document(RULES, parse_rules(RULES.decode())[0],
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
    rules = parse_rules("# S\n\n- Lonely rule\n")[0]
    document = build_document(b"-", rules, [
        {"rule": "Lonely rule", "kind": "ignored", "quote": "once"}],
        [])
    assert document["proposals"] == []


def test_a_followed_observation_blocks_a_delete():
    rules = parse_rules("# S\n\n- Mixed rule\n")[0]
    obs = [{"rule": "Mixed rule", "kind": "ignored", "quote": "a"},
           {"rule": "Mixed rule", "kind": "ignored", "quote": "b"},
           {"rule": "Mixed rule", "kind": "followed", "quote": "c"}]
    assert build_document(b"-", rules, obs, [])["proposals"] == []


def test_promote_needs_the_second_half():
    text = "# A\n\n- r1\n- r2\n- r3\n- r4\n"
    rules = parse_rules(text)[0]
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
    rules = parse_rules("# S\n\n- Some rule\n")[0]
    with pytest.raises(ProposalError, match="not among its"):
        build_document(b"-", rules, [], [
            {"rule": "Some rule", "misreading_quote": "invented",
             "replacement": "new text"}])


def test_a_rewrite_for_a_missing_rule_is_refused():
    rules = parse_rules("# S\n\n- Some rule\n")[0]
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
    assert "error, no such rules file" in capsys.readouterr().err


def test_promote_fires_exactly_on_the_second_half_boundary():
    # A 4-rule file puts rule-3 at index 2, where 2i == n.
    text = "# A\n\n- r1\n- r2\n\n## B\n\n- r3\n- r4\n"
    rules = parse_rules(text)[0]
    obs = [{"rule": "r3", "kind": "followed", "quote": "a"},
           {"rule": "r3", "kind": "followed", "quote": "b"}]
    proposals = build_document(b"-", rules, obs, [])["proposals"]
    assert [p["rule_id"] for p in proposals] == ["rule-3"]


def test_one_followed_observation_does_not_promote():
    text = "# A\n\n- r1\n- r2\n\n## B\n\n- r3\n- r4\n"
    rules = parse_rules(text)[0]
    obs = [{"rule": "r4", "kind": "followed", "quote": "only once"}]
    assert build_document(b"-", rules, obs, [])["proposals"] == []


def test_the_first_section_guard_blocks_a_promote():
    text = "# A\n\n- r1\n- r2\n- r3\n- r4\n"
    rules = parse_rules(text)[0]
    obs = [{"rule": "r4", "kind": "followed", "quote": "a"},
           {"rule": "r4", "kind": "followed", "quote": "b"}]
    assert build_document(b"-", rules, obs, [])["proposals"] == []


def test_generated_evidence_is_filtered_by_kind():
    document = build()
    delete = next(p for p in document["proposals"]
                  if p["kind"] == "delete")
    assert all(o["kind"] == "ignored" for o in delete["evidence"])
    promote = next(p for p in document["proposals"]
                   if p["kind"] == "promote")
    assert all(o["kind"] == "followed" for o in promote["evidence"])


def test_the_validator_refuses_an_unknown_kind_and_unquoted_evidence():
    document = {"target": {"sha256": "a" * 64}, "proposals": [
        {"rule_id": "rule-1", "kind": "shred",
         "evidence": [{"quote": "q"}]}]}
    with pytest.raises(ProposalError, match="unknown kind"):
        validate_proposals(document)
    document["proposals"][0]["kind"] = "delete"
    document["proposals"][0]["evidence"] = [{"kind": "ignored"}]
    with pytest.raises(ProposalError, match="no quote"):
        validate_proposals(document)


def test_a_write_failure_is_a_named_error_with_exit_1(capsys, tmp_path):
    code = main(["propose", str(FIX / "rules_synthetic.md"),
                 str(FIX / "observations_synthetic.json"),
                 "-o", str(tmp_path / "missing" / "out.json")])
    assert code == 1
    assert "cannot write the proposal file" in capsys.readouterr().err


def test_stdout_mode_writes_the_document(capsys):
    assert main(["propose", str(FIX / "rules_synthetic.md"),
                 str(FIX / "observations_synthetic.json")]) == 0
    document = json.loads(capsys.readouterr().out)
    assert document["proposal_version"] == "1.0"


def test_a_first_half_rule_in_a_later_section_does_not_promote():
    text = "# A\n\n- r1\n\n## B\n\n- r2\n- r3\n- r4\n- r5\n- r6\n"
    rules = parse_rules(text)[0]
    obs = [{"rule": "r2", "kind": "followed", "quote": "a"},
           {"rule": "r2", "kind": "followed", "quote": "b"}]
    # r2 is at index 1 of 6, first half, though outside the
    # first section: position alone must block the promote.
    assert build_document(b"-", rules, obs, [])["proposals"] == []


def test_a_rules_file_with_a_bom_still_parses(tmp_path):
    rules = tmp_path / "rules.md"
    rules.write_bytes(b"\xef\xbb\xbf# S\n\n- A rule\n")
    out = tmp_path / "out.json"
    empty = tmp_path / "empty.json"
    empty.write_text(json.dumps({"observations": []}))
    assert main(["propose", str(rules), str(empty),
                 "-o", str(out)]) == 0
    document = json.loads(out.read_text())
    assert document["inventory"][0]["text"] == "A rule"


def test_an_unencodable_quote_is_a_named_write_error(capsys, tmp_path):
    obs = tmp_path / "obs.json"
    obs.write_text(json.dumps({"observations": [
        {"rule": "Never force push to main", "kind": "ignored",
         "quote": "bad \ud800 quote"},
        {"rule": "Never force push to main", "kind": "ignored",
         "quote": "fine quote"}]}))
    code = main(["propose", str(FIX / "rules_synthetic.md"),
                 str(obs), "-o", str(tmp_path / "out.json")])
    assert code == 1
    assert "cannot write the proposal file" in capsys.readouterr().err
