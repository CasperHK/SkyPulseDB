pub mod gorilla;
pub mod delta;

// Convenience re-exports and small helpers for callers.
pub use gorilla::{decode as decode_floats, encode as encode_floats};
pub use delta::{decode_timestamps, encode_timestamps};

// Higher-level helpers could be added here later (e.g., chunk-level
// encode/decode that combine timestamps + columns).
