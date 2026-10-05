//! CHRONO Real-Time Benchmarking & Execution Routing Lab
//!
//! Subsystem implementing rigorous empirical measurement of transaction
//! preparation, routing, submission, landing, confirmation, and finality.

pub mod experiment;
pub mod freshness;
pub mod decision;
pub mod route;
pub mod stats;
pub mod storage;
pub mod runner;
pub mod explainer;
pub mod leader_transport;
