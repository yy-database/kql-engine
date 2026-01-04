pub mod sql_gen;

/// LIR is represented using sqlparser's AST for multi-dialect support
pub use sqlparser::ast::Statement as LirStatement;
