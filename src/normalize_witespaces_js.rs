use regex::{Regex};
use once_cell::sync::Lazy;

static KEEP_SPACE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?:\\(?:\r\n|\r|\n))+\s+").unwrap());
static KEEP_NEW_LINE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?:\r\n|\r|\n)+\s+").unwrap());

// JS code for the reference:
// const keepSpaceRe = /(?:\\(?:\r\n|\r|\n))+\s+/g
// const keepNewLineRe = /(?:\r\n|\r|\n)+\s+/g
//
// function normalizeWhitespace(text: string): string {
//   return text
//     .replace(keepSpaceRe, " ")
//     .replace(keepNewLineRe, "\n")
//     .trim()
// }

pub fn normalize_whitespaces_js(str: &str) -> String {
    let str = KEEP_SPACE_RE.replace_all(&str, " ");
    let str = KEEP_NEW_LINE_RE.replace_all(&str, "\n")
        .trim().to_string();

    return str
}

#[cfg(test)]
mod tests {
    use super::{*};

    #[test]
    fn test_normalize_whitespaces_js() {
        assert_eq!(
            normalize_whitespaces_js(
                r#"Multiline
                    string"#
            ),
            "Multiline\nstring")
    }
}