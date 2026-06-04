//! ExplanationGenerator: human-readable explanations from DecisionTrace.

use crate::lib::trace::*;
use crate::lib::feature::*;

/// Configuration for explanation generation.
#[derive(Debug, Clone)]
pub struct ExplanationConfig {
    /// Include feature contributions in the explanation.
    pub include_features: bool,
    /// Include score breakdown.
    pub include_scores: bool,
    /// Include margin information.
    pub include_margin: bool,
    /// Maximum number of top features to mention.
    pub max_features: usize,
    /// Threshold for flagging "close" decisions.
    pub close_threshold: f64,
}

impl Default for ExplanationConfig {
    fn default() -> Self {
        Self {
            include_features: true,
            include_scores: true,
            include_margin: true,
            max_features: 5,
            close_threshold: 0.1,
        }
    }
}

/// A generated explanation for a ternary decision.
#[derive(Debug, Clone)]
pub struct Explanation {
    /// The decision ID this explains.
    pub decision_id: u64,
    /// One-line summary.
    pub summary: String,
    /// Detailed breakdown paragraphs.
    pub details: Vec<String>,
    /// Whether this was flagged as a close/uncertain decision.
    pub is_close_call: bool,
}

/// Generator for human-readable explanations.
#[derive(Debug, Clone)]
pub struct ExplanationGenerator {
    config: ExplanationConfig,
}

impl ExplanationGenerator {
    /// Create a new generator with the given config.
    pub fn new(config: ExplanationConfig) -> Self {
        Self { config }
    }

    /// Create with default config.
    pub fn default_generator() -> Self {
        Self::new(ExplanationConfig::default())
    }

    /// Generate an explanation from a trace (no feature contributions).
    pub fn explain_trace(&self, trace: &DecisionTrace) -> Explanation {
        let mut details = Vec::new();
        let is_close = trace.is_close(self.config.close_threshold);

        // Summary
        let close_flag = if is_close { " (close call)" } else { "" };
        let confidence_str = trace.confidence
            .map(|c| format!(" (confidence: {:.0}%)", c * 100.0))
            .unwrap_or_default();
        let summary = format!(
            "Decision #{}: chose {}{}.{}",
            trace.id,
            trace.action.label(),
            close_flag,
            confidence_str,
        );

        // Context
        if let Some(ref ctx) = trace.context {
            details.push(format!("Context: {}", ctx));
        }

        // Score breakdown
        if self.config.include_scores && !trace.scores.is_empty() {
            let mut score_parts: Vec<String> = trace.scores.iter().map(|s| {
                let marker = if s.action == trace.action { " ★" } else { "" };
                format!("{}={:.4}{}", s.action.label(), s.score, marker)
            }).collect();
            score_parts.sort();
            details.push(format!("Scores: {}", score_parts.join(", ")));
        }

        // Margin
        if self.config.include_margin {
            if let Some(margin) = trace.margin() {
                if is_close {
                    details.push(format!(
                        "⚠ This was a close decision (margin={:.4} < threshold={:.4})",
                        margin, self.config.close_threshold
                    ));
                } else {
                    details.push(format!("Decision margin: {:.4}", margin));
                }
            }
        }

        // Inputs summary
        if !trace.inputs.is_empty() {
            let input_strs: Vec<String> = trace.inputs.iter().map(|i| {
                format!("{}={:.4}", i.name, i.value)
            }).collect();
            details.push(format!("Inputs: {}", input_strs.join(", ")));
        }

        Explanation {
            decision_id: trace.id,
            summary,
            details,
            is_close_call: is_close,
        }
    }

    /// Generate a full explanation from trace + feature contributions.
    pub fn explain_full(&self, trace: &DecisionTrace, contributions: &ContributionSet) -> Explanation {
        let mut explanation = self.explain_trace(trace);

        if self.config.include_features {
            let top = contributions.top_features(self.config.max_features);
            if !top.is_empty() {
                let mut feature_lines = vec!["Top contributing features:".to_string()];
                for c in top {
                    let direction = c.dominant_direction();
                    let push = c.net_push();
                    let arrow = if push > 0.0 { "→ +" } else if push < 0.0 { "→ -" } else { "→ 0" };
                    feature_lines.push(format!(
                        "  {} (net push {}: {:.4}, magnitude: {:.4})",
                        c.feature, arrow, push, c.magnitude()
                    ));
                    if direction != trace.action {
                        feature_lines.push(format!(
                            "    ⚠ Pushes toward {}, which opposes the chosen action",
                            direction.label()
                        ));
                    }
                }
                explanation.details.extend(feature_lines);
            }

            let opposing = contributions.opposing_features();
            if !opposing.is_empty() {
                explanation.details.push(format!(
                    "{} feature(s) opposed the final decision.",
                    opposing.len()
                ));
            }
        }

        explanation
    }

    /// Format an explanation as a single string.
    pub fn format_explanation(&self, explanation: &Explanation) -> String {
        let mut parts = vec![explanation.summary.clone()];
        for detail in &explanation.details {
            parts.push(detail.clone());
        }
        parts.join("\n")
    }
}
