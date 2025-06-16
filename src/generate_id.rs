use data_encoding::BASE64;
use sha2::{Digest, Sha256};

const UNIT_SEPARATOR: &char = &'\u{001F}';

pub fn generate_message_id(message: &str, context: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{message}{UNIT_SEPARATOR}{context}"));

    let result = hasher.finalize();

    BASE64.encode(result.as_ref())[0..6].into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_message_id() {
        assert_eq!(generate_message_id("my message", ""), "vQhkQx")
    }

    #[test]
    fn test_generate_message_id_with_context() {
        assert_eq!(
            generate_message_id("my message", "custom context"),
            "gGUeZH"
        )
    }
}
