# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.0-rc.1](https://github.com/Michael-F-Bryan/gcode-rs/compare/v0.7.0-beta...v0.7.0-rc.1) (2026-03-17)


### Features

* hoisted the contents of the `ast` module to the top level ([259f685](https://github.com/Michael-F-Bryan/gcode-rs/commit/259f68544b91902871311ce2e09c5a67e4f06752))
* Syntax errors will now include the expected tokens as a `TokenType` instead of their string representation ([424a5b2](https://github.com/Michael-F-Bryan/gcode-rs/commit/424a5b25ac3d89345237cdc86af31f263ceb5dcd))


### Bug Fixes

* Make sure the `DiagnosticKind` is public ([152d89a](https://github.com/Michael-F-Bryan/gcode-rs/commit/152d89ac17be686961f9f09b548de8c4ff6bad41))


### Miscellaneous Chores

* release 0.7.0-rc.1 ([44c5649](https://github.com/Michael-F-Bryan/gcode-rs/commit/44c56494ad77cf3fdc549699d1773b6678622159))

## [0.7.0-beta](https://github.com/Michael-F-Bryan/gcode-rs/compare/v0.7.0-alpha...v0.7.0-beta) (2026-03-17)


### doc

* Adding examples to the README ([bf4569f](https://github.com/Michael-F-Bryan/gcode-rs/commit/bf4569f826584ca00bd5054a0430c23ab23dbccb))


### Features

* feat:  ([bf4569f](https://github.com/Michael-F-Bryan/gcode-rs/commit/bf4569f826584ca00bd5054a0430c23ab23dbccb))
* **ast:** add Display impls for g-code round-trip ([806699a](https://github.com/Michael-F-Bryan/gcode-rs/commit/806699a105197246d9ff08c28496dbc182f2bda2))

## [0.7.0-alpha](https://github.com/Michael-F-Bryan/gcode-rs/compare/v0.6.0...v0.7.0-alpha) (2026-03-17)


### ⚠ BREAKING CHANGES

* Public API replaced. Removed: parse(), full_parse_with_callbacks(), Parser, GCode, Line, Word, Callbacks, Buffers, Comment, Span, and related types. Use core::parse() with a ProgramVisitor for push-based parsing, or ast::parse() (with alloc feature) for an alloc-based Program and Diagnostics.
* upgrade to Rust 1.85 and edition 2024

### Features

* **ast:** add alloc-based AST and parse() returning Program and Diagnostics ([63abac0](https://github.com/Michael-F-Bryan/gcode-rs/commit/63abac0a1bb23eee8b900f052ebd95aee7c177fe))
* **core:** implement push-based parser in resume() ([029d5f5](https://github.com/Michael-F-Bryan/gcode-rs/commit/029d5f5d1efd06b251d6061adbfa340680636bc0))
* **core:** implement push-based parser with TDD (empty input, single G-code) ([264b76f](https://github.com/Michael-F-Bryan/gcode-rs/commit/264b76fd95d4aa6aac07f77074d325a42073716b))
* **examples:** add pretty-print visitor example ([c110e4b](https://github.com/Michael-F-Bryan/gcode-rs/commit/c110e4b7a6cbe40820fa2d0fee5353b735828d0f))
* Introduced a `core` module which is based around a push-based parser ([5c0e230](https://github.com/Michael-F-Bryan/gcode-rs/commit/5c0e23091dcf7f3a7917937d26b76b9050b10f5d))
* **parser:** add % delimiter, modal word addresses, G/M/T token types ([16ef482](https://github.com/Michael-F-Bryan/gcode-rs/commit/16ef48289d1f872fedd90a3feed054b10b14a48b))
* upgrade to Rust 1.85 and edition 2024 ([fc94c1d](https://github.com/Michael-F-Bryan/gcode-rs/commit/fc94c1d8aef3ed47c229ae75c2298b8fb876925c))


### Bug Fixes

* **ast:** record M and T codes in BlockBuilder ([fd9f815](https://github.com/Michael-F-Bryan/gcode-rs/commit/fd9f8157775342f366edf7b8a8a9e85bcb92bca1))
* **core:** silence unused_assignments in feed_line; add code+comment and regression tests ([bca2d87](https://github.com/Michael-F-Bryan/gcode-rs/commit/bca2d87da09e815df96629dcd42a6ff663409a1e))


### Miscellaneous Chores

* release 0.7.0-alpha ([bf076b5](https://github.com/Michael-F-Bryan/gcode-rs/commit/bf076b53264a25eb8b51035aaa077880fae7f363))


### Code Refactoring

* replace public API with core and ast, remove iterator/callback API ([a63fec1](https://github.com/Michael-F-Bryan/gcode-rs/commit/a63fec1bd529ef7c444eb8d2d394de7c03d95d72))

## [0.6.1]

### Added

- WebAssembly build and npm wrapper with TypeScript bindings ([#45](https://github.com/Michael-F-Bryan/gcode-rs/issues/45)).
- CI job for testing WebAssembly bindings.

### Fixed

- Span handling bug that could produce incorrect span data ([#45](https://github.com/Michael-F-Bryan/gcode-rs/issues/45)).

### Removed

- Unused FFI and kinematics crates.

### Changed

- Dependency bumps; CI and Appveyor path updates.

## [0.6.0]

### Added

- Configurable buffer sizes and parser facade for custom buffers ([#41](https://github.com/Michael-F-Bryan/gcode-rs/issues/41)).
- `DefaultArguments` type alias as default buffer for `GCode` ([#41](https://github.com/Michael-F-Bryan/gcode-rs/issues/41)).
- Additional docs, examples, and crate-level documentation for Spans ([#42](https://github.com/Michael-F-Bryan/gcode-rs/issues/42)).
- Redirect from `docs/` to gcode docs ([#42](https://github.com/Michael-F-Bryan/gcode-rs/issues/42)).
- Explicit `Debug` impls for parsing results and `Span::PLACEHOLDER` equivalence ([#42](https://github.com/Michael-F-Bryan/gcode-rs/issues/42)).

### Fixed

- Lexer now passes through unknown characters correctly ([#41](https://github.com/Michael-F-Bryan/gcode-rs/issues/41)).
- Lexer handling of numbers with an explicit `+` sign ([#41](https://github.com/Michael-F-Bryan/gcode-rs/issues/41)).
- `WordsOrComments` state bug (saved `last_letter` as member) ([#41](https://github.com/Michael-F-Bryan/gcode-rs/issues/41)).

### Changed

- Re-licensed under MIT OR Apache-2.0 ([#41](https://github.com/Michael-F-Bryan/gcode-rs/issues/41)).
- Parser module split into smaller chunks; parser facade exposed for users ([#41](https://github.com/Michael-F-Bryan/gcode-rs/issues/41)).
- Serde compiles without `std`; MSRV and feature-flag CI updates ([#41](https://github.com/Michael-F-Bryan/gcode-rs/issues/41)).
- Mentioned @etrombly's showcase website (fixes #32).

## [0.5.2]

### Fixed

- `arrayvec/std` feature is only enabled when `std` feature is enabled (fixes #39).
- Replaced `f32` methods with libm equivalents for `no_std` (fixes #40).

## [0.5.1]

### Added

- Kinematics crate (separate workspace crate).
- Packaging script and insulpro example file.

### Changed

- Relaxed trait bounds on FFI Callbacks.
- Removed `gcode-` prefix from `gcode-ffi` crate name.

### Removed

- `it_works()` placeholder test.

## [0.5.0]

### Added

- Complete rewrite with nom-based parser, tokeniser, and line-number support ([#38](https://github.com/Michael-F-Bryan/gcode-rs/issues/38)).
- Serde support (optional) ([#38](https://github.com/Michael-F-Bryan/gcode-rs/issues/38)).
- `no_std` support and benchmarks ([#38](https://github.com/Michael-F-Bryan/gcode-rs/issues/38)).
- Smoke test suite ([#38](https://github.com/Michael-F-Bryan/gcode-rs/issues/38)).
- C bindings and example C program ([#37](https://github.com/Michael-F-Bryan/gcode-rs/issues/37)).
- docs.rs metadata ([#38](https://github.com/Michael-F-Bryan/gcode-rs/issues/38)).

### Fixed

- Bug where parsing could exit from lines too early ([#38](https://github.com/Michael-F-Bryan/gcode-rs/issues/38)).

### Changed

- Upgraded to 2018 edition; bumped minimum Rust version and dependencies ([#37](https://github.com/Michael-F-Bryan/gcode-rs/issues/37), [#38](https://github.com/Michael-F-Bryan/gcode-rs/issues/38)).
- FFI bindings and C example improvements ([#37](https://github.com/Michael-F-Bryan/gcode-rs/issues/37)).

## [0.4.0]

### Added

- New lexer and parser (recursive descent, state machine); comment and block parsing ([#28](https://github.com/Michael-F-Bryan/gcode-rs/issues/28)).
- G-code operations: transforms, helpers, `FromGcode` trait, axis positions with uom ([#28](https://github.com/Michael-F-Bryan/gcode-rs/issues/28)).
- Basic simulator and transformation/operations module behind feature flag ([#28](https://github.com/Michael-F-Bryan/gcode-rs/issues/28)).
- `large-buffers` feature flag ([#28](https://github.com/Michael-F-Bryan/gcode-rs/issues/28)).
- Comments use `Cow<str>`; parser skips erroneous input ([#28](https://github.com/Michael-F-Bryan/gcode-rs/issues/28)).
- README badges ([#33](https://github.com/Michael-F-Bryan/gcode-rs/issues/33)).

### Fixed

- Floating point rounding in coordinate/dwell handling ([#28](https://github.com/Michael-F-Bryan/gcode-rs/issues/28)).
- Arguments and comments documentation ([#28](https://github.com/Michael-F-Bryan/gcode-rs/issues/28)).

### Changed

- Operations refactored to `TryFrom<&Gcode>`; associated constants instead of hard-coded values ([#28](https://github.com/Michael-F-Bryan/gcode-rs/issues/28)).
- Cargo.toml metadata (fixes #29) ([#28](https://github.com/Michael-F-Bryan/gcode-rs/issues/28)).
- Example of mutating a gcode's arguments (cc: @MicroJoe) ([#28](https://github.com/Michael-F-Bryan/gcode-rs/issues/28)).

### Removed

- `src` field from `Block` ([#28](https://github.com/Michael-F-Bryan/gcode-rs/issues/28)).

## [0.3.2]

### Fixed

- `#[deprecated]` attribute used correctly for deprecated API (fixes #34).

## [0.3.1]

### Added

- Writing gcodes via `core::fmt::Write`.
- README example; CLI example reads all input.
- CI tests with `--all-features`; README example tested via rustdoc.

### Deprecated

- `Gcode::number()` method (fixes #23).

### Changed

- FFI bindings are now off by default.
- Bumped minimum Rust version; Windows build tweaks.

## [0.3.0]

### Added

- Fresh start with nom-based parser; tokeniser, words iterator, round-trippable numbers ([#21](https://github.com/Michael-F-Bryan/gcode-rs/issues/21)).
- FFI feature and C example ([#21](https://github.com/Michael-F-Bryan/gcode-rs/issues/21)).
- Line number support and access on gcode ([#21](https://github.com/Michael-F-Bryan/gcode-rs/issues/21)).
- Expanded docs and basic usage example ([#21](https://github.com/Michael-F-Bryan/gcode-rs/issues/21)).

### Changed

- Parser public API; CI and build matrix updates ([#21](https://github.com/Michael-F-Bryan/gcode-rs/issues/21)).

## [0.2.1]

### Fixed

- E argument supported in new parser ([#18](https://github.com/Michael-F-Bryan/gcode-rs/issues/18)).

### Changed

- Inlined Token methods; parser performance improvement (~20%) ([#18](https://github.com/Michael-F-Bryan/gcode-rs/issues/18)).

## [0.2.0]

### Added

- New parser with command type and argument recognition ([#16](https://github.com/Michael-F-Bryan/gcode-rs/issues/16)).
- QuickCheck tests for parser ([#16](https://github.com/Michael-F-Bryan/gcode-rs/issues/16)).
- Documentation for new parser ([#16](https://github.com/Michael-F-Bryan/gcode-rs/issues/16)).

### Deprecated

- Old (Basic) parser ([#16](https://github.com/Michael-F-Bryan/gcode-rs/issues/16)).

### Removed

- High-level module ([#16](https://github.com/Michael-F-Bryan/gcode-rs/issues/16)).
- Nightly feature and NIGHTLY Travis flag ([#16](https://github.com/Michael-F-Bryan/gcode-rs/issues/16)).

## [0.1.0]

### Added

- Initial lexer and low-level parser for gcode ([#13](https://github.com/Michael-F-Bryan/gcode-rs/issues/13)).
- Support for program numbers, line numbers, and basic command arguments ([#13](https://github.com/Michael-F-Bryan/gcode-rs/issues/13)).
- High-level type-check module and ArgumentReader for interpreting command args ([#13](https://github.com/Michael-F-Bryan/gcode-rs/issues/13)).
- G01, G02 (with I, J), G04 (dwell) ([#9](https://github.com/Michael-F-Bryan/gcode-rs/issues/9)).
- QuickCheck and fuzzing support ([#11](https://github.com/Michael-F-Bryan/gcode-rs/issues/11), [#13](https://github.com/Michael-F-Bryan/gcode-rs/issues/13)).
- Case-insensitive parsing via `SwapCase` trait ([#12](https://github.com/Michael-F-Bryan/gcode-rs/issues/12)).
- Benches, examples, and Display impl ([#13](https://github.com/Michael-F-Bryan/gcode-rs/issues/13)).
- Contributing guide and CI with doc upload ([#13](https://github.com/Michael-F-Bryan/gcode-rs/issues/13)).

### Fixed

- Removed M as argument kind (fixes #5) ([#13](https://github.com/Michael-F-Bryan/gcode-rs/issues/13)).
- Error message for unknown M code now says M not G ([#14](https://github.com/Michael-F-Bryan/gcode-rs/issues/14)).
- Lexer panic found via QuickCheck ([#13](https://github.com/Michael-F-Bryan/gcode-rs/issues/13)).
- `no_std` compatibility (arrayvec, logging removed where it pulled in std) ([#13](https://github.com/Michael-F-Bryan/gcode-rs/issues/13)).

[0.6.2-alpha.0]: https://github.com/Michael-F-Bryan/gcode-rs/compare/gcode-v0.6.1...HEAD
[0.6.1]: https://github.com/Michael-F-Bryan/gcode-rs/compare/v0.6.0...gcode-v0.6.1
[0.6.0]: https://github.com/Michael-F-Bryan/gcode-rs/compare/v0.5.2...v0.6.0
[0.5.2]: https://github.com/Michael-F-Bryan/gcode-rs/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/Michael-F-Bryan/gcode-rs/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/Michael-F-Bryan/gcode-rs/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/Michael-F-Bryan/gcode-rs/compare/v0.3.2...v0.4.0
[0.3.2]: https://github.com/Michael-F-Bryan/gcode-rs/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/Michael-F-Bryan/gcode-rs/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/Michael-F-Bryan/gcode-rs/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/Michael-F-Bryan/gcode-rs/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/Michael-F-Bryan/gcode-rs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/Michael-F-Bryan/gcode-rs/releases/tag/v0.1.0
