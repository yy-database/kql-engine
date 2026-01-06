//! KQL Language Server Protocol implementation
//!
//! This crate provides an LSP server for KQL (Query with ADTs Language).
//!
//! # Features
//!
//! - Text document synchronization
//! - Syntax diagnostics
//! - Code completion
//! - Hover information
//! - Document formatting (planned)
//!
//! # Usage
//!
//! The LSP server can be used with any LSP-compatible editor.
//!
//! ## VSCode
//!
//! See the `kql-vscode` extension in the workspace.
//!
//! ## Other Editors
//!
//! Configure your editor to use the `kql-lsp` binary.

pub mod server;
pub mod diagnostics;
pub mod completion;

pub use server::KqlLanguageServer;
