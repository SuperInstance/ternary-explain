//! Core types for ternary decisions.

/// A ternary action: negative (-1), neutral (0), or positive (+1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TernaryAction {
    /// Negative / oppose / sell / reject
    Negative = -1,
    /// Neutral / hold / abstain / skip
    Neutral = 0,
    /// Positive / support / buy / accept
    Positive = 1,
}

impl TernaryAction {
    /// Convert from integer value (-1, 0, 1) to TernaryAction.
    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            -1 => Some(Self::Negative),
            0 => Some(Self::Neutral),
            1 => Some(Self::Positive),
            _ => None,
        }
    }

    /// Convert to integer value.
    pub fn as_i8(self) -> i8 {
        self as i8
    }

    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Negative => "Negative (-1)",
            Self::Neutral => "Neutral (0)",
            Self::Positive => "Positive (+1)",
        }
    }

    /// All variants.
    pub fn all() -> [TernaryAction; 3] {
        [Self::Negative, Self::Neutral, Self::Positive]
    }

    /// The opposite action (flip sign).
    pub fn opposite(self) -> Self {
        match self {
            Self::Negative => Self::Positive,
            Self::Neutral => Self::Neutral,
            Self::Positive => Self::Negative,
        }
    }
}

impl std::fmt::Display for TernaryAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// A named score for a single ternary action.
#[derive(Debug, Clone)]
pub struct ActionScore {
    /// Which action this score belongs to.
    pub action: TernaryAction,
    /// The numerical score (higher = more likely chosen).
    pub score: f64,
}

/// A named input feature with its raw value.
#[derive(Debug, Clone)]
pub struct FeatureValue {
    /// Feature name.
    pub name: String,
    /// Raw feature value.
    pub value: f64,
    /// Optional human-readable description.
    pub description: Option<String>,
}

impl FeatureValue {
    /// Create a new feature value.
    pub fn new(name: impl Into<String>, value: f64) -> Self {
        Self {
            name: name.into(),
            value,
            description: None,
        }
    }

    /// Add a description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// Timestamp in milliseconds since Unix epoch.
pub type Timestamp = u64;

/// Generate a simple timestamp (ms since epoch) using a counter-based approach.
/// In production, use actual time; this is deterministic for testing.
#[derive(Debug, Clone)]
pub struct Clock {
    /// Current time in ms.
    now_ms: u64,
}

impl Clock {
    /// Create a new clock starting at the given time.
    pub fn new(start_ms: u64) -> Self {
        Self { now_ms: start_ms }
    }

    /// Get current time and advance by `step_ms`.
    pub fn tick(&mut self, step_ms: u64) -> u64 {
        let t = self.now_ms;
        self.now_ms += step_ms;
        t
    }

    /// Current time without advancing.
    pub fn now(&self) -> u64 {
        self.now_ms
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new(1_700_000_000_000)
    }
}
