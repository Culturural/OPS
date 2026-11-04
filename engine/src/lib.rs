//! ops-engine - Recording engine for OPS (OBS Lightweight Recorder)
//!
//! Provides screen capture, encoding, muxing, hardware detection, and metrics.
//! Designed to be used by both CLI and Tauri desktop frontends.

pub mod capture;
pub mod encode;
pub mod hardware;
pub mod metrics;
pub mod mux;
