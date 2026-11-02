"""BP-5, BP-5a, BP-6, BP-7. Inventory and evidence join."""
import json
from pathlib import Path

import pytest

from bluepencil.inventory import InventoryError, parse_rules  # noqa: F401
from bluepencil.observations import load_observations
from bluepencil.proposals import build_document

FIX = Path(__file__).parent / "fixtures"
RULES = (FIX / "rules_synthetic.md").read_bytes()


def test_every_rule_is_inventoried_with_position_section_text_span():
    rules = parse_rules(RULES.decode())[0]
    assert [r["id"] for r in rules] == [f"rule-{i}" for i in range(1, 8)]
    assert rules[0]["section"] == ["Setup rules"]
    assert rules[0]["text"] == "Always run the local gate before pushing"
    assert rules[3]["section"] == ["Setup rules", "Testing"]
    assert rules[3]["line_span"] == [11, 12]
    assert rules[6]["section"] == ["Setup rules", "Style"]


def test_prose_and_fenced_lines_are_never_rules():
    rules = parse_rules(RULES.decode())[0]
    texts = [r["text"] for r in rules]
    assert not any("fossil" in t or "inside a fence" in t for t in texts)
    assert not any("Read these before" in t for t in texts)


def test_a_continuation_line_joins_its_rules_text():
    rules = parse_rules(RULES.decode())[0]
    assert rules[3]["text"] == ("Run the full suite locally, never "
                                "trust CI first\nincluding the slow "
                                "integration half")


def test_a_rule_before_any_heading_refuses_by_name():
    with pytest.raises(InventoryError, match="outside any section"):
        parse_rules("- A rule with no home\n\n# Later\n\n- Fine\n")


def test_numbered_lines_are_rules_and_nesting_is_kept_flat_in_order():
    text = "# S\n\n1. First\n2. Second\n   - Nested third\n"
    rules = parse_rules(text)[0]
    assert [r["text"] for r in rules] == ["First", "Second",
                                          "Nested third"]


def test_observations_join_by_exact_text_and_unmatched_are_reported():
    observations, _ = load_observations(FIX / "observations_synthetic.json")
    document = build_document(RULES, parse_rules(RULES.decode())[0],
                              observations, [])
    assert len(document["unmatched_observations"]) == 1
    assert document["unmatched_observations"][0]["rule"].startswith(
        "A rule that was edited away")
    counts = {e["id"]: e["counts"] for e in document["inventory"]}
    assert counts["rule-2"] == {"misreading": 0, "ignored": 2,
                                "followed": 0, "exercised": 2}
    assert counts["rule-1"]["exercised"] == 2


def test_the_counting_method_is_stated_in_the_output():
    document = build_document(RULES, parse_rules(RULES.decode())[0], [], [])
    assert "exact rule text" in document["counting_method"]


def test_a_duplicate_text_is_flagged_and_counts_attach_to_the_first():
    text = "# S\n\n- Same words\n- Same words\n"
    rules = parse_rules(text)[0]
    document = build_document(text.encode(), rules,
                              [{"rule": "Same words", "kind": "ignored",
                                "quote": "q"}], [])
    first, second = document["inventory"]
    assert first["counts"]["ignored"] == 1
    assert second["counts"]["ignored"] == 0
    assert second["duplicate_of"] == "rule-1"


def test_malformed_observations_refuse_by_name(tmp_path):
    bad = tmp_path / "bad.json"
    bad.write_text(json.dumps({"observations": [
        {"rule": "x", "kind": "shrugged", "quote": "q"}]}))
    from bluepencil.observations import ObservationError
    with pytest.raises(ObservationError, match="unknown kind"):
        load_observations(bad)


def test_bullets_inside_an_html_comment_are_not_rules():
    text = ("# S\n\n<!--\n- A phantom rule from an example\n"
            "- Another phantom\n-->\n\n- A real rule\n")
    rules, _ = parse_rules(text)
    assert [r["text"] for r in rules] == ["A real rule"]


def test_an_empty_marker_refuses_by_name():
    with pytest.raises(InventoryError, match="empty rule at line 3"):
        parse_rules("# S\n\n-\n")


def test_a_fence_closes_only_on_its_own_character_and_length():
    text = ("# S\n\n````\n```\n- Inside the long fence\n~~~\n````\n"
            "\n- After the fence\n")
    rules, _ = parse_rules(text)
    assert [r["text"] for r in rules] == ["After the fence"]


def test_an_unclosed_fence_warns_and_swallows_nothing_silently():
    rules, warnings = parse_rules("# S\n\n- Kept\n\n```\n- Lost\n")
    assert [r["text"] for r in rules] == ["Kept"]
    assert warnings and "unclosed fence at line 5" in warnings[0]


def test_setext_headings_open_sections():
    text = ("Top section\n===========\n\n- Rule one\n\n"
            "Sub section\n-----------\n\n- Rule two\n")
    rules, _ = parse_rules(text)
    assert rules[0]["section"] == ["Top section"]
    assert rules[1]["section"] == ["Top section", "Sub section"]


def test_closing_hashes_are_not_part_of_a_section_name():
    rules, _ = parse_rules("## Style ##\n\n- A rule\n")
    assert rules[0]["section"] == ["Style"]


def test_a_task_list_checkbox_is_stripped_from_the_rule_text():
    rules, _ = parse_rules("# S\n\n- [ ] Do the thing\n- [x] Done thing\n")
    assert [r["text"] for r in rules] == ["Do the thing", "Done thing"]


def test_a_continuation_after_a_blank_line_still_joins_its_rule():
    text = "# S\n\n- A rule that\n\n  continues here\n- Next\n"
    rules, _ = parse_rules(text)
    assert rules[0]["text"] == "A rule that\ncontinues here"
    assert rules[0]["line_span"] == [3, 5]


def test_a_near_miss_observation_is_unmatched_not_attached():
    rules, _ = parse_rules("# S\n\n- Keep functions short\n")
    document = build_document(b"-", rules, [
        {"rule": "keep functions short", "kind": "ignored",
         "quote": "lowercase differs"}], [])
    assert len(document["unmatched_observations"]) == 1
    assert document["inventory"][0]["counts"]["exercised"] == 0
