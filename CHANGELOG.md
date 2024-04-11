# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Breaking changes

Renamed the `diffedit3-web` binary to `diffedit3`

The `diffedit3-gui` release artifact is now marked as unsupported in
its name.

### New features

`diffedit3` can now be installed with `cargo binstall diffedit3`.

### Fixed bugs


## [v0.2.0] - 2024-04-05

### Breaking changes

* Major version update to `poem` dependency

### New features

### Fixed bugs

* A bug that prevented saving the comparison when a file's dir existed on the
  left side but not on the right side.

### Maintenance

* There are now some tests; GitHub CI runs them.


## [v0.1.2] - 2024-03-18

### New features

* The editor now supports a "pinned" mode where one file merge takes up the
  whole app window.

* Minor visual design fixes/improvements

## [v0.1.1] - 2024-03-15

Mostly some internal code reorgaization.

### New features

* `diffedit3` now has a logo (and an icon, and a favicon). See the top of the
  README. :)


## [v0.1.0] - 2024-03-13

### Breaking changes

* The library package is renamed to `diffedit3`, with a binary called
  `diffedit3-web`. The Tauri binary is still called `diffedit3-gui`.
  
* Line-wrapping defaults to on

### New features

* The web server version of diffedit3 can be used as a library

* Line-wrapping, collapsing of long identical regions, and aligning of changes
  using blank lines can now be toggled in the editor.

* It is now possible to remove the right pane for a 2-pane view (left pane &
  output pane only)

* Binary and files over 200KiB will not be compared.

* diffedit3-web: new --port-range option allows trying a different port if the
  first port we try is already in use (e.g. another instance of diffedit3-web is
  running)

* diffedit3-web: a simple security check is added so that the tool will not
  overwrite files it wasn't meant to compare.

### Fixed bugs

Who's counting? Not me.

Some known existing bugs are mentioned at the bottom of the README.

## [0.0.1] - 2024-02-19

Initial release
