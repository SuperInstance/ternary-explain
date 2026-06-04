//! AuditLog: timestamped log of all decisions with explanations.

use crate::lib::types::*;
use crate::lib::trace::*;
use crate::lib::feature::*;
use crate::lib::explanation::*;

/// A single audit entry combining a trace, contributions, and explanation.
#[derive(Debug, Clone)]
pub struct AuditEntry {
    /// The decision trace.
    pub trace: DecisionTrace,
    /// Optional feature contributions.
    pub contributions: Option<ContributionSet>,
    /// The generated explanation.
    pub explanation: Explanation,
}

/// A timestamped audit log of decisions.
#[derive(Debug, Clone)]
pub struct AuditLog {
    /// Entries in chronological order.
    entries: Vec<AuditEntry>,
    /// The explanation generator.
    generator: ExplanationGenerator,
}

impl AuditLog {
    /// Create a new audit log with default settings.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            generator: ExplanationGenerator::default_generator(),
        }
    }

    /// Create with custom explanation config.
    pub fn with_config(config: ExplanationConfig) -> Self {
        Self {
            entries: Vec::new(),
            generator: ExplanationGenerator::new(config),
        }
    }

    /// Log a decision with trace only (no feature contributions).
    pub fn log_trace(&mut self, trace: DecisionTrace) {
        let explanation = self.generator.explain_trace(&trace);
        self.entries.push(AuditEntry {
            trace,
            contributions: None,
            explanation,
        });
    }

    /// Log a decision with trace and feature contributions.
    pub fn log_full(&mut self, trace: DecisionTrace, contributions: ContributionSet) {
        let explanation = self.generator.explain_full(&trace, &contributions);
        self.entries.push(AuditEntry {
            trace,
            contributions: Some(contributions),
            explanation,
        });
    }

    /// Number of logged entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the log is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get all entries.
    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    /// Get entry by decision ID.
    pub fn get_by_id(&self, id: u64) -> Option<&AuditEntry> {
        self.entries.iter().find(|e| e.trace.id == id)
    }

    /// Filter entries by action.
    pub fn filter_by_action(&self, action: TernaryAction) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.trace.action == action).collect()
    }

    /// Filter entries that were close calls.
    pub fn close_calls(&self, threshold: f64) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.trace.is_close(threshold)).collect()
    }

    /// Get entries within a time range [start_ms, end_ms].
    pub fn filter_by_time(&self, start_ms: Timestamp, end_ms: Timestamp) -> Vec<&AuditEntry> {
        self.entries.iter()
            .filter(|e| e.trace.timestamp >= start_ms && e.trace.timestamp <= end_ms)
            .collect()
    }

    /// Distribution of actions across all entries.
    pub fn action_distribution(&self) -> [(TernaryAction, usize); 3] {
        let mut counts = [0usize; 3];
        for entry in &self.entries {
            match entry.trace.action {
                TernaryAction::Negative => counts[0] += 1,
                TernaryAction::Neutral => counts[1] += 1,
                TernaryAction::Positive => counts[2] += 1,
            }
        }
        [
            (TernaryAction::Negative, counts[0]),
            (TernaryAction::Neutral, counts[1]),
            (TernaryAction::Positive, counts[2]),
        ]
    }

    /// Average confidence across all entries that have confidence set.
    pub fn average_confidence(&self) -> Option<f64> {
        let confidences: Vec<f64> = self.entries.iter()
            .filter_map(|e| e.trace.confidence)
            .collect();
        if confidences.is_empty() {
            return None;
        }
        Some(confidences.iter().sum::<f64>() / confidences.len() as f64)
    }

    /// Format the entire audit log as a string.
    pub fn format_report(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("Audit Log ({} entries)", self.entries.len()));
        lines.push("─".repeat(50).to_string());

        let dist = self.action_distribution();
        lines.push("Action Distribution:".to_string());
        for (action, count) in &dist {
            let pct = if self.entries.is_empty() {
                0.0
            } else {
                (*count as f64 / self.entries.len() as f64) * 100.0
            };
            lines.push(format!("  {}: {} ({:.1}%)", action.label(), count, pct));
        }

        if let Some(avg_conf) = self.average_confidence() {
            lines.push(format!("Average confidence: {:.1}%", avg_conf * 100.0));
        }

        lines.push(String::new());
        lines.push("Entries:".to_string());
        for entry in &self.entries {
            lines.push(format!("  {}", entry.explanation.summary));
        }

        lines.join("\n")
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}
