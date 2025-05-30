---
applyTo: '**/*.rs,**/Cargo.toml'
---

When introducing new dependencies, add them to the workspace `Cargo.toml` file.
Always use workspace dependencies instead of version or path attributes in crate `Cargo.toml` files.
Use `{crate_name}.workspace = true` syntax when applicable.

When introducing new crates, add them to the workspace `Cargo.toml` file.

Use `imports_granularity = "Item"` to do java-style imports, one per line.

Do not pass `-p {}` to `cargo check`, just run `cargo check` without arguments.

After introducing a new dependency, run `cargo check` to ensure that the workspace is in a valid state.