//! Behavioral spec for `parse-css-font`, cross-checked against the npm package.

use parse_css_font::{parse, Font, LineHeight, ParseError, Shorthand, SystemFont};

fn shorthand(value: &str) -> Shorthand {
    match parse(value).unwrap() {
        Font::Shorthand(s) => s,
        Font::System(_) => panic!("expected a shorthand for {value:?}"),
    }
}

#[test]
fn minimal() {
    let f = shorthand("12px serif");
    assert_eq!(f.style, "normal");
    assert_eq!(f.variant, "normal");
    assert_eq!(f.weight, "normal");
    assert_eq!(f.stretch, "normal");
    assert_eq!(f.size, "12px");
    assert_eq!(f.line_height, LineHeight::Normal);
    assert_eq!(f.family, ["serif"]);
}

#[test]
fn full_shorthand() {
    let f = shorthand("italic small-caps bold condensed 12px/1.2 Arial");
    assert_eq!(f.style, "italic");
    assert_eq!(f.variant, "small-caps");
    assert_eq!(f.weight, "bold");
    assert_eq!(f.stretch, "condensed");
    assert_eq!(f.size, "12px");
    assert_eq!(f.line_height, LineHeight::Number(1.2));
    assert_eq!(f.family, ["Arial"]);
}

#[test]
fn order_independent_prefix() {
    let f = shorthand("small-caps 400 16px/1.2 system-ui");
    assert_eq!(f.variant, "small-caps");
    assert_eq!(f.weight, "400"); // numeric weight not mistaken for a size
    assert_eq!(f.size, "16px");
    assert_eq!(f.family, ["system-ui"]);

    let f = shorthand("condensed bold 12px/14px Arial");
    assert_eq!(f.stretch, "condensed");
    assert_eq!(f.weight, "bold");
    assert_eq!(f.line_height, LineHeight::Other("14px".into())); // has a unit -> string
}

#[test]
fn line_height_kinds() {
    assert_eq!(shorthand("12px/1.5 A").line_height, LineHeight::Number(1.5));
    assert_eq!(shorthand("12px / 2 A").line_height, LineHeight::Number(2.0)); // standalone slash
    assert_eq!(
        shorthand("12px/14px A").line_height,
        LineHeight::Other("14px".into())
    );
    assert_eq!(shorthand("12px/normal A").line_height, LineHeight::Normal);
    assert_eq!(shorthand("12px A").line_height, LineHeight::Normal); // default
}

#[test]
fn size_keywords_and_units() {
    assert_eq!(shorthand("x-large serif").size, "x-large");
    assert_eq!(shorthand("smaller monospace").size, "smaller");
    assert_eq!(shorthand("50% serif").size, "50%");
    assert_eq!(shorthand("1.2em \"Open Sans\", Arial").size, "1.2em");
    assert_eq!(shorthand(".5em serif").size, ".5em");
}

#[test]
fn families_and_quotes() {
    assert_eq!(
        shorthand("1.2em \"Open Sans\", Arial, sans-serif").family,
        ["Open Sans", "Arial", "sans-serif"]
    );
    assert_eq!(
        shorthand("bold 1em \"Times New Roman\"").family,
        ["Times New Roman"]
    );
    assert_eq!(shorthand("12px 'a b', c").family, ["a b", "c"]);
    // a comma inside a quoted family name is not a separator
    assert_eq!(shorthand("12px \"a, b\", c").family, ["a, b", "c"]);
}

// Regression: adversarial-review findings — exact css-list-helpers/unquote parity.
#[test]
fn faithful_edge_splitting() {
    // a backslash-escaped quote inside a family name does not end the string
    assert_eq!(
        shorthand(r#"12px "a\"b", serif"#).family,
        [r#"a\"b"#, "serif"]
    );
    // square brackets are not nesting: a comma inside them still splits
    assert_eq!(shorthand("12px a[1,2], b").family, ["a[1", "2]", "b"]);
    // parentheses ARE nesting: a comma inside them does not split
    assert_eq!(
        shorthand("12px Foo(a, b), serif").family,
        ["Foo(a, b)", "serif"]
    );
    // only space/newline/tab separate tokens — CR stays inside the family name
    assert_eq!(shorthand("12px serif\rmono").family, ["serif\rmono"]);
    // empty comma segments are dropped (a trailing one is kept)
    assert_eq!(shorthand("12px Arial,, serif").family, ["Arial", "serif"]);
    // an unbalanced quote still has its leading quote stripped
    assert_eq!(shorthand("12px \"Arial").family, ["Arial"]);
    assert_eq!(shorthand("12px Arial'").family, ["Arial"]);
}

#[test]
fn whitespace_is_tolerated() {
    let f = shorthand("  12px   serif  ");
    assert_eq!(f.size, "12px");
    assert_eq!(f.family, ["serif"]);
}

#[test]
fn system_fonts() {
    assert_eq!(parse("caption").unwrap(), Font::System(SystemFont::Caption));
    assert_eq!(
        parse("status-bar").unwrap(),
        Font::System(SystemFont::StatusBar)
    );
    assert_eq!(
        parse("message-box").unwrap(),
        Font::System(SystemFont::MessageBox)
    );
    assert_eq!(SystemFont::Caption.as_str(), "caption");
    // case-sensitive: uppercase is not a system keyword
    assert_eq!(parse("MENU"), Err(ParseError::MissingFontSize));
}

#[test]
fn errors() {
    assert_eq!(parse(""), Err(ParseError::EmptyString));
    assert_eq!(parse("12px"), Err(ParseError::MissingFontFamily));
    assert_eq!(parse("italic 12px"), Err(ParseError::MissingFontFamily));
    assert_eq!(parse("bold serif"), Err(ParseError::MissingFontSize));
    assert_eq!(
        parse("italic oblique 12px serif"),
        Err(ParseError::DuplicateStyle)
    );
    assert_eq!(
        parse("bold bolder 12px serif"),
        Err(ParseError::DuplicateWeight)
    );
    assert_eq!(
        parse("condensed expanded 12px serif"),
        Err(ParseError::DuplicateStretch)
    );
}
