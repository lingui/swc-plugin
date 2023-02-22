use sha2::{Sha256, Digest};
use data_encoding::BASE64;

const UNIT_SEPARATOR: &char = &'\u{001F}';

pub fn generate_message_id(message: &str, context: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(format!("{message}{UNIT_SEPARATOR}{context}"));

  let result = hasher.finalize();
  return BASE64.encode(result.as_ref())[0..6].into()
}

#[cfg(test)]
mod tests {
  use super::{*};

  #[test]
  fn test_generate_message_id() {
    assert_eq!(
      generate_message_id("my message", ""), "vQhkQx")
  }

  #[test]
  fn test_generate_message_id_with_context() {
    assert_eq!(
      generate_message_id("my message", "custom context"), "gGUeZH")
  }
}
