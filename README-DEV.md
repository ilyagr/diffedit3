# Prerequisites

## While `dist` is not checked in

(TODO: Check it in, create a github check)

- Install `node`, including `npm`.
- Run `npm i`.

## For `tauri`
- Install `cargo-tauri`

# Running the app while developing

Either way, changes to `webapp/` will be visible immediately. With the Tauri
backend, it will recompile the Rust code on changes as well. With the Warp
backend, you have to do that manually.

## Local Server backend

Run `npm run watch` and then `cargo run -p backend-local-server -- --demo` or
`cd backend-local-server; cargo run -- --demo`.

## Tauri Backend

- Run `cargo tauri dev -- -- --demo` **from the root dir**.

# Tech choices

## Why Codemirror 5 and other possibilities (TODO: Links, flesh out)

- Codemirror 6 has a merge plugin, but only two-way
- mismerge. TODO: File FRs. Delete one side. Do not collapse missing lines.

### Codemirror 5 has its weirdness
- Omitting unchanged blocks is buggy when changing whether the lines are wrapped.
- Diff quality is far from perfect
- No dark theme support
- Hard to tell blank lines from missing lines.
- Who knows how soon it'll be abandoned and unsupported.

## `poem` backend

I first tried `warp`, but once the error messages passed a mile mark, decided to
switch.

`poem` was the next framework that I could immediately find relevant examples
(embedding a website into a binary with `rust_embed` and error handling) that
seemed straightforward, seems to avoid making everything a macro, and claims to
try to keep things to have types with names of finite length.

If that doesn't work out, I'd try `axum` next and the `arctix` based on blogs
and Reddit.

# TODOs

## Better merging
Simpler: look into configuring parameters to the diff library.

Hard: Do calculations in Rust using a newer library. Would require a UI that can
import such a diff. `mismerge` is close.

## Warning for unsaved changes, fancier save button

Keep track of whether the content changed since last save. Grey out the save
button in this case. (This means that while saving, we need to keep track of
both the last successfully saved state and the state we're trying to save). Tell
the user when the last successful save happened.

## Save per-file

Simple part: don't overwrite unchanged files

Harder: separate save buttons per file and maybe an overall save button.