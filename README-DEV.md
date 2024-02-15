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

## Warp backend

Run `npm run watch` and then `cargo run -p backend-warp -- --demo` or `cd
backend-warp; cargo run -- --demo`.

## Tauri Backend

- Run `cargo tauri dev -- -- --demo` **from the root dir**.

# Why Codemirror 5 and other possibilities (TODO: Links, flesh out)

- Codemirror 6 has a merge plugin, but only two-way
- mismerge. TODO: File FRs. Delete one side. Do not collapse missing lines.

### Codemirror 5 has its weirdness
- Omitting unchanged blocks is buggy when changing whether the lines are wrapped.
- Diff quality is far from perfect
- No dark theme support
- Hard to tell blank lines from missing lines.
- Who knows how soon it'll be abandoned and unsupported.

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