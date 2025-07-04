// src/shell/mod.rs
// Updated shell module with abstraction layers

#![no_std]

pub mod interfaces;
pub mod adapters;
pub mod shell;

pub use interfaces::*;
pub use adapters::*;
pub use shell::*;

// Re-export the main shell type for easy access
pub type KernelShell = adapters::Kernel1Shell;