use sqlparser::ast::{ColumnDef, DataType, Statement, TableConstraint};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

pub struct Transpiler;

impl Transpiler {
    pub fn transpile(sql: &str) -> Result<String, String> {
        let dialect = GenericDialect {};
        let ast = Parser::parse_sql(&dialect, sql).map_err(|e| e.to_string())?;

        let mut kql = String::new();
        for statement in ast {
            if let Statement::CreateTable { name, columns, constraints, .. } = statement {
                let table_name = name.to_string();
                let struct_name = to_pascal_case(&table_name);
                
                kql.push_str(&format!("@table(\"{}\")\n", table_name));
                kql.push_str(&format!("struct {} {{\n", struct_name));
                for col in columns {
                    let col_kql = transpile_column(&col, &table_name, &constraints);
                    kql.push_str(&format!("    {},\n", col_kql));
                }
                kql.push_str("}\n\n");
            }
        }

        Ok(kql.trim().to_string())
    }
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c.to_ascii_lowercase());
        }
    }

    // Handle the case where the name ends in 's' (common for tables like 'users')
    // and we want the struct to be singular (User).
    // This is a simple heuristic.
    if result.ends_with('s') && result.len() > 1 {
        result.pop();
    }

    result
}

fn transpile_column(col: &ColumnDef, _table_name: &str, table_constraints: &[TableConstraint]) -> String {
    let name = col.name.to_string();
    let mut kql_type = match &col.data_type {
        DataType::Int(_) | DataType::Integer(_) => "i32".to_string(),
                DataType::SmallInt(_) => "i16".to_string(),
                DataType::BigInt(_) => "i64".to_string(),
                DataType::Float(_) | DataType::Real => "f32".to_string(),
                DataType::Double | DataType::DoublePrecision => "f64".to_string(),
        DataType::Boolean => "bool".to_string(),
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
        format!("{}: Key<{}>", name, kql_type)
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
        assert_eq!(
            kql,
            "@table(\"users\")\nstruct User {\n    id: Key<i32>,\n    name: String,\n    age: i32?,\n}"
        );
    }
}
