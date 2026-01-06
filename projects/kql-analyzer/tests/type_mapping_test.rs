mod common;
use common::assert_sql_has;
use kql_parser::Parser;
use kql_analyzer::hir::lower::Lowerer;
use kql_analyzer::mir::mir_gen::MirLowerer;
use kql_analyzer::lir::sql_gen::SqlGenerator;
use kql_analyzer::lir::SqlDialect;

#[test]
fn test_type_mapping_postgres() {
    let input = r#"
        struct AllTypes {
            @primary_key
            id: i32,
            c_i8: i8,
            c_u8: u8,
            c_u64: u64,
            c_date: date,
            c_time: time,
            c_datetime: datetime,
            c_uuid: uuid,
            c_decimal: decimal,
            c_bytes: bytes,
            c_json: json
        }
    "#;

    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    let mut lowerer = Lowerer::new();
    let hir = lowerer.lower_program(&ast).unwrap();
    let mut mir_gen = MirLowerer::new(hir);
    let mir = mir_gen.lower().unwrap();

    let sql_gen = SqlGenerator::new(mir, SqlDialect::Postgres);
    let sqls = sql_gen.generate_ddl_sql();
    let sql = sqls.join("\n");

    assert_sql_has(&sql, &[
        "c_i8 SMALLINT",
        "c_u8 SMALLINT",
        "c_u64 NUMERIC(20)",
        "c_date DATE",
        "c_time TIME",
        "c_datetime TIMESTAMP",
        "c_uuid UUID",
                "c_decimal NUMERIC(38,10)",
                "c_bytes BYTEA",
                "c_json JSONB"
            ]);
}

#[test]
fn test_type_mapping_mysql() {
    let input = r#"
        struct AllTypes {
            @primary_key
            id: i32,
            c_i8: i8,
            c_u8: u8,
            c_u64: u64,
            c_date: date,
            c_time: time,
            c_datetime: datetime,
            c_uuid: uuid,
            c_decimal: decimal,
            c_bytes: bytes,
            c_json: json
        }
    "#;

    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    let mut lowerer = Lowerer::new();
    let hir = lowerer.lower_program(&ast).unwrap();
    let mut mir_gen = MirLowerer::new(hir);
    let mir = mir_gen.lower().unwrap();

    let sql_gen = SqlGenerator::new(mir, SqlDialect::MySql);
    let sqls = sql_gen.generate_ddl_sql();
    let sql = sqls.join("\n");

    assert_sql_has(&sql, &[
        "c_i8 TINYINT",
        "c_u8 TINYINT UNSIGNED",
        "c_u64 BIGINT UNSIGNED",
        "c_date DATE",
        "c_time TIME",
        "c_datetime DATETIME",
        "c_uuid CHAR(36)",
                "c_decimal DECIMAL(38,10)",
                "c_bytes BLOB",
                "c_json JSON"
            ]);
}

#[test]
fn test_type_mapping_sqlite() {
    let input = r#"
        struct AllTypes {
            @primary_key
            id: i32,
            c_i8: i8,
            c_u8: u8,
            c_u64: u64,
            c_date: date,
            c_time: time,
            c_datetime: datetime,
            c_uuid: uuid,
            c_decimal: decimal,
            c_bytes: bytes,
            c_json: json
        }
    "#;

    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    let mut lowerer = Lowerer::new();
    let hir = lowerer.lower_program(&ast).unwrap();
    let mut mir_gen = MirLowerer::new(hir);
    let mir = mir_gen.lower().unwrap();

    let sql_gen = SqlGenerator::new(mir, SqlDialect::Sqlite);
    let sqls = sql_gen.generate_ddl_sql();
    let sql = sqls.join("\n");

    assert_sql_has(&sql, &[
        "c_i8 INTEGER",
        "c_u8 INTEGER",
        "c_u64 INTEGER",
        "c_date TEXT",
        "c_time TEXT",
        "c_datetime TEXT",
        "c_uuid TEXT",
        "c_decimal TEXT",
        "c_bytes BLOB",
        "c_json TEXT"
    ]);
}
