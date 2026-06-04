//! DecisionTrace: full reasoning chain for a single decision.

use crate::lib::types::*;

/// The full reasoning chain for a single ternary decision.
///
/// Records inputs → features → scores → final action, providing
/// complete traceability for explainability.
#[derive(Debug, Clone)]
pub struct DecisionTrace {
    /// Unique identifier for this decision.
    pub id: u64,
    /// Timestamp of the decision.
    pub timestamp: Timestamp,
    /// Optional context label (e.g., "market_open", "risk_check").
    pub context: Option<String>,
    /// Input features used in the decision.
    pub inputs: Vec<FeatureValue>,
    /// Scores for each possible action.
    pub scores: Vec<ActionScore>,
    /// The final chosen action.
    pub action: TernaryAction,
    /// Optional confidence level (0.0 to 1.0).
    pub confidence: Option<f64>,
    /// Optional metadata key-value pairs.
    pub metadata: Vec<(String, String)>,
}

impl DecisionTrace {
    /// Create a new trace.
    pub fn new(id: u64, timestamp: Timestamp, action: TernaryAction) -> Self {
        Self {
            id,
            timestamp,
            context: None,
            inputs: Vec::new(),
            scores: Vec::new(),
            action,
            confidence: None,
            metadata: Vec::new(),
        }
    }

    /// Add context label.
    pub fn with_context(mut self, ctx: impl Into<String>) -> Self {
        self.context = Some(ctx.into());
        self
    }

    /// Add an input feature.
    pub fn with_input(mut self, feature: FeatureValue) -> Self {
        self.inputs.push(feature);
        self
    }

    /// Add multiple input features.
    pub fn with_inputs(mut self, features: Vec<FeatureValue>) -> Self {
        self.inputs.extend(features);
        self
    }

    /// Add an action score.
    pub fn with_score(mut self, score: ActionScore) -> Self {
        self.scores.push(score);
        self
    }

    /// Add all three action scores.
    pub fn with_scores(mut self, neg: f64, neu: f64, pos: f64) -> Self {
        self.scores = vec![
            ActionScore { action: TernaryAction::Negative, score: neg },
            ActionScore { action: TernaryAction::Neutral, score: neu },
            ActionScore { action: TernaryAction::Positive, score: pos },
        ];
        self
    }

    /// Set confidence.
    pub fn with_confidence(mut self, c: f64) -> Self {
        self.confidence = Some(c.clamp(0.0, 1.0));
        self
    }

    /// Add metadata.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.push((key.into(), value.into()));
        self
    }

    /// Get the winning score value.
    pub fn winning_score(&self) -> Option<f64> {
        self.scores
            .iter()
            .find(|s| s.action == self.action)
            .map(|s| s.score)
    }

    /// Get the margin between the winning score and the second-best.
    pub fn margin(&self) -> Option<f64> {
        if self.scores.len() < 2 {
            return None;
        }
        let mut sorted: Vec<f64> = self.scores.iter().map(|s| s.score).collect();
        sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        Some(sorted[0] - sorted[1])
    }

    /// Check if this was a "close" decision (margin below threshold).
    pub fn is_close(&self, threshold: f64) -> bool {
        self.margin().map_or(false, |m| m < threshold)
    }
}
