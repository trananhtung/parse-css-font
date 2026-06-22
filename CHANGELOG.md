# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-06-22

### Added

- Initial release.
- `parse` — parse a CSS `font` shorthand into a `Font` (`System(SystemFont)` or
  `Shorthand`), or a `ParseError`.
- `Shorthand` with `style`, `variant`, `weight`, `stretch`, `size`, `line_height`,
  and `family`; `LineHeight` (`Normal` / `Number` / `Other`); `SystemFont`.
- Faithful to the `parse-css-font` npm package, including the order-independent
  prefix, `size/line-height` handling, quoted family lists, and system-font
  keywords. Zero dependencies; `#![no_std]`.

[0.1.0]: https://github.com/trananhtung/parse-css-font/releases/tag/v0.1.0
