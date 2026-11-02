"""Score paired agent trials using freshly observed test outcomes, never model claims.

No model calls or user-store access. Real model runs must be supplied separately.
"""
import argparse
import json
from collections import defaultdict


def evaluate(trials, min_pairs=20):
    if type(min_pairs) is not int or min_pairs < 1:
        raise ValueError("min_pairs must be positive")
    pairs = defaultdict(dict)
    for trial in trials:
        required = {"task", "trial", "condition", "kind", "attempts", "fresh_test_exit", "misleading_hint", "repository", "commit", "test_command"}
        if not required <= trial.keys():
            raise ValueError("missing trial evidence fields")
        condition = trial["condition"]
        if condition not in {"cold", "memory"} or trial["kind"] not in {"fixture", "model"}:
            raise ValueError("invalid condition or kind")
        if type(trial["attempts"]) is not int or trial["attempts"] < 1:
            raise ValueError("attempts must be a positive integer")
        if type(trial["fresh_test_exit"]) is not int or type(trial["misleading_hint"]) is not bool:
            raise ValueError("exit must be an integer and misleading_hint a boolean")
        if any(not isinstance(trial[k], str) or not trial[k].strip() for k in ("repository", "commit", "test_command")):
            raise ValueError("provenance must be nonempty")
        key = (trial["task"], trial["trial"])
        if condition in pairs[key]:
            raise ValueError("duplicate condition")
        pairs[key][condition] = trial
    if not pairs or any(set(pair) != {"cold", "memory"} for pair in pairs.values()):
        raise ValueError("complete cold/memory pairs required")
    totals = {c: {"solved": 0, "wasted_attempts": 0, "misleading_hints": 0} for c in ("cold", "memory")}
    kinds = set()
    for pair in pairs.values():
        a, b = pair["cold"], pair["memory"]
        if any(a[k] != b[k] for k in ("kind", "repository", "commit", "test_command")):
            raise ValueError("paired runs must share kind and starting test provenance")
        kinds.add(a["kind"])
        for condition, trial in pair.items():
            solved = trial["fresh_test_exit"] == 0
            totals[condition]["solved"] += solved
            totals[condition]["wasted_attempts"] += trial["attempts"] - int(solved)
            totals[condition]["misleading_hints"] += int(trial["misleading_hint"])
    # Fixture-only or mixed results can never authorize a model-benefit claim.
    passing = len(pairs) >= min_pairs and kinds == {"model"} and totals["memory"]["solved"] > totals["cold"]["solved"] and totals["memory"]["wasted_attempts"] <= totals["cold"]["wasted_attempts"] and totals["memory"]["misleading_hints"] == 0
    return {"pairs": len(pairs), "minimum_pairs": min_pairs, "kinds": sorted(kinds), "metrics": totals, "release_gate_passed": passing}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("trials", help="JSONL trials; no model/API is invoked")
    args = parser.parse_args()
    with open(args.trials) as source:
        report = evaluate([json.loads(line) for line in source if line.strip()])
    print(json.dumps(report, indent=2))
    raise SystemExit(0 if report["release_gate_passed"] else 1)
