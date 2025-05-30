---
applyTo: '**/*.rs,**/Cargo.toml'
---

When introducing new dependencies, add them to the workspace `Cargo.toml` file.
Always use workspace dependencies instead of version or path attributes in crate `Cargo.toml` files.

When introducing new crates, add them to the workspace `Cargo.toml` file.

Use `imports_granularity = "Item"` to do java-style imports, one per line.