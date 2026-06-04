//! ExplainReport: formatted report showing top decisions, common patterns, edge cases.

use crate::lib::types::*;
use crate::lib::audit::*;

/// Configuration for report generation.
#[derive(Debug, Clone)]
pub struct ReportConfig {
    /// Number of top features to highlight.
    pub top_n: usize,
    /// Threshold for flagging close calls.
    pub close_threshold: f64,
    /// Include detailed per-entry breakdowns.
    pub include_details: bool,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            top_n: 10,
            close_threshold: 0.1,
            include_details: true,
        }
    }
}

/// A formatted explainability report.
#[derive(Debug, Clone)]
pub struct ExplainReport {
    /// Report title.
    pub title: String,
    /// Summary statistics.
    pub summary: ReportSummary,
    /// Top decisions by confidence.
    pub top_decisions: Vec<ReportDecision>,
    /// Close call decisions.
    pub close_calls: Vec<ReportDecision>,
    /// Edge cases (low confidence, close margin, opposing features).
    pub edge_cases: Vec<ReportDecision>,
    /// Formatted full text.
    pub text: String,
}

/// Summary statistics for the report.
#[derive(Debug, Clone)]
pub struct ReportSummary {
    /// Total decisions analyzed.
    pub total_decisions: usize,
    /// Action distribution counts.
    pub action_counts: [(TernaryAction, usize); 3],
    /// Average confidence.
    pub avg_confidence: Option<f64>,
    /// Number of close calls.
    pub close_call_count: usize,
}

/// A decision entry in the report.
#[derive(Debug, Clone)]
pub struct ReportDecision {
    /// Decision ID.
    pub id: u64,
    /// The action chosen.
    pub action: TernaryAction,
    /// Confidence level.
    pub confidence: Option<f64>,
    /// Decision margin.
    pub margin: Option<f64>,
    /// Explanation summary.
    pub explanation_summary: String,
}

/// Report generator.
pub struct ReportGenerator {
    config: ReportConfig,
}

impl ReportGenerator {
    /// Create a new report generator.
    pub fn new(config: ReportConfig) -> Self {
        Self { config }
    }

    /// Create with default config.
    pub fn default_generator() -> Self {
        Self::new(ReportConfig::default())
    }

    /// Generate a report from an audit log.
    pub fn generate(&self, title: &str, log: &AuditLog) -> ExplainReport {
        let entries = log.entries();

        // Summary
        let dist = log.action_distribution();
        let close_calls = log.close_calls(self.config.close_threshold);
        let avg_conf = log.average_confidence();

        let summary = ReportSummary {
            total_decisions: entries.len(),
            action_counts: dist,
            avg_confidence: avg_conf,
            close_call_count: close_calls.len(),
        };

        // Convert entries to report decisions
        let all_decisions: Vec<ReportDecision> = entries.iter().map(|e| ReportDecision {
            id: e.trace.id,
            action: e.trace.action,
            confidence: e.trace.confidence,
            margin: e.trace.margin(),
            explanation_summary: e.explanation.summary.clone(),
        }).collect();

        // Top decisions (sorted by confidence, highest first)
        let mut top = all_decisions.clone();
        top.sort_by(|a, b| {
            let ca = a.confidence.unwrap_or(0.0);
            let cb = b.confidence.unwrap_or(0.0);
            cb.partial_cmp(&ca).unwrap_or(std::cmp::Ordering::Equal)
        });
        top.truncate(self.config.top_n);

        // Close calls
        let close_decisions: Vec<ReportDecision> = close_calls.iter().map(|e| ReportDecision {
            id: e.trace.id,
            action: e.trace.action,
            confidence: e.trace.confidence,
            margin: e.trace.margin(),
            explanation_summary: e.explanation.summary.clone(),
        }).collect();

        // Edge cases: low confidence OR close margin
        let edge_cases: Vec<ReportDecision> = all_decisions.iter()
            .filter(|d| {
                d.confidence.map_or(true, |c| c < 0.6)
                    || d.margin.map_or(true, |m| m < self.config.close_threshold)
            })
            .cloned()
            .collect();

        // Format text
        let text = self.format_text(title, &summary, &top, &close_decisions, &edge_cases);

        ExplainReport {
            title: title.to_string(),
            summary,
            top_decisions: top,
            close_calls: close_decisions,
            edge_cases,
            text,
        }
    }

    fn format_text(
        &self,
        title: &str,
        summary: &ReportSummary,
        top: &[ReportDecision],
        close: &[ReportDecision],
        edges: &[ReportDecision],
    ) -> String {
        let mut lines = Vec::new();

        lines.push(format!("╔══ {} ══╗", title));
        lines.push(String::new());

        // Summary
        lines.push("📊 Summary".to_string());
        lines.push(format!("  Total decisions: {}", summary.total_decisions));
        for (action, count) in &summary.action_counts {
            lines.push(format!("  {}: {}", action.label(), count));
        }
        if let Some(conf) = summary.avg_confidence {
            lines.push(format!("  Average confidence: {:.1}%", conf * 100.0));
        }
        lines.push(format!("  Close calls: {}", summary.close_call_count));
        lines.push(String::new());

        // Top decisions
        if !top.is_empty() {
            lines.push("🏆 Top Decisions (by confidence)".to_string());
            for d in top {
                let conf_str = d.confidence.map(|c| format!("{:.0}%", c * 100.0)).unwrap_or_else(|| "N/A".to_string());
                lines.push(format!("  #{}: {} (confidence: {})", d.id, d.action.label(), conf_str));
            }
            lines.push(String::new());
        }

        // Close calls
        if !close.is_empty() {
            lines.push("⚖️ Close Calls".to_string());
            for d in close {
                let margin_str = d.margin.map(|m| format!("{:.4}", m)).unwrap_or_else(|| "N/A".to_string());
                lines.push(format!("  #{}: {} (margin: {})", d.id, d.action.label(), margin_str));
            }
            lines.push(String::new());
        }

        // Edge cases
        if !edges.is_empty() {
            lines.push("⚡ Edge Cases".to_string());
            for d in edges {
                lines.push(format!("  #{}: {}", d.id, d.explanation_summary));
            }
            lines.push(String::new());
        }

        // Common patterns
        lines.push("📈 Common Patterns".to_string());
        let total = summary.total_decisions as f64;
        if total > 0.0 {
            for (action, count) in &summary.action_counts {
                let pct = (*count as f64 / total) * 100.0;
                let bar_len = (pct / 5.0) as usize;
                let bar: String = "█".repeat(bar_len);
                lines.push(format!("  {} {:>3} ({:>5.1}%) {}", action.label(), count, pct, bar));
            }
        }

        lines.join("\n")
    }
}
