use regex::Regex;
use once_cell::sync::Lazy;
use std::borrow::Cow;
use regex::RegexSet;

static STRIP_AFTER_TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"([>}])((?:\r\n|\r|\n)+\s*)").unwrap());
static STRIP_BEFORE_TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?:\r\n|\r|\n)+\s*?([<{])+").unwrap());

// // replace whitespace before/after newline with single space
// const keepSpaceRe = /\s*(?:\r\n|\r|\n)+\s*/g
// // remove whitespace before/after tag or expression
// const stripAroundTagsRe = /(?:([>}])(?:\r\n|\r|\n)+\s*|(?:\r\n|\r|\n)+\s*(?=[<{]))/g

// function normalizeWhitespace(text) {
//   return (
//     text
//       .replace(stripAroundTagsRe, "$1")
//       .replace(keepSpaceRe, " ")
//       // keep escaped newlines
//       .replace(/\\n/g, "\n")
//       .replace(/\\s/g, " ")
//       // we remove trailing whitespace inside Plural
//       .replace(/(\s+})/gm, "}")
//       .trim()
//   )
// }

pub fn normalize_whitespaces(str: &str) -> String {
    let str = STRIP_AFTER_TAG.replace_all(str, "$1").to_string(); //.trim().to_string()
    STRIP_BEFORE_TAG.replace_all(&str, "$1").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::{*};

    #[test]
    fn test_normalize_whitespaces() {
        assert_eq!(
            normalize_whitespaces(
                r#"
    Hello <strong>World!</strong><br />
    <p>
     My name is <a href="/about">{{" "}}\s
      <em>{{name}}</em></a>
    </p>
    "#
            ),
            r#"Hello <strong>World!</strong><br /><p>My name is <a href="/about">{{" "}}\s<em>{{name}}</em></a></p>"#)
    }
}