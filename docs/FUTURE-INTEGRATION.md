# Future Integration: ternary-explain

## Current State
Explainability for ternary agent decisions with `types`, `trace` (decision tracing), `feature` (feature importance), `explanation` (human-readable explanations), `counterfactual` (what-if analysis), `audit` (decision audit trail), and `report` (explanation reports). Answers WHY an agent chose -1, 0, or +1.

## Integration Opportunities

### With Ensign Pattern (Explainable Specialists)
Every `Ensign` specialist produces explanations via ternary-explain. When a code-review ensign rejects a PR, `counterfactual` shows what changes would make it acceptable. When a scheduling ensign prioritizes a job, `feature` shows which factors (deadline, resource availability, fitness score) drove the decision. This makes the git-agent army auditable — every automated action has a human-readable explanation.

### With ternary-cell (Explainable Tick Cycle)
Each cell's tick cycle generates a `trace`. The predict→perceive→surprise sequence is an explanation path: what the cell expected, what it observed, and why it was surprised. `counterfactual` shows what inputs would have avoided the surprise. This transforms cell debugging from "why did this cell die?" to "here's what would have kept it alive."

### With negative-space-core (Explaining Avoidance)
When `AvoidanceTracker` records a decision, ternary-explain's `feature` module identifies which features drove the avoidance. The 294:1 ratio means most decisions are avoidances — explaining avoidances is more valuable than explaining choices. `explanation` generates: "Agent avoided Option X because features A, B, C matched its avoidance profile with 0.92 confidence."

## Potential in Mature Systems
Every decision in the system is explainable. The audit trail (`audit` module) is the compliance layer — every automated action has a tamper-proof log of why it happened. `report` generates daily summaries: "Today the fleet made 1.2M decisions. 99.7% were avoidances (matching conservation law). The top 5 surprising decisions were..." This is the system's conscience and accountability mechanism.

## Cross-Pollination Ideas
- `counterfactual` analysis could feed back into `ternary-fitness` for landscape exploration — "what if we tried this unexplored region?"
- `trace` data serialized via `ternary-protocol` enables distributed explanation — trace a decision across rooms
- `report` could render as `ternary-spreadsheet` rows for non-technical users
- `feature` importance connects to `ternary-entropy` — high-entropy features are less informative for explanations

## Dependencies for Next Steps
- Integration with ternary-ensign's trait for automatic explanation generation
- Storage backend for `audit` trail (append-only log)
- ternary-protocol message types for explanation exchange between rooms
- Visualization layer for `report` output (connect to spreadsheet heatmap)
