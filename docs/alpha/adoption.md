# Adoption plan

alpha adopted this repo. These steps bring its code up to the principles in [principles.toml](principles.toml), phase by phase. Run `/alpha:install` to pick up the next one: each step gets a branch of its own, and it's done only when everything under its *Done when* holds.

## Progress

| Phase | Done | Steps |
| --- | --- | --- |
| P4 Rework | 0 | 9 |
| P5 Burn-down | 0 | 3 |

## P4 Rework

| Step | Component | Status |
| --- | --- | --- |
| `p4-ty-banned-crates` | extract | pending |
| `p4-guidelines-refs` | - | pending |
| `p4-plan-layout` | mentalc | pending |
| `p4-timeouts` | mentalc | pending |
| `p4-trim-later` | mentalc | pending |
| `p4-golden-model` | - | pending |
| `p4-project-principles` | - | pending |
| `p4-diagrams` | - | pending |
| `p4-glossary-verdicts` | - | pending |

### `p4-ty-banned-crates`: Resolve the ty vs banned-crates conflict

G16's banned_crates tidy test bans ty_*, salsa and rand, but X1 and the extract allowlist need ty's crates, which pull in salsa, rand and rayon (about 240 crates). Milestone 2 could only pass by weakening the check.

Serves `never-weaken-checks`, `dependency-reasons`.

Done when:

- the user confirms: The owner has decided how G16 and X1 fit together, and docs/guidelines/README.md, docs/guidelines/extract.md and docs/specification.md say the same
- `! grep -nE 'ty_\*' docs/guidelines/README.md | grep -i ban` passes

### `p4-guidelines-refs`: Sync guideline and resolution references with the rewritten spec

model.md cites old spec sections and plan D7/D9 from the deleted model plan and gives milestone 1 the wrong acceptance; G11, C24 and X22 point at old sections or milestones; resolution.md cites an undefined E4 and skips section 15; mentalvm.md names symbolic runs, which are Later.

Serves `docs-with-code`, `one-vocabulary`.

Done when:

- `! grep -nE 'spec (4\.4|4\.8)|plan D[79]|rejects seeded errors|ordered-reads rule' docs/guidelines/model.md && ! grep -nE 'spec 3 and 6\.5|spec 8\.2 and 6\.8' docs/guidelines/*.md && ! grep -n '(E4)' docs/resolution.md && ! grep -n 'concrete or symbolic' docs/guidelines/mentalvm.md` passes

### `p4-plan-layout`: Fix the milestone 1 plan layout

mentalc/src/build.rs writes .mental/ but is not an I/O edge under G4; the plan has no mentalc/src/lib.rs with run() (G12); packs have two homes, extract/packs and mentalc/packs, though extract does no I/O.

Serves `pure-core`, `lean-structure`. After `p4-guidelines-refs`.

Done when:

- `grep -q 'lib.rs' docs/plans/milestone-1-build-model.md && ! grep -q 'extract/packs' docs/plans/milestone-1-build-model.md && ! grep -nE 'build.rs .*write .mental' docs/plans/milestone-1-build-model.md` passes

### `p4-timeouts`: Add timeouts to the mentalc guidelines

mentalc makes network calls, but C6 sets retries with no request timeout and docker compose config has no time limit, so a stalled daemon or socket hangs mentalc build instead of exiting 2 with a fix.

Serves `timeouts`, `fail-helpfully`.

Done when:

- `grep -qi 'timeout' docs/guidelines/mentalc.md && grep -n 'docker compose config' docs/guidelines/mentalc.md docs/plans/milestone-1-build-model.md | grep -qi 'time'` passes

### `p4-trim-later`: Trim Later items from the mentalc guidelines

C5 lists the local and agent-cli backends and the layout has backend/agent_cli.rs, which spec 9.3 puts in Later; C17 adds an optional mental_build_status tool nobody asked for.

Serves `smallest-change`.

Done when:

- `! grep -nE 'agent_cli\.rs|`local`, `agent-cli`|optionally `mental_build_status`' docs/guidelines/mentalc.md` passes

### `p4-golden-model`: Settle the golden reference model

T6, C24, M22 and M24 name a hand-written reference model at examples/report-jobs/model/, and it is a risk path, but it does not exist, so the tests built on it cannot be written.

Serves `tests-first-class`.

Done when:

- the user confirms: The owner has decided which milestone writes examples/report-jobs/model/, and the guidelines and the milestone plans say so

### `p4-project-principles`: Register the project principles and the exceptions rule

The guidelines define eight project principles (fixed-crate-graph, deterministic-output, fail-rather-than-degrade, no-bloat, one-canonical-form, extractors-never-guess, one-judge-for-every-backend, semantics-stay-in-the-step) that alpha's reviewer does not check; G4, G18 and T16 allow #[expect] and #[mutants::skip] opt-outs scattered in code rather than as rows of CONTRIBUTING.md's Exceptions table.

Serves `simplest-architecture`, `never-weaken-checks`.

Done when:

- `grep -q 'fixed-crate-graph' docs/alpha/principles.toml && grep -q 'deterministic-output' docs/alpha/principles.toml && grep -q 'Exceptions' docs/guidelines/README.md` passes

### `p4-diagrams`: Delete unreferenced diagrams and redraw the rest as SVG

14 of 18 PNGs in docs/images are referenced nowhere; the 4 linked ones have no source, and components.png uses an off-palette blue, overflows a box and shows backends the spec puts in Later.

Serves `generated-from-source`, `no-dead-code`, `design-language`. After `p4-guidelines-refs`.

Done when:

- `test -z "$(ls docs/images/*.png 2>/dev/null)" && for f in docs/images/*.svg; do git grep -q "$(basename $f)" -- '*.md' || exit 1; done` passes

### `p4-glossary-verdicts`: Make the glossary agree with the spec on move verdicts

Spec 8.1 and the glossary's Verdict of a move give atomic, guarded, non-atomic; the glossary's Atomicity entry gives stutter, isolated, guarded, non-atomic.

Serves `one-vocabulary`.

Done when:

- `! grep -qE '\*Stutter:\*|\*Isolated:\*' docs/glossary.md` passes

## P5 Burn-down

| Step | Component | Status |
| --- | --- | --- |
| `p5-baseline-typos` | - | pending |
| `p5-baseline-markdown` | - | pending |
| `p5-baseline-links` | - | pending |

### `p5-baseline-typos`: Clear the typos baseline

Legacy typos findings are baselined (alpha baseline show). Fix them, risk paths first, then remove typos from adoption.baselines so the check runs plain again.

Serves `docs-with-code`.

Done when:

- typos is no longer baselined (not in adoption.baselines)

### `p5-baseline-markdown`: Clear the markdown baseline

Legacy markdown findings are baselined (alpha baseline show). Fix them, risk paths first, then remove markdown from adoption.baselines so the check runs plain again.

Serves `docs-with-code`.

Done when:

- markdown is no longer baselined (not in adoption.baselines)

### `p5-baseline-links`: Clear the links baseline

Legacy links findings are baselined (alpha baseline show). Fix them, risk paths first, then remove links from adoption.baselines so the check runs plain again.

Serves `docs-with-code`.

Done when:

- links is no longer baselined (not in adoption.baselines)
