//! # ternary-explain
//!
//! Explainability for ternary agent decisions — WHY did the agent choose -1, 0, or +1?
//!
//! This crate provides tools to trace, explain, and audit ternary decisions
//! where an agent chooses between three actions: negative (-1), neutral (0),
//! or positive (+1).

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod lib {
    pub mod types;
    pub mod trace;
    pub mod feature;
    pub mod explanation;
    pub mod counterfactual;
    pub mod audit;
    pub mod report;
}

pub use lib::types::*;
pub use lib::trace::*;
pub use lib::feature::*;
pub use lib::explanation::*;
pub use lib::counterfactual::*;
pub use lib::audit::*;
pub use lib::report::*;
