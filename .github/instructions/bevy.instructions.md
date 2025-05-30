---
applyTo: '**/*.rs,**/Cargo.toml'
---

Use `.write` since `.send` is deprecated on `EventWriter<T>`

In plugins, do not chain `app.add_systems` style calls. Place each `app.whatever` on a new line instead.