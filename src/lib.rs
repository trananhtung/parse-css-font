//! # parse-css-font — parse the CSS `font` shorthand
//!
//! Split a CSS [`font`](https://developer.mozilla.org/en-US/docs/Web/CSS/font)
//! shorthand value into its parts — style, variant, weight, stretch, size,
//! line-height and the font-family list — or recognise a system-font keyword. A
//! faithful Rust port of the [`parse-css-font`](https://www.npmjs.com/package/parse-css-font)
//! npm package (handy for canvas text, PDF generation, and layout tooling). Zero
//! dependencies and `#![no_std]`.
//!
//! ```
//! use parse_css_font::{parse, Font, LineHeight};
//!
//! let Font::Shorthand(f) = parse("italic bold 12px/1.5 Arial, sans-serif").unwrap() else {
//!     panic!("expected a shorthand");
//! };
//! assert_eq!(f.style, "italic");
//! assert_eq!(f.weight, "bold");
//! assert_eq!(f.size, "12px");
//! assert_eq!(f.line_height, LineHeight::Number(1.5));
//! assert_eq!(f.family, ["Arial", "sans-serif"]);
//!
//! // System fonts are returned as-is.
//! assert!(matches!(parse("caption"), Ok(Font::System(_))));
//! ```

#![no_std]
#![doc(html_root_url = "https://docs.rs/parse-css-font/0.1.0")]

extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

// Compile-test the README's examples as part of `cargo test`.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

const SYSTEM_FONTS: [(&str, SystemFont); 6] = [
    ("caption", SystemFont::Caption),
    ("icon", SystemFont::Icon),
    ("menu", SystemFont::Menu),
    ("message-box", SystemFont::MessageBox),
    ("small-caption", SystemFont::SmallCaption),
    ("status-bar", SystemFont::StatusBar),
];

const STYLE_KEYWORDS: [&str; 3] = ["normal", "italic", "oblique"];
const WEIGHT_KEYWORDS: [&str; 13] = [
    "normal", "bold", "bolder", "lighter", "100", "200", "300", "400", "500", "600", "700", "800",
    "900",
];
const STRETCH_KEYWORDS: [&str; 9] = [
    "normal",
    "condensed",
    "semi-condensed",
    "extra-condensed",
    "ultra-condensed",
    "expanded",
    "semi-expanded",
    "extra-expanded",
    "ultra-expanded",
];
const SIZE_KEYWORDS: [&str; 9] = [
    "xx-small", "x-small", "small", "medium", "large", "x-large", "xx-large", "larger", "smaller",
];

/// A parsed CSS `font` value: either a system-font keyword or the shorthand parts.
#[derive(Debug, Clone, PartialEq)]
pub enum Font {
    /// A system-font keyword such as `caption` or `menu`.
    System(SystemFont),
    /// The parsed `font` shorthand components.
    Shorthand(Shorthand),
}

/// One of the six CSS system-font keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemFont {
    /// `caption`
    Caption,
    /// `icon`
    Icon,
    /// `menu`
    Menu,
    /// `message-box`
    MessageBox,
    /// `small-caption`
    SmallCaption,
    /// `status-bar`
    StatusBar,
}

impl SystemFont {
    /// The keyword as it appears in CSS.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            SystemFont::Caption => "caption",
            SystemFont::Icon => "icon",
            SystemFont::Menu => "menu",
            SystemFont::MessageBox => "message-box",
            SystemFont::SmallCaption => "small-caption",
            SystemFont::StatusBar => "status-bar",
        }
    }
}

impl fmt::Display for SystemFont {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The components of a parsed `font` shorthand. Each property defaults to `"normal"`
/// when not specified, matching the CSS `font` shorthand.
#[derive(Debug, Clone, PartialEq)]
pub struct Shorthand {
    /// `font-style`, e.g. `"normal"`, `"italic"`, `"oblique"`.
    pub style: String,
    /// `font-variant`, e.g. `"normal"`, `"small-caps"`.
    pub variant: String,
    /// `font-weight`, e.g. `"normal"`, `"bold"`, `"400"`.
    pub weight: String,
    /// `font-stretch`, e.g. `"normal"`, `"condensed"`.
    pub stretch: String,
    /// `font-size`, e.g. `"12px"`, `"1em"`, `"x-large"`.
    pub size: String,
    /// `line-height`.
    pub line_height: LineHeight,
    /// The `font-family` list, in order, with surrounding quotes removed.
    pub family: Vec<String>,
}

/// A `line-height` value.
#[derive(Debug, Clone, PartialEq)]
pub enum LineHeight {
    /// `normal` (the default).
    Normal,
    /// A unitless multiplier, e.g. `1.5`.
    Number(f64),
    /// A value with a unit or other form, e.g. `"14px"`.
    Other(String),
}

impl fmt::Display for LineHeight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LineHeight::Normal => f.write_str("normal"),
            LineHeight::Number(n) => write!(f, "{n}"),
            LineHeight::Other(s) => f.write_str(s),
        }
    }
}

/// An error returned when a `font` value cannot be parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    /// The input was an empty string.
    EmptyString,
    /// No `font-size` was found.
    MissingFontSize,
    /// No `font-family` was found.
    MissingFontFamily,
    /// `font-style` was specified more than once.
    DuplicateStyle,
    /// `font-weight` was specified more than once.
    DuplicateWeight,
    /// `font-stretch` was specified more than once.
    DuplicateStretch,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ParseError::EmptyString => "cannot parse an empty string",
            ParseError::MissingFontSize => "missing required font-size",
            ParseError::MissingFontFamily => "missing required font-family",
            ParseError::DuplicateStyle => "font-style already defined",
            ParseError::DuplicateWeight => "font-weight already defined",
            ParseError::DuplicateStretch => "font-stretch already defined",
        })
    }
}

impl core::error::Error for ParseError {}

/// Parse a CSS `font` shorthand value.
///
/// # Errors
///
/// Returns [`ParseError`] for an empty string, a value missing the required
/// `font-size` or `font-family`, or a property specified more than once.
///
/// ```
/// use parse_css_font::{parse, Font};
/// assert!(matches!(parse("12px serif"), Ok(Font::Shorthand(_))));
/// assert!(parse("12px").is_err()); // no family
/// ```
pub fn parse(value: &str) -> Result<Font, ParseError> {
    if value.is_empty() {
        return Err(ParseError::EmptyString);
    }
    if let Some(system) = system_font(value) {
        return Ok(Font::System(system));
    }

    let mut style = String::new();
    let mut variant = String::new();
    let mut weight = String::new();
    let mut stretch = String::new();
    let mut line_height = LineHeight::Normal;

    let tokens = split_by_spaces(value);
    let mut idx = 0;
    while idx < tokens.len() {
        let token = tokens[idx].as_str();
        idx += 1;

        if token == "normal" {
            continue;
        }
        if STYLE_KEYWORDS.contains(&token) {
            if !style.is_empty() {
                return Err(ParseError::DuplicateStyle);
            }
            token.clone_into(&mut style);
            continue;
        }
        if WEIGHT_KEYWORDS.contains(&token) {
            if !weight.is_empty() {
                return Err(ParseError::DuplicateWeight);
            }
            token.clone_into(&mut weight);
            continue;
        }
        if STRETCH_KEYWORDS.contains(&token) {
            if !stretch.is_empty() {
                return Err(ParseError::DuplicateStretch);
            }
            token.clone_into(&mut stretch);
            continue;
        }
        if !is_size(token) {
            // The variant slot is the catch-all for any remaining non-size token.
            variant = if variant.is_empty() {
                token.to_owned()
            } else {
                format!("{variant} {token}")
            };
            continue;
        }

        // This token is the font-size (optionally `size/line-height`).
        let parts = split_on_slash(token);
        let size = parts[0].clone();
        if parts.len() > 1 && !parts[1].is_empty() {
            line_height = parse_line_height(&parts[1]);
        } else if tokens.get(idx).map(String::as_str) == Some("/") {
            idx += 1; // consume the standalone "/"
            if let Some(lh) = tokens.get(idx) {
                line_height = parse_line_height(lh);
                idx += 1;
            }
        }

        if idx >= tokens.len() {
            return Err(ParseError::MissingFontFamily);
        }
        let family = split_by_commas(&tokens[idx..].join(" "))
            .iter()
            .map(|f| unquote(f))
            .collect();

        return Ok(Font::Shorthand(Shorthand {
            style: or_normal(style),
            variant: or_normal(variant),
            weight: or_normal(weight),
            stretch: or_normal(stretch),
            size,
            line_height,
            family,
        }));
    }

    Err(ParseError::MissingFontSize)
}

fn or_normal(value: String) -> String {
    if value.is_empty() {
        "normal".to_string()
    } else {
        value
    }
}

/// Match the whole `value` against a system-font keyword (case-sensitive).
fn system_font(value: &str) -> Option<SystemFont> {
    SYSTEM_FONTS
        .iter()
        .find(|(kw, _)| *kw == value)
        .map(|(_, font)| *font)
}

/// Whether a token is a font-size: it starts with a digit or `.`, contains `/`, or
/// is a font-size keyword (mirrors css-font's `isSize`).
fn is_size(token: &str) -> bool {
    token
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_digit() || c == '.')
        || token.contains('/')
        || SIZE_KEYWORDS.contains(&token)
}

/// Classify a line-height value the way `parse-css-font` does: a bare number becomes
/// [`LineHeight::Number`]; `normal` is [`LineHeight::Normal`]; anything else is
/// [`LineHeight::Other`].
fn parse_line_height(value: &str) -> LineHeight {
    if value == "normal" {
        return LineHeight::Normal;
    }
    if value.starts_with(|c: char| c.is_ascii_digit() || c == '+' || c == '-' || c == '.') {
        if let Ok(n) = value.parse::<f64>() {
            if n.is_finite() && format!("{n}") == value {
                return LineHeight::Number(n);
            }
        }
    }
    LineHeight::Other(value.to_owned())
}

/// Strip one leading and one trailing quote character (either `"` or `'`, and they
/// need not match), like the `unquote` npm package. Nothing is trimmed — the split
/// helpers already trim each part.
fn unquote(value: &str) -> String {
    let mut s = value;
    if s.starts_with(['"', '\'']) {
        s = &s[1..];
    }
    if s.ends_with(['"', '\'']) {
        s = &s[..s.len() - 1];
    }
    s.to_owned()
}

// ---------------------------------------------------------------------------
// Splitting helpers — a faithful port of css-list-helpers `split`: nests on `()`
// only, honors quoted strings with `\` escapes, and trims each emitted part.
// ---------------------------------------------------------------------------

/// Split on top-level whitespace (` `, `\n`, `\t`), dropping empty parts.
fn split_by_spaces(s: &str) -> Vec<String> {
    split(s, |c| c == ' ' || c == '\n' || c == '\t', false)
}

/// Split on top-level commas; the final (possibly empty) part is always kept.
fn split_by_commas(s: &str) -> Vec<String> {
    split(s, |c| c == ',', true)
}

/// Split a single size token on `/`.
fn split_on_slash(s: &str) -> Vec<String> {
    split(s, |c| c == '/', false)
}

/// Split `value` on `is_sep` at the top level: separators inside `(…)` or a quoted
/// string (honoring `\` escapes) do not split. Each emitted part is trimmed; when
/// `last` is set, the final part is emitted even if empty.
fn split(value: &str, is_sep: impl Fn(char) -> bool, last: bool) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut func: i32 = 0;
    let mut quote: Option<char> = None;
    let mut escape = false;

    for c in value.chars() {
        let mut split_here = false;
        if let Some(q) = quote {
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == q {
                quote = None;
            }
        } else if c == '"' || c == '\'' {
            quote = Some(c);
        } else if c == '(' {
            func += 1;
        } else if c == ')' {
            if func > 0 {
                func -= 1;
            }
        } else if func == 0 && is_sep(c) {
            split_here = true;
        }

        if split_here {
            if !current.is_empty() {
                parts.push(current.trim().to_owned());
            }
            current.clear();
        } else {
            current.push(c);
        }
    }
    if last || !current.is_empty() {
        parts.push(current.trim().to_owned());
    }
    parts
}
