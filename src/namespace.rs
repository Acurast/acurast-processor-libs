//! Registry namespaces: groups of endpoints that can be discovered together. A namespace is
//! ASCII, at most [`MAX_NAMESPACE_LEN`] bytes, made of `[a-z0-9._/-]`.

use alloc::{format, string::String};

use crate::consts::MAX_NAMESPACE_LEN;

/// Namespace of the holders of the network secret of `generation`.
pub fn secret_holder(generation: u32) -> String {
	format!("secret-holder/{generation}")
}

/// Whether `ns` is a syntactically valid namespace. Servers additionally reject namespaces
/// they do not know (`UnknownNamespace`).
pub fn is_valid(ns: &str) -> bool {
	!ns.is_empty()
		&& ns.len() <= MAX_NAMESPACE_LEN
		&& ns
			.bytes()
			.all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'.' | b'_' | b'/' | b'-'))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn known_namespaces_are_valid() {
		assert_eq!(secret_holder(3), "secret-holder/3");
		assert!(is_valid(&secret_holder(u32::MAX)));
	}

	#[test]
	fn invalid_namespaces() {
		assert!(!is_valid(""));
		assert!(!is_valid("Upper"));
		assert!(!is_valid("with space"));
		assert!(!is_valid("ünicode"));
		assert!(!is_valid(&"a".repeat(MAX_NAMESPACE_LEN + 1)));
		assert!(is_valid(&"a".repeat(MAX_NAMESPACE_LEN)));
	}
}
