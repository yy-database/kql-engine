# KQL for VS Code

Visual Studio Code extension for KQL (Query with ADTs Language).

## Features

- **Syntax Highlighting**: Full syntax highlighting for KQL files
- **Language Server Protocol**: Intelligent code completion, diagnostics, hover information
- **Formatting**: Code formatting support (coming soon)
- **SQL Generation**: Generate SQL from KQL schemas (coming soon)

## Installation

### From Marketplace

Coming soon...

### From Source

1. Build the kql-lsp binary:
   ```bash
   cd projects/kql-lsp
   cargo build --release
   ```

2. Install the extension:
   - Open VS Code
   - Go to Extensions
   - Click "Install from VSIX..."
   - Select the generated `.vsix` file

## Configuration

Configure the extension in VS Code settings:

```json
{
  "kql.lsp.enabled": true,
  "kql.lsp.path": "",  // Path to kql-lsp binary (empty for bundled)
  "kql.format.enabled": true,
  "kql.diagnostics.enabled": true
}
```

## Language Features

### Syntax Highlighting

KQL files (`.kql`) are recognized with full syntax highlighting for:
- Keywords: `struct`, `enum`, `let`, `namespace`, `type`
- Types: `String`, `i32`, `i64`, `f32`, `f64`, `bool`, `DateTime`, `UUID`, `Key`, `ForeignKey`, `List`
- Annotations: `@auto_increment`, `@unique`, `@relation`, etc.
- Comments: `//` and `/* */`
- Strings and numbers

### Code Completion

Triggered by typing or with `Ctrl+Space`:
- Keywords and type declarations
- Type names and annotations
- Query methods (`.filter`, `.map`, `.sort`, etc.)

### Diagnostics

Real-time error detection and reporting:
- Syntax errors
- Semantic errors
- Type checking errors

### Hover Information

Hover over any symbol to see documentation and type information.

## Example KQL Code

```kql
// User model
struct User {
    @auto_increment
    id: Key<i32>,
    name: String,
    email: String?,
    created_at: DateTime
}

// Post model with foreign key
struct Post {
    @auto_increment
    id: Key<i32>,
    title: String,
    content: String,
    @relation(name: "user_posts", foreign_key: "user_id")
    author: ForeignKey<User>,
    created_at: DateTime
}

// Query examples
let active_users = User.filter { $.created_at > "2024-01-01" }
let popular_posts = Post.sort { $.id.desc() }.take(10)
```

## Requirements

- VS Code 1.85.0 or higher
- kql-lsp binary (bundled or custom path)

## License

MPL-2.0

## Repository

https://github.com/aster-practice/kql-engine
