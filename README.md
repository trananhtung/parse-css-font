# parse-css-font

[![All Contributors](https://img.shields.io/badge/all_contributors-1-orange.svg?style=flat-square)](#contributors-)

[![Crates.io](https://img.shields.io/crates/v/parse-css-font.svg)](https://crates.io/crates/parse-css-font)
[![Documentation](https://docs.rs/parse-css-font/badge.svg)](https://docs.rs/parse-css-font)
[![CI](https://github.com/trananhtung/parse-css-font/actions/workflows/ci.yml/badge.svg)](https://github.com/trananhtung/parse-css-font/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/parse-css-font.svg)](#license)

**Parse the CSS [`font`](https://developer.mozilla.org/en-US/docs/Web/CSS/font)
shorthand** into its parts — style, variant, weight, stretch, size, line-height and
the font-family list — or recognise a system-font keyword. A faithful Rust port of
the [`parse-css-font`](https://www.npmjs.com/package/parse-css-font) npm package.
Zero dependencies and `#![no_std]`.

```rust
use parse_css_font::{parse, Font, LineHeight};

let Font::Shorthand(f) = parse("italic bold 12px/1.5 Arial, sans-serif").unwrap() else {
    unreachable!()
};
assert_eq!(f.style, "italic");
assert_eq!(f.weight, "bold");
assert_eq!(f.size, "12px");
assert_eq!(f.line_height, LineHeight::Number(1.5));
assert_eq!(f.family, ["Arial", "sans-serif"]);

assert!(matches!(parse("caption"), Ok(Font::System(_))));
```

## Why parse-css-font?

The `font` shorthand packs up to seven properties into one string with an order-
independent prefix, an optional `size/line-height`, and a quoted, comma-separated
family list. Canvas text rendering, PDF generation, and layout tools all need to
pull those apart. This is the small, dependency-free piece that does exactly that —
matching the canonical JS implementation.

```toml
[dependencies]
parse-css-font = "0.1"
```

## API

| Item | Purpose |
| --- | --- |
| `parse(value)` | `Result<Font, ParseError>` |
| `Font::System(SystemFont)` | A system-font keyword (`caption`, `icon`, `menu`, …) |
| `Font::Shorthand(Shorthand)` | The parsed parts |
| `Shorthand { style, variant, weight, stretch, size, line_height, family }` | Strings, with `family: Vec<String>` |
| `LineHeight` | `Normal`, `Number(f64)` (unitless), or `Other(String)` (e.g. `"14px"`) |

## Behavior

- `style`, `variant`, `weight`, and `stretch` may appear in any order before the
  size, and each defaults to `"normal"`.
- The `font-size` is required; a `/line-height` after it is optional. A unitless
  line-height becomes `LineHeight::Number`, a `normal` one becomes `Normal`, and any
  other (e.g. `14px`) becomes `Other`.
- `font-family` is required, split on top-level commas, with surrounding quotes
  removed.
- System-font keywords are matched exactly (case-sensitive) and returned as
  `Font::System`.
- Missing size/family, an empty string, or a repeated property returns a
  `ParseError`.

## Contributors ✨

This project follows the [all-contributors](https://github.com/all-contributors/all-contributors) specification. Contributions of any kind are welcome — code, docs, bug reports, ideas, reviews! See the [emoji key](https://allcontributors.org/docs/en/emoji-key) for how each contribution is recognized, and open a PR or issue to get involved.

Thanks goes to these wonderful people:

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/trananhtung"><img src="https://avatars.githubusercontent.com/u/30992229?v=4?s=100" width="100px;" alt="Tung Tran"/><br /><sub><b>Tung Tran</b></sub></a><br /><a href="https://github.com/trananhtung/parse-css-font/commits?author=trananhtung" title="Code">💻</a> <a href="#maintenance-trananhtung" title="Maintenance">🚧</a></td>
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at
your option.
