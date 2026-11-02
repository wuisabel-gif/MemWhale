import unittest
from evaluate import evaluate


def pair(kind="fixture"):
    base = dict(task="synthetic-calculator", trial=1, kind=kind, repository="synthetic", commit="fixture-base", test_command="python -m unittest", misleading_hint=False)
    return [dict(base, condition="cold", attempts=2, fresh_test_exit=1), dict(base, condition="memory", attempts=1, fresh_test_exit=0)]


class GateTests(unittest.TestCase):
    def test_fixtures_cannot_pass_model_gate(self):
        report = evaluate(pair())
        self.assertFalse(report["release_gate_passed"])
        self.assertEqual(report["metrics"]["cold"]["wasted_attempts"], 2)

    def test_improved_model_pair_passes_but_misleading_hint_blocks(self):
        trials = pair("model")
        self.assertFalse(evaluate(trials)["release_gate_passed"])
        self.assertTrue(evaluate(trials, min_pairs=1)["release_gate_passed"])
        trials[1]["misleading_hint"] = True
        self.assertFalse(evaluate(trials, min_pairs=1)["release_gate_passed"])

    def test_missing_duplicate_or_mismatched_pairs_rejected(self):
        trials = pair()
        for invalid in [[], trials[:1], trials + trials[:1], [trials[0], dict(trials[1], commit="other")]]:
            with self.assertRaises(ValueError):
                evaluate(invalid)

    def test_no_gain_does_not_pass(self):
        trials = pair("model")
        trials[0]["fresh_test_exit"] = 0
        self.assertFalse(evaluate(trials)["release_gate_passed"])


if __name__ == "__main__":
    unittest.main()
