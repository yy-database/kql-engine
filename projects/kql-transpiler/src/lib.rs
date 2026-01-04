use sqlparser::ast::{ColumnDef, DataType, Statement, TableConstraint};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use anyhow::{Result, anyhow};

pub struct Transpiler;

impl Transpiler {
    pub fn transpile(sql: &str) -> Result<String> {
        let dialect = GenericDialect {};
        let ast = Parser::parse_sql(&dialect, sql)?;

        let mut kql_output = String::new();

        for stmt in ast {
            match stmt {
                Statement::CreateTable { name, columns, constraints, .. } => {
                    let table_name = name.to_string();
                    kql_output.push_str(&format!("struct {} {{\n", table_name));

                    for col in columns {
                        let col_str = self::transpile_column(&col, &table_name, &constraints);
                        kql_output.push_str(&format!("    {},\n", col_str));
                    }

                    kql_output.push_str("}\n\n");
                }
                _ => return Err(anyhow!("Unsupported SQL statement: {:?}", stmt)),
            }
        }

        Ok(kql_output.trim().to_string())
    }
}

fn transpile_column(col: &ColumnDef, table_name: &str, table_constraints: &[TableConstraint]) -> String {
    let name = col.name.to_string();
    let mut kql_type = match &col.data_type {
        DataType::Int(_) | DataType::Integer(_) => "i32".to_string(),
        DataType::SmallInt(_) => "i16".to_string(),
        DataType::BigInt(_) => "i64".to_string(),
        DataType::Float(_) | DataType::Real => "f32".to_string(),
        DataType::Double | DataType::DoublePrecision => "f64".to_string(),
        DataType::Boolean => "Boolean".to_string(),
        DataType::Varchar(_) | DataType::Text | DataType::String(_) => "String".to_string(),
        DataType::Timestamp(_, _) | DataType::Datetime(_) => "DateTime".to_string(),
        DataType::Uuid => "UUID".to_string(),
        DataType::Decimal(_) | DataType::Numeric(_) => "d128".to_string(),
        _ => "String".to_string(), // Default to String for unknown types
    };

    let mut is_nullable = true;
    let mut is_primary = false;

    for option in &col.options {
        match &option.option {
            sqlparser::ast::ColumnOption::NotNull => is_nullable = false,
            sqlparser::ast::ColumnOption::Unique { is_primary: p, .. } => {
                if *p {
                    is_primary = true;
                    is_nullable = false;
                }
            }
            _ => {}
        }
    }

    // Check table constraints for primary key
    for constraint in table_constraints {
        match constraint {
            TableConstraint::Unique {
                columns,
                is_primary: p,
                ..
            } => {
                if *p && columns.iter().any(|c| c.to_string() == name) {
                    is_primary = true;
                    is_nullable = false;
                }
            }
            _ => {}
        }
    }

    if is_primary {
        format!("{}: Key<{}, {}>", name, table_name, kql_type)
    } else {
        if is_nullable {
            kql_type.push('?');
        }
        format!("{}: {}", name, kql_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transpile_basic() {
        let sql = "CREATE TABLE users (id INT PRIMARY KEY, name VARCHAR(255) NOT NULL, age INT)";
        let kql = Transpiler::transpile(sql).unwrap();
        assert_eq!(kql, "struct users {\n    id: int32 @primary,\n    name: string,\n    age: int32?,\n}");
    }
}
