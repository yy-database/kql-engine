use sqlparser::ast::{ColumnDef, DataType, Statement, TableConstraint};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use indexmap::IndexMap;

pub struct Transpiler;

impl Transpiler {
    pub fn transpile(sql: &str) -> Result<String, String> {
        let dialect = GenericDialect {};
        let ast = Parser::parse_sql(&dialect, sql).map_err(|e| e.to_string())?;

        let mut schemas: IndexMap<Option<String>, Vec<Statement>> = IndexMap::new();

        for statement in ast {
            if let Statement::CreateTable { name, .. } = &statement {
                let schema = if name.0.len() > 1 {
                    Some(name.0[0].value.clone())
                } else {
                    None
                };
                schemas.entry(schema).or_default().push(statement);
            }
        }

        let mut kql = String::new();
        for (schema_name, statements) in schemas {
            let mut current_kql = String::new();
            if let Some(s) = &schema_name {
                current_kql.push_str(&format!("@schema(\"{}\")\n", s));
                current_kql.push_str(&format!("database {} {{\n", to_pascal_case(s)));
            }

            for statement in statements {
                if let Statement::CreateTable { name, columns, constraints, .. } = statement {
                    let table_name = name.0.last().unwrap().value.clone();
                    let struct_name = to_pascal_case(&table_name);
                    
                    let table_attr = if let Some(s) = &schema_name {
                        format!("@table(schema: \"{}\", \"{}\")\n", s, table_name)
                    } else {
                        format!("@table(\"{}\")\n", table_name)
                    };
                    
                    let indent = if schema_name.is_some() { "    " } else { "" };
                    current_kql.push_str(&format!("{}{}", indent, table_attr));
                    current_kql.push_str(&format!("{}struct {} {{\n", indent, struct_name));
                    for col in columns {
                        let col_kql = transpile_column(&col, &table_name, &constraints);
                        current_kql.push_str(&format!("{}    {},\n", indent, col_kql));
                    }
                    current_kql.push_str(&format!("{}}}\n\n", indent));
                }
            }

            if schema_name.is_some() {
                current_kql = current_kql.trim_end().to_string();
                current_kql.push_str("\n}\n\n");
            }
            kql.push_str(&current_kql);
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

    #[test]
    fn test_transpile_schema() {
        let sql = "CREATE TABLE pg_catalog.pg_proc (id INT PRIMARY KEY, name TEXT)";
        let kql = Transpiler::transpile(sql).unwrap();
        assert_eq!(
            kql,
            "@schema(\"pg_catalog\")\ndatabase PgCatalog {\n    @table(schema: \"pg_catalog\", \"pg_proc\")\n    struct PgProc {\n        id: Key<i32>,\n        name: String?,\n    }\n}"
        );
    }
}
