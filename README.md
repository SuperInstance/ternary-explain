# ternary-explain

**Explainability for ternary agent decisions** — _Why did the agent choose -1, 0, or +1?_

When an AI agent makes ternary decisions (negative/neutral/positive), it's not enough to know _what_ it chose. You need to know _why_. This crate provides a complete explainability framework for tracing, auditing, and understanding ternary decisions.

## Explainability Methodology

### SHAP-Inspired Feature Contributions

Each feature's contribution to the final decision is decomposed into per-action contributions, inspired by [SHAP (SHapley Additive exPlanations)](https://arxiv.org/abs/1705.07874):

- **Per-feature decomposition**: Every input feature gets a contribution score for each of the three actions (-1, 0, +1)
- **Net push**: The difference between positive and negative contributions indicates overall direction
- **Magnitude**: The total absolute contribution indicates how influential a feature is
- **Supporting vs. opposing**: Features are classified by whether they push toward or against the chosen action

### Decision Tracing

Every decision records the complete reasoning chain:

1. **Inputs** → raw feature values entering the decision
2. **Scores** → numerical scores for each possible action
3. **Action** → the final chosen action
4. **Confidence** → how certain the agent was
5. **Margin** → gap between the winning and second-best score

### Counterfactual Analysis

For any decision, we can answer: _"What would need to change for the agent to choose differently?"_

- Identifies which features would need to change
- Estimates the required value for each feature
- Ranks changes by feasibility (smaller relative changes = more feasible)

### Audit Trail

All decisions are logged with timestamps, enabling:
- Action distribution analysis (is the agent biased?)
- Close-call detection (decisions with thin margins)
- Time-range filtering
- Confidence tracking over time

## Usage

```rust
use ternary_explain::*;

// Build a decision trace
let trace = DecisionTrace::new(1, 1700000000000, TernaryAction::Positive)
    .with_context("market_open")
    .with_input(FeatureValue::new("sentiment", 0.85))
    .with_input(FeatureValue::new("momentum", 0.3))
    .with_scores(0.1, 0.25, 0.65)
    .with_confidence(0.78);

// Compute feature contributions (SHAP-like)
let mut contributions = ContributionSet::new(TernaryAction::Positive);
contributions.add(FeatureContribution::new("sentiment", -0.05, 0.1, 0.7));
contributions.add(FeatureContribution::new("momentum", 0.1, 0.15, 0.4));

// Generate human-readable explanation
let generator = ExplanationGenerator::default_generator();
let explanation = generator.explain_full(&trace, &contributions);
println!("{}", generator.format_explanation(&explanation));

// Counterfactual: what would flip this to Negative?
let cf = CounterfactualAnalyzer::analyze(&trace, &contributions, TernaryAction::Negative);
println!("{}", cf.summary);

// Audit log
let mut audit = AuditLog::new();
audit.log_full(trace, contributions);

// Generate report
let reporter = ReportGenerator::default_generator();
let report = reporter.generate("Daily Trading Report", &audit);
println!("{}", report.text);
```

## Core Components

| Component | Description |
|-----------|-------------|
| `DecisionTrace` | Full reasoning chain: inputs → features → scores → action |
| `FeatureContribution` | Per-feature contribution to the decision (SHAP-like) |
| `ExplanationGenerator` | Human-readable explanations from traces |
| `CounterfactualAnalyzer` | "What would need to change?" analysis |
| `AuditLog` | Timestamped log of all decisions |
| `ExplainReport` | Formatted report with patterns, edge cases, distributions |

## Design Principles

- **Pure Rust, no unsafe, no external deps** — easy to audit and embed anywhere
- **Builder pattern** — ergonomic API for constructing traces and explanations
- **No-std friendly** (minimal allocation, no system dependencies)
- **Explainability-first** — every component designed for human understanding

## License

MIT
