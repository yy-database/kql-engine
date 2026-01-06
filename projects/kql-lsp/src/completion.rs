//! Completion provider for KQL LSP

use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, InsertTextFormat};

/// Returns completion items for KQL keywords and common patterns
pub fn get_keyword_completions() -> Vec<CompletionItem> {
    vec![
        // Declarations
        CompletionItem {
            label: "struct".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Define a struct".to_string()),
            insert_text: Some("struct ${1:Name} {\n    ${2:field}: ${3:Type}\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "enum".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Define an enum".to_string()),
            insert_text: Some("enum ${1:Name} {\n    ${2:Variant}\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "let".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Define a let binding".to_string()),
            insert_text: Some("let ${1:name} = ${2:value}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "namespace".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Define a namespace".to_string()),
            insert_text: Some("namespace ${1:name}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "type".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Define a type alias".to_string()),
            insert_text: Some("type ${1:Alias} = ${2:Type}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
    ]
}

/// Returns completion items for KQL types
pub fn get_type_completions() -> Vec<CompletionItem> {
    vec![
        // Key types
        CompletionItem {
            label: "Key".to_string(),
            kind: Some(CompletionItemKind::TYPE),
            detail: Some("Primary key type".to_string()),
            insert_text: Some("Key<${1:i32}>".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "ForeignKey".to_string(),
            kind: Some(CompletionItemKind::TYPE),
            detail: Some("Foreign key type".to_string()),
            insert_text: Some("ForeignKey<${1:Entity}>".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        // Primitives
        CompletionItem {
            label: "String".to_string(),
            kind: Some(CompletionItemKind::TYPE),
            detail: Some("String primitive".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "i32".to_string(),
            kind: Some(CompletionItemKind::TYPE),
            detail: Some("32-bit integer".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "i64".to_string(),
            kind: Some(CompletionItemKind::TYPE),
            detail: Some("64-bit integer".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "f32".to_string(),
            kind: Some(CompletionItemKind::TYPE),
            detail: Some("32-bit float".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "f64".to_string(),
            kind: Some(CompletionItemKind::TYPE),
            detail: Some("64-bit float".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "bool".to_string(),
            kind: Some(CompletionItemKind::TYPE),
            detail: Some("Boolean".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "DateTime".to_string(),
            kind: Some(CompletionItemKind::TYPE),
            detail: Some("Date and time".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "UUID".to_string(),
            kind: Some(CompletionItemKind::TYPE),
            detail: Some("UUID".to_string()),
            ..Default::default()
        },
        // Special types
        CompletionItem {
            label: "List".to_string(),
            kind: Some(CompletionItemKind::TYPE),
            detail: Some("List type".to_string()),
            insert_text: Some("List<${1:Type}>".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
    ]
}

/// Returns completion items for KQL annotations
pub fn get_annotation_completions() -> Vec<CompletionItem> {
    vec![
        // Field annotations
        CompletionItem {
            label: "@auto_increment".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Auto increment field".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "@unique".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Unique constraint".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "@primary_key".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Primary key".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "@nullable".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Nullable field".to_string()),
            ..Default::default()
        },
        // Table annotations
        CompletionItem {
            label: "@table".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Custom table name".to_string()),
            insert_text: Some("@table(\"${1:name}\")".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "@schema".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Schema qualifier".to_string()),
            insert_text: Some("@schema(\"${1:name}\")".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        // Layout annotations
        CompletionItem {
            label: "@layout(json)".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("JSON storage layout".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "@layout(u8)".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Enum as u8".to_string()),
            ..Default::default()
        },
        // Relation annotations
        CompletionItem {
            label: "@relation".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Relation definition".to_string()),
            insert_text: Some("@relation(name: \"${1:name}\", foreign_key: \"${2:fk}\")".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        // Audit and lifecycle
        CompletionItem {
            label: "@audit".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Audit fields".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "@soft_delete".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Soft delete".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "@before_save".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Before save hook".to_string()),
            insert_text: Some("@before_save(${1:hook_fn})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "@after_delete".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("After delete hook".to_string()),
            insert_text: Some("@after_delete(${1:hook_fn})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
    ]
}

/// Returns completion items for KQL query methods
pub fn get_query_method_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: ".filter".to_string(),
            kind: Some(CompletionItemKind::METHOD),
            detail: Some("Filter query".to_string()),
            insert_text: Some(".filter { ${1:$.field == value} }".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: ".map".to_string(),
            kind: Some(CompletionItemKind::METHOD),
            detail: Some("Map projection".to_string()),
            insert_text: Some(".map { ${1:$.{field1, field2}} }".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: ".sort".to_string(),
            kind: Some(CompletionItemKind::METHOD),
            detail: Some("Sort order".to_string()),
            insert_text: Some(".sort { ${1:$.field.desc()} }".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: ".take".to_string(),
            kind: Some(CompletionItemKind::METHOD),
            detail: Some("Limit results".to_string()),
            insert_text: Some(".take(${1:10})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: ".skip".to_string(),
            kind: Some(CompletionItemKind::METHOD),
            detail: Some("Skip results".to_string()),
            insert_text: Some(".skip(${1:10})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        // Aggregations
        CompletionItem {
            label: ".count".to_string(),
            kind: Some(CompletionItemKind::METHOD),
            detail: Some("Count aggregation".to_string()),
            insert_text: Some(".count(${1:*})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: ".avg".to_string(),
            kind: Some(CompletionItemKind::METHOD),
            detail: Some("Average aggregation".to_string()),
            insert_text: Some(".avg(${1:field})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: ".sum".to_string(),
            kind: Some(CompletionItemKind::METHOD),
            detail: Some("Sum aggregation".to_string()),
            insert_text: Some(".sum(${1:field})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: ".max".to_string(),
            kind: Some(CompletionItemKind::METHOD),
            detail: Some("Max aggregation".to_string()),
            insert_text: Some(".max(${1:field})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: ".min".to_string(),
            kind: Some(CompletionItemKind::METHOD),
            detail: Some("Min aggregation".to_string()),
            insert_text: Some(".min(${1:field})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        // Window functions
        CompletionItem {
            label: ".over".to_string(),
            kind: Some(CompletionItemKind::METHOD),
            detail: Some("Window function".to_string()),
            insert_text: Some(".over(partition_by: ${1:$.field}, order_by: ${2:$.field.desc()})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        // Joins
        CompletionItem {
            label: ".join".to_string(),
            kind: Some(CompletionItemKind::METHOD),
            detail: Some("Join tables".to_string()),
            insert_text: Some(".join(${1:Other}, on: ${2:$.id == $.other_id})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
    ]
}

/// Returns all completion items
pub fn get_all_completions() -> Vec<CompletionItem> {
    let mut completions = Vec::new();
    completions.extend(get_keyword_completions());
    completions.extend(get_type_completions());
    completions.extend(get_annotation_completions());
    completions.extend(get_query_method_completions());
    completions
}
