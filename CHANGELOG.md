# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Breaking changes

* The library package is renamed to `diffedit3`, with a binary called
  `diffedit3-web`. The Tauri binary is still called `diffedit3-gui`.
  
* Line-wrapping defaults to on

### New features

* The web server version of diffedit3 can be used as a library

* Line-wrapping can now be toggled in the editor

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