pub mod sql_gen;

use sqlparser::dialect::{Dialect, MySqlDialect, PostgreSqlDialect, SQLiteDialect};
/// LIR is represented using sqlparser's AST for multi-dialect support
pub use sqlparser::ast::Statement as LirStatement;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlDialect {
    Postgres,
    MySql,
    Sqlite,
}

impl SqlDialect {
    pub fn get_dialect(&self) -> Box<dyn Dialect> {
        match self {
            SqlDialect::Postgres => Box::new(PostgreSqlDialect {}),
            SqlDialect::MySql => Box::new(MySqlDialect {}),
            SqlDialect::Sqlite => Box::new(SQLiteDialect {}),
        }
    }
}
