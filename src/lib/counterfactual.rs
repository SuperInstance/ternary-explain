//! Counterfactual: "what would need to change for the agent to choose differently?"

use crate::lib::types::*;
use crate::lib::trace::*;
use crate::lib::feature::*;

/// A suggested feature change that could flip the decision.
#[derive(Debug, Clone)]
pub struct FeatureChange {
    /// Feature name.
    pub feature: String,
    /// Current value.
    pub current_value: f64,
    /// Required value to flip the decision.
    pub required_value: f64,
    /// The magnitude of change needed.
    pub delta: f64,
    /// How feasible this change is (0.0 = impossible, 1.0 = trivial).
    pub feasibility: f64,
}

impl FeatureChange {
    /// Create a new feature change.
    pub fn new(feature: impl Into<String>, current: f64, required: f64, feasibility: f64) -> Self {
        Self {
            feature: feature.into(),
            current_value: current,
            required_value: required,
            delta: required - current,
            feasibility: feasibility.clamp(0.0, 1.0),
        }
    }
}

/// A counterfactual analysis result.
#[derive(Debug, Clone)]
pub struct Counterfactual {
    /// The original decision action.
    pub original_action: TernaryAction,
    /// The alternative action being considered.
    pub alternative_action: TernaryAction,
    /// Feature changes that could flip the decision.
    pub changes: Vec<FeatureChange>,
    /// Human-readable summary.
    pub summary: String,
}

impl Counterfactual {
    /// Create a new counterfactual.
    pub fn new(original: TernaryAction, alternative: TernaryAction) -> Self {
        Self {
            original_action: original,
            alternative_action: alternative,
            changes: Vec::new(),
            summary: String::new(),
        }
    }

    /// Add a feature change.
    pub fn add_change(&mut self, change: FeatureChange) {
        self.changes.push(change);
    }

    /// Sort changes by feasibility (most feasible first).
    pub fn sorted_by_feasibility(&self) -> Vec<&FeatureChange> {
        let mut v: Vec<_> = self.changes.iter().collect();
        v.sort_by(|a, b| b.feasibility.partial_cmp(&a.feasibility).unwrap_or(std::cmp::Ordering::Equal));
        v
    }

    /// Get the minimum-change counterfactual (fewest/smallest changes).
    pub fn minimal(&self) -> Vec<&FeatureChange> {
        self.sorted_by_feasibility()
    }
}

/// Counterfactual analysis engine.
pub struct CounterfactualAnalyzer;

impl CounterfactualAnalyzer {
    /// Analyze what features would need to change for a different action.
    ///
    /// Uses a simplified SHAP-based approach: for each feature, estimate
    /// how much its value would need to shift to make the alternative
    /// action's total score exceed the original action's score.
    pub fn analyze(
        trace: &DecisionTrace,
        contributions: &ContributionSet,
        target_action: TernaryAction,
    ) -> Counterfactual {
        let mut cf = Counterfactual::new(trace.action, target_action);

        if trace.action == target_action {
            cf.summary = "The agent already chose this action.".to_string();
            return cf;
        }

        // Get score gap we need to overcome
        let original_score = trace.scores.iter()
            .find(|s| s.action == trace.action)
            .map(|s| s.score)
            .unwrap_or(0.0);
        let target_score = trace.scores.iter()
            .find(|s| s.action == target_action)
            .map(|s| s.score)
            .unwrap_or(0.0);
        let gap = original_score - target_score;

        // For each feature, estimate the change needed
        for contrib in &contributions.contributions {
            let current_push = contrib.for_action(trace.action) - contrib.for_action(target_action);
            if current_push <= 0.0 {
                continue; // This feature already favors the target
            }

            // Find the input value for this feature
            let input_val = trace.inputs.iter()
                .find(|i| i.name == contrib.feature)
                .map(|i| i.value)
                .unwrap_or(1.0);

            // Estimate: if we reverse the push direction, how much do we need?
            let push_ratio = if current_push.abs() > f64::EPSILON {
                gap / current_push
            } else {
                continue;
            };

            let required_value = input_val - push_ratio * input_val;
            let delta = required_value - input_val;

            // Estimate feasibility: smaller relative changes are more feasible
            let relative_change = if input_val.abs() > f64::EPSILON {
                (delta / input_val).abs()
            } else {
                f64::INFINITY
            };
            let feasibility = 1.0 / (1.0 + relative_change);

            cf.add_change(FeatureChange::new(
                &contrib.feature,
                input_val,
                required_value,
                feasibility,
            ));
        }

        // Generate summary
        if cf.changes.is_empty() {
            cf.summary = format!(
                "No single feature change could flip the decision from {} to {}.",
                trace.action.label(),
                target_action.label()
            );
        } else {
            let best = cf.sorted_by_feasibility();
            let top = &best[0];
            cf.summary = format!(
                "To flip from {} to {}: change '{}' from {:.4} to {:.4} (delta={:.4}, feasibility={:.0}%)",
                trace.action.label(),
                target_action.label(),
                top.feature,
                top.current_value,
                top.required_value,
                top.delta,
                top.feasibility * 100.0,
            );
        }

        cf
    }

    /// Analyze all alternative actions.
    pub fn analyze_all(
        trace: &DecisionTrace,
        contributions: &ContributionSet,
    ) -> Vec<Counterfactual> {
        TernaryAction::all()
            .iter()
            .filter(|&&a| a != trace.action)
            .map(|&a| Self::analyze(trace, contributions, a))
            .collect()
    }
}
