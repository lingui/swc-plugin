use regex::{Regex};
use once_cell::sync::Lazy;

// replace whitespace before/after newline with single space
static KEEP_SPACE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*(?:\r\n|\r|\n)+\s*").unwrap());
// remove whitespace before/after tag or expression
static STRIP_AROUND_TAGS: Lazy<Regex> = Lazy::new(|| Regex::new(r"([>}])(?:\r\n|\r|\n)+\s*|(?:\r\n|\r|\n)+\s*([<{])").unwrap());

static TRAILING_IN_EXPRESSIONS: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\s+})").unwrap());

// JS code for the reference:
// // replace whitespace before/after newline with single space
// const keepSpaceRe = /\s*(?:\r\n|\r|\n)+\s*/g
// // remove whitespace before/after tag or expression
// const stripAroundTagsRe = /(?:([>}])(?:\r\n|\r|\n)+\s*|(?:\r\n|\r|\n)+\s*(?=[<{]))/g
//
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
    let str = STRIP_AROUND_TAGS.replace_all(&str, "$1$2");
    let str = KEEP_SPACE_RE.replace_all(&str, " ");

    // we remove trailing whitespace inside Plural
    let str = TRAILING_IN_EXPRESSIONS.replace_all(&str, "}").trim().to_string();

    return str
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

    #[test]
    fn test_normalize_whitespaces2() {
        assert_eq!(
            normalize_whitespaces(
                r#"
          Property {0},
          function {1},
          array {2},
          constant {3},
          object {4},
          everything {5}
    "#
            ),
            r#"Property {0}, function {1}, array {2}, constant {3}, object {4}, everything {5}"#)
    }

    #[test]
    fn remove_trailing_in_icu() {
        assert_eq!(
            normalize_whitespaces(
                r#"{count, plural, one {

              <0>#</0> slot added

            } other {

              <1>#</1> slots added

            }}
"#
            ),
            r#"{count, plural, one {<0>#</0> slot added} other {<1>#</1> slots added}}"#)
    }
}