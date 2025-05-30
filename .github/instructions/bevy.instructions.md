---
applyTo: '**/*.rs,**/Cargo.toml'
---

Use `.write` since `.send` is deprecated on `EventWriter<T>`

In plugins, do not chain `app.add_systems` style calls. Place each `app.whatever` on a new line instead.

New plugins should be added as a dependency for `crates/welcome_gui` and should have `app.add_plugins` called for it.

`crates\worker_plugin\examples\simple.rs` is a good reference for when encountering type issues involving worker plugins.
Note that the `handle_threadbound_message` type signature must match.

Add `#[reflect(Resource)]` to resources with `#[derive(Reflect)]`.
Do not add `#[reflect(Event)]` to events with `#[derive(Reflect)]` since this does not exist.