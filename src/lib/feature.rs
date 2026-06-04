//! FeatureContribution: per-feature contribution to the final decision (SHAP-like).

use crate::lib::types::*;

/// The contribution of a single feature to the decision outcome.
#[derive(Debug, Clone)]
pub struct FeatureContribution {
    /// Feature name.
    pub feature: String,
    /// Contribution to the negative (-1) action.
    pub neg_contribution: f64,
    /// Contribution to the neutral (0) action.
    pub neu_contribution: f64,
    /// Contribution to the positive (+1) action.
    pub pos_contribution: f64,
}

impl FeatureContribution {
    /// Create a new feature contribution.
    pub fn new(feature: impl Into<String>, neg: f64, neu: f64, pos: f64) -> Self {
        Self {
            feature: feature.into(),
            neg_contribution: neg,
            neu_contribution: neu,
            pos_contribution: pos,
        }
    }

    /// Get the contribution for a specific action.
    pub fn for_action(&self, action: TernaryAction) -> f64 {
        match action {
            TernaryAction::Negative => self.neg_contribution,
            TernaryAction::Neutral => self.neu_contribution,
            TernaryAction::Positive => self.pos_contribution,
        }
    }

    /// The action this feature pushes toward most strongly.
    pub fn dominant_direction(&self) -> TernaryAction {
        let max = self.neg_contribution
            .max(self.neu_contribution)
            .max(self.pos_contribution);
        if (self.pos_contribution - max).abs() < f64::EPSILON {
            TernaryAction::Positive
        } else if (self.neu_contribution - max).abs() < f64::EPSILON {
            TernaryAction::Neutral
        } else {
            TernaryAction::Negative
        }
    }

    /// Absolute magnitude of this feature's influence (sum of absolute contributions).
    pub fn magnitude(&self) -> f64 {
        self.neg_contribution.abs()
            + self.neu_contribution.abs()
            + self.pos_contribution.abs()
    }

    /// Net push: positive contribution minus negative contribution.
    pub fn net_push(&self) -> f64 {
        self.pos_contribution - self.neg_contribution
    }
}

/// A complete set of feature contributions for a decision.
#[derive(Debug, Clone)]
pub struct ContributionSet {
    /// The action that was chosen.
    pub chosen_action: TernaryAction,
    /// Per-feature contributions.
    pub contributions: Vec<FeatureContribution>,
}

impl ContributionSet {
    /// Create a new contribution set.
    pub fn new(chosen_action: TernaryAction) -> Self {
        Self {
            chosen_action,
            contributions: Vec::new(),
        }
    }

    /// Add a feature contribution.
    pub fn add(&mut self, c: FeatureContribution) {
        self.contributions.push(c);
    }

    /// Sort contributions by magnitude (largest first).
    pub fn sorted_by_magnitude(&self) -> Vec<&FeatureContribution> {
        let mut v: Vec<_> = self.contributions.iter().collect();
        v.sort_by(|a, b| b.magnitude().partial_cmp(&a.magnitude()).unwrap_or(std::cmp::Ordering::Equal));
        v
    }

    /// Get the top N most influential features for the chosen action.
    pub fn top_features(&self, n: usize) -> Vec<&FeatureContribution> {
        let mut sorted = self.sorted_by_magnitude();
        sorted.truncate(n);
        sorted
    }

    /// Features that support the chosen action (positive contribution to it).
    pub fn supporting_features(&self) -> Vec<&FeatureContribution> {
        self.contributions
            .iter()
            .filter(|c| c.for_action(self.chosen_action) > 0.0)
            .collect()
    }

    /// Features that oppose the chosen action (push toward a different action).
    pub fn opposing_features(&self) -> Vec<&FeatureContribution> {
        let action = self.chosen_action;
        self.contributions
            .iter()
            .filter(|c| {
                let other_max = TernaryAction::all()
                    .iter()
                    .filter(|&&a| a != action)
                    .map(|&a| c.for_action(a))
                    .fold(f64::NEG_INFINITY, f64::max);
                other_max > c.for_action(action)
            })
            .collect()
    }
}
