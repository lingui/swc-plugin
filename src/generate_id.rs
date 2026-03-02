use data_encoding::{BASE64, BASE64URL};
use sha2::{Digest, Sha256};

const UNIT_SEPARATOR: &char = &'\u{001F}';

pub fn generate_message_id(message: &str, context: &str, use_lingui_v5: bool) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{message}{UNIT_SEPARATOR}{context}"));

    let result = hasher.finalize();

    let encoder = if use_lingui_v5 { BASE64 } else { BASE64URL };
    encoder.encode(result.as_ref())[0..6].into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_message_id() {
        assert_eq!(generate_message_id("my message", "", false), "vQhkQx")
    }

    #[test]
    fn test_generate_message_id_with_context() {
        assert_eq!(
            generate_message_id("my message", "custom context", false),
            "gGUeZH"
        )
    }

    #[test]
    fn test_generate_message_id_url_safe() {
        // this normally should produce `SO/WB8` but with urlsafe result should be different
        assert_eq!(
            generate_message_id("Hello World", "my context", false),
            "SO_WB8"
        )
    }

    #[test]
    fn test_generate_message_id_v5_compatibility() {
        // When using v5 mode (non-url-safe), should produce BASE64 encoding with / and +
        assert_eq!(
            generate_message_id("Hello World", "my context", true),
            "SO/WB8"
        )
    }
}
