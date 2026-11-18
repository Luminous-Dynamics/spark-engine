# Decision-stability synthetic controls

**Status:** hand-authored synthetic benchmark seeds with a Rust known-answer oracle wired into the targeted CI test step; no successful exact-head run is claimed and no tournament/scoring executable exists yet.  
**Parent:** [FUSION-DECISION-ROBUSTNESS-001 #44](https://github.com/Luminous-Dynamics/spark-engine/issues/44).  
**Claim ceiling:** software/decision semantics only. No fixture represents a fusion design, physical solver output, measured residual, calibrated probability, or plant performance.

## Fixtures

- `synthetic-model-choice-cliff-v0.1.json` — two matched synthetic model variants cause candidate A to cross a hard clearance constraint while candidate B remains feasible. There is no probability assigned to the two model variants.
- `synthetic-decision-irrelevant-disagreement-v0.1.json` — two matched variants differ by 21 MWe for candidate A, but both remain on the same side of every hard gate; disagreement is recorded without a false decision split.
- `synthetic-common-mode-consensus-v0.1.json` — three model labels agree closely but share one synthetic closure/calibration lineage and disagree with a hidden reference. Agreement cannot be counted as three independent confirmations.
- `synthetic-conformal-coverage-arithmetic-v0.1.json` — hand-computed finite-sample conformal rank and conservative union-bound controls. It is arithmetic only, not an empirical calibration corpus.
- `synthetic-ood-refusal-v0.1.json` — a numeric model output is present at a requested state outside the declared applicability interval; the only accepted disposition is `OutOfDomain` / `Indeterminate`, with no adjusted robust lower bound.

These are separate research fixtures rather than a second general case-package schema. When a source-pinned solver/measurement case is admitted, the Spark case-package contract remains the outer evidence envelope.

## Admission rules

Before these fixtures are used by an evaluator:
1. Validate JSON syntax and freeze exact fixture digests.
2. Implement an independent hand-calculated oracle separate from the system under test.
3. Test deliberate mutations of thresholds, model ordering, dependency groups, coverage ranks and status fields.
4. Preserve every input/model/candidate result and expected refusal.
5. Wire the exact fixtures into Rust unit tests and CI only after their report contract is reviewed.
6. The current Rust tests cover five synthetic decision controls, including OOD refusal, plus hostile empty-set and missing/duplicate-pair mutations; they do not score real solver runs or establish measured calibration.
7. Never promote the synthetic class or these fixture digests into measured/model-validated evidence.
