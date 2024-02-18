# What this is

`diffedit3` is intended mainly to be used with
[`jj`](https://github.com/martinvonz/jj). It could also be adapted for use with
`git` or other VCS, but interaction with such tools hasn't been implemented yet.

`diffedit3` presents the user with a three-way comparison between two or three
directories. The user can then edit the middle pane, creating any intermediate
version of the data they like. The most obvious use-cases are [splitting
commits](...) or as an interactive version of `git add -p` (pick a version of
your changes between the HEAD commit and the working copy to stage, essentially
editing the contents of the staging area; not implemented yet).

When used with `jj`, this is a substitude for using [`meld`](...) in the
[three-pane diffedit mode](..._). If you can use `meld` easily, I recommend you
use it, as `diffedit3` is far less polished. `diffedit3` is meant for
environments where it's difficult to use `meld`: MacOS, Windows, or over an SSH
connection (or some other environment where using a GUI is difficult, but port
forwarding is possible). It is also designed so that it can be bundled with
`jj`, as a proof of concept of a `jj` GUI.

`diffedit3` works much like Jupyter: it starts a local server, and opens up a
browser window with a GUI. After you are done, you hit a button, the browser
window closes and the local server exits. When using over SSH, you would need to
set up port forwarding to use it.

Currently, `diffedit3` is a wrapper around the CodeMirror5 merge plugin. This will hopefully [change in the future]. 

There is also an experimental GUI version of `diffedit3`, based on
[Tauri](https://tauri.app).

# Potential future features and alternatives

## Viewing diffs in a two-pane view

This is likely straightforward to do. There are many alternative tools, `meld` and `kdiff3` among them. Vim, Emacs.

## Resolving merge conflicts in a 4-pane view

This is not straightforward; I am not aware of a preexisting JS library that implements such a UI.

As an alternative for use with `jj`, I recommend setting up either
`kdiff3` or VS Code as a merge tool. The latter can be used remotely using
<...>, if you can install VS Code on the remote machine.

## UI Changes, e.g. a tabbed view instead of scrolling, change size of the editors, ...

Patches welcome :). Any such changes run the risk of running into CodeMirror5's merge plugin's bugs, so be careful checking that your changes work.

## Word wrapping, buttons to take all changes from  the left/right pane

This is planned. This requires destroying the CodeMirror widget and creating a
new one (again, to avoid bugs).

# Building (TODO: Check in `dist` to make this possible)

For the local server version, running `cargo build --release` should suffice. This assumes that the compiled javascript in `webapp/dist` is up to date and therefore does not require installing `npm`.

For the GUI (Tauri) version, the procedure is:

1. Install Rust
2. Install the `cargo-tauri` CLI (TODO: Better explanation.). Do not use the `node` Tauri CLI; it does not seem to work with the Rust-focused directory structure of this project.
3. Install `node`/`npm`. `cd` to the `backend-tauri` dir.
4. Run `npm install` (only has to be done once).
5. Run `cargo tauri build`. Do **not** use the regular `cargo build`, it produces
   a broken binary.

In both cases, the binary will be produced in the `target/release` dir.

## Git interaction

- Better dirdiff
- Staging
- Either make git (link) or use gix/libgit2

# Development

## Prerequisites


## Option 1: using `backend-local-server`

1. (If you are planning to touch HTML/CSS/JS) Install `node`/`npm`, run `npm
  install`.
2. (If you are touching HTML/CSS/JS) Run `npm run watch` in a separate terminal. This will auto-update `webapp/dist` whenever you change files in `webapp/`.
3. Use `cargo run -p backend-local-server -- --demo` to run the app. If touching
   the webapp, remember to manually make sure that `npm run watch` is not
   showing any errors before running/building the app. 

## Option 2: using `backend-tauri`

1. Follow the steps of the "building" procedure for the GUI/Tauri version. You can skip the last buiding step.
2. Run `cargo tauri dev -- -- --demo` (note the two `--`s). This will
   automatically refresh the app to use the latest version of the webapp from
   `webapp/` (as opposed to `webapp/dist) in dev mode. 

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


## Make it work with git?

Might be hard, see https://stackoverflow.com/questions/9023928/git-add-patch-with-difftool

Maybe `git-edit-staging-area`?
