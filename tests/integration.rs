//! Comprehensive tests for ternary-explain.

use ternary_explain::*;

// ── Types ──

#[test]
fn test_ternary_action_from_i8() {
    assert_eq!(TernaryAction::from_i8(-1), Some(TernaryAction::Negative));
    assert_eq!(TernaryAction::from_i8(0), Some(TernaryAction::Neutral));
    assert_eq!(TernaryAction::from_i8(1), Some(TernaryAction::Positive));
    assert_eq!(TernaryAction::from_i8(2), None);
    assert_eq!(TernaryAction::from_i8(-2), None);
}

#[test]
fn test_ternary_action_as_i8() {
    assert_eq!(TernaryAction::Negative.as_i8(), -1);
    assert_eq!(TernaryAction::Neutral.as_i8(), 0);
    assert_eq!(TernaryAction::Positive.as_i8(), 1);
}

#[test]
fn test_ternary_action_opposite() {
    assert_eq!(TernaryAction::Negative.opposite(), TernaryAction::Positive);
    assert_eq!(TernaryAction::Positive.opposite(), TernaryAction::Negative);
    assert_eq!(TernaryAction::Neutral.opposite(), TernaryAction::Neutral);
}

#[test]
fn test_feature_value_builder() {
    let fv = FeatureValue::new("price", 42.5)
        .with_description("Current market price");
    assert_eq!(fv.name, "price");
    assert_eq!(fv.value, 42.5);
    assert_eq!(fv.description.as_deref(), Some("Current market price"));
}

#[test]
fn test_clock() {
    let mut clock = Clock::new(1000);
    assert_eq!(clock.now(), 1000);
    let t1 = clock.tick(100);
    assert_eq!(t1, 1000);
    assert_eq!(clock.now(), 1100);
    let t2 = clock.tick(50);
    assert_eq!(t2, 1100);
}

// ── DecisionTrace ──

#[test]
fn test_trace_builder() {
    let trace = DecisionTrace::new(1, 1000, TernaryAction::Positive)
        .with_context("market_open")
        .with_input(FeatureValue::new("sentiment", 0.8))
        .with_input(FeatureValue::new("volume", 1500.0))
        .with_scores(0.1, 0.3, 0.6)
        .with_confidence(0.85)
        .with_metadata("model", "v2");

    assert_eq!(trace.id, 1);
    assert_eq!(trace.action, TernaryAction::Positive);
    assert_eq!(trace.context.as_deref(), Some("market_open"));
    assert_eq!(trace.inputs.len(), 2);
    assert_eq!(trace.scores.len(), 3);
    assert_eq!(trace.confidence, Some(0.85));
}

#[test]
fn test_trace_winning_score() {
    let trace = DecisionTrace::new(1, 0, TernaryAction::Positive)
        .with_scores(0.1, 0.3, 0.6);
    assert_eq!(trace.winning_score(), Some(0.6));
}

#[test]
fn test_trace_margin() {
    let trace = DecisionTrace::new(1, 0, TernaryAction::Positive)
        .with_scores(0.1, 0.3, 0.6);
    assert_eq!(trace.margin(), Some(0.3)); // 0.6 - 0.3
}

#[test]
fn test_trace_is_close() {
    let close = DecisionTrace::new(1, 0, TernaryAction::Positive)
        .with_scores(0.49, 0.5, 0.51);
    assert!(close.is_close(0.1));

    let not_close = DecisionTrace::new(2, 0, TernaryAction::Positive)
        .with_scores(0.1, 0.2, 0.9);
    assert!(!not_close.is_close(0.1));
}

// ── FeatureContribution ──

#[test]
fn test_feature_contribution_direction() {
    let c = FeatureContribution::new("sentiment", 0.1, 0.2, 0.7);
    assert_eq!(c.dominant_direction(), TernaryAction::Positive);
    assert_eq!(c.net_push(), 0.6); // 0.7 - 0.1
}

#[test]
fn test_feature_contribution_magnitude() {
    let c = FeatureContribution::new("price", -0.3, 0.1, 0.2);
    assert!((c.magnitude() - 0.6).abs() < 1e-9); // 0.3 + 0.1 + 0.2
}

#[test]
fn test_contribution_set_sorted() {
    let mut cs = ContributionSet::new(TernaryAction::Positive);
    cs.add(FeatureContribution::new("weak", 0.1, 0.1, 0.1));
    cs.add(FeatureContribution::new("strong", -0.5, 0.0, 0.8));
    cs.add(FeatureContribution::new("medium", 0.2, 0.3, 0.4));

    let sorted = cs.sorted_by_magnitude();
    assert_eq!(sorted[0].feature, "strong");
    assert_eq!(sorted[1].feature, "medium");
    assert_eq!(sorted[2].feature, "weak");
}

#[test]
fn test_supporting_and_opposing_features() {
    let mut cs = ContributionSet::new(TernaryAction::Positive);
    cs.add(FeatureContribution::new("bullish", 0.0, 0.1, 0.8)); // supports Positive
    cs.add(FeatureContribution::new("bearish", 0.9, 0.05, 0.0)); // opposes Positive
    cs.add(FeatureContribution::new("neutral_feat", 0.0, 0.9, 0.0)); // pushes Neutral

    assert_eq!(cs.supporting_features().len(), 1);
    assert_eq!(cs.opposing_features().len(), 2);
}

// ── ExplanationGenerator ──

#[test]
fn test_explain_trace_basic() {
    let gen = ExplanationGenerator::default_generator();
    let trace = DecisionTrace::new(42, 0, TernaryAction::Neutral)
        .with_scores(0.2, 0.5, 0.3)
        .with_confidence(0.7);

    let expl = gen.explain_trace(&trace);
    assert_eq!(expl.decision_id, 42);
    assert!(expl.summary.contains("Neutral"));
    assert!(!expl.is_close_call);
}

#[test]
fn test_explain_close_call() {
    let gen = ExplanationGenerator::default_generator();
    let trace = DecisionTrace::new(1, 0, TernaryAction::Positive)
        .with_scores(0.49, 0.5, 0.51);

    let expl = gen.explain_trace(&trace);
    assert!(expl.is_close_call);
}

#[test]
fn test_explain_full() {
    let gen = ExplanationGenerator::default_generator();
    let trace = DecisionTrace::new(1, 0, TernaryAction::Positive)
        .with_scores(0.1, 0.3, 0.6)
        .with_input(FeatureValue::new("sentiment", 0.8));

    let mut cs = ContributionSet::new(TernaryAction::Positive);
    cs.add(FeatureContribution::new("sentiment", 0.0, 0.1, 0.7));

    let expl = gen.explain_full(&trace, &cs);
    assert!(expl.details.iter().any(|d| d.contains("sentiment")));
}

// ── Counterfactual ──

#[test]
fn test_counterfactual_same_action() {
    let trace = DecisionTrace::new(1, 0, TernaryAction::Positive)
        .with_scores(0.1, 0.3, 0.6);
    let cs = ContributionSet::new(TernaryAction::Positive);

    let cf = CounterfactualAnalyzer::analyze(&trace, &cs, TernaryAction::Positive);
    assert!(cf.summary.contains("already chose"));
}

#[test]
fn test_counterfactual_with_contributions() {
    let trace = DecisionTrace::new(1, 0, TernaryAction::Positive)
        .with_scores(0.1, 0.3, 0.6)
        .with_input(FeatureValue::new("rsi", 70.0));

    let mut cs = ContributionSet::new(TernaryAction::Positive);
    cs.add(FeatureContribution::new("rsi", 0.5, 0.3, 0.2)); // rsi pushes toward Negative

    let cf = CounterfactualAnalyzer::analyze(&trace, &cs, TernaryAction::Negative);
    // If no changes found (depends on contribution calculation), verify the summary
    if !cf.changes.is_empty() {
        assert_eq!(cf.changes[0].feature, "rsi");
    } else {
        assert!(cf.summary.contains("No single feature"));
    }
}

#[test]
fn test_counterfactual_analyze_all() {
    let trace = DecisionTrace::new(1, 0, TernaryAction::Positive)
        .with_scores(0.1, 0.3, 0.6);
    let cs = ContributionSet::new(TernaryAction::Positive);

    let cfs = CounterfactualAnalyzer::analyze_all(&trace, &cs);
    assert_eq!(cfs.len(), 2); // Negative and Neutral
}

// ── AuditLog ──

#[test]
fn test_audit_log_basic() {
    let mut log = AuditLog::new();
    log.log_trace(DecisionTrace::new(1, 1000, TernaryAction::Positive)
        .with_scores(0.1, 0.3, 0.6));
    log.log_trace(DecisionTrace::new(2, 2000, TernaryAction::Negative)
        .with_scores(0.7, 0.2, 0.1));

    assert_eq!(log.len(), 2);
    assert_eq!(log.get_by_id(1).unwrap().trace.action, TernaryAction::Positive);
    assert_eq!(log.get_by_id(2).unwrap().trace.action, TernaryAction::Negative);
}

#[test]
fn test_audit_log_filter_by_action() {
    let mut log = AuditLog::new();
    log.log_trace(DecisionTrace::new(1, 0, TernaryAction::Positive));
    log.log_trace(DecisionTrace::new(2, 0, TernaryAction::Positive));
    log.log_trace(DecisionTrace::new(3, 0, TernaryAction::Negative));

    let pos = log.filter_by_action(TernaryAction::Positive);
    assert_eq!(pos.len(), 2);
}

#[test]
fn test_audit_log_distribution() {
    let mut log = AuditLog::new();
    log.log_trace(DecisionTrace::new(1, 0, TernaryAction::Positive));
    log.log_trace(DecisionTrace::new(2, 0, TernaryAction::Positive));
    log.log_trace(DecisionTrace::new(3, 0, TernaryAction::Negative));
    log.log_trace(DecisionTrace::new(4, 0, TernaryAction::Neutral));

    let dist = log.action_distribution();
    assert_eq!(dist[0], (TernaryAction::Negative, 1));
    assert_eq!(dist[1], (TernaryAction::Neutral, 1));
    assert_eq!(dist[2], (TernaryAction::Positive, 2));
}

#[test]
fn test_audit_log_format() {
    let mut log = AuditLog::new();
    log.log_trace(DecisionTrace::new(1, 0, TernaryAction::Positive)
        .with_scores(0.1, 0.3, 0.6));

    let report = log.format_report();
    assert!(report.contains("Audit Log"));
    assert!(report.contains("Positive"));
}

// ── ExplainReport ──

#[test]
fn test_report_generation() {
    let mut log = AuditLog::new();
    for i in 0..15 {
        let action = match i % 3 {
            0 => TernaryAction::Negative,
            1 => TernaryAction::Neutral,
            _ => TernaryAction::Positive,
        };
        log.log_trace(
            DecisionTrace::new(i, i * 1000, action)
                .with_scores(0.2, 0.3, 0.5)
                .with_confidence(0.5 + (i as f64 * 0.03).min(0.45))
        );
    }

    let gen = ReportGenerator::default_generator();
    let report = gen.generate("Test Report", &log);

    assert_eq!(report.summary.total_decisions, 15);
    assert!(!report.text.is_empty());
    assert!(report.text.contains("Test Report"));
}

#[test]
fn test_report_edge_cases() {
    let mut log = AuditLog::new();
    // Low confidence decision
    log.log_trace(
        DecisionTrace::new(1, 0, TernaryAction::Neutral)
            .with_scores(0.33, 0.34, 0.33)
            .with_confidence(0.35)
    );

    let gen = ReportGenerator::default_generator();
    let report = gen.generate("Edge Report", &log);

    assert_eq!(report.edge_cases.len(), 1);
}
