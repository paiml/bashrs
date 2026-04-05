#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // =========================================================================
    // EXTREME TDD Tests for Signature Verification (#108)
    // =========================================================================

    #[test]
    fn test_SIGNATURE_108_keyring_create() {
        let keyring = Keyring::new();
        assert!(keyring.is_empty());
        assert!(!keyring.is_tofu_enabled());
    }

    #[test]
    fn test_SIGNATURE_108_keyring_add_key() {
        let mut keyring = Keyring::new();
        let key = TrustedKey::new("test-key", [1u8; 32]);

        keyring.add_key(key).unwrap();
        assert_eq!(keyring.len(), 1);
        assert!(keyring.get_key("test-key").is_some());
    }

    #[test]
    fn test_SIGNATURE_108_keyring_duplicate_key_fails() {
        let mut keyring = Keyring::new();
        let key1 = TrustedKey::new("test-key", [1u8; 32]);
        let key2 = TrustedKey::new("test-key", [2u8; 32]);

        keyring.add_key(key1).unwrap();
        let result = keyring.add_key(key2);
        assert!(result.is_err());
    }

    #[test]
    fn test_SIGNATURE_108_keyring_remove_key() {
        let mut keyring = Keyring::new();
        let key = TrustedKey::new("test-key", [1u8; 32]);

        keyring.add_key(key).unwrap();
        assert!(keyring.remove_key("test-key").unwrap());
        assert!(keyring.is_empty());
    }

    #[test]
    fn test_SIGNATURE_108_tofu_first_use() {
        let mut keyring = Keyring::new();
        keyring.enable_tofu();

        let result = keyring.trust_on_first_use("new-key", [1u8; 32]).unwrap();
        assert_eq!(result, TrustDecision::NewlyTrusted);

        // Second time should be AlreadyTrusted
        let result = keyring.trust_on_first_use("new-key", [1u8; 32]).unwrap();
        assert_eq!(result, TrustDecision::AlreadyTrusted);
    }

    #[test]
    fn test_SIGNATURE_108_tofu_key_changed_error() {
        let mut keyring = Keyring::new();
        keyring.enable_tofu();

        keyring.trust_on_first_use("test-key", [1u8; 32]).unwrap();

        // Different key should fail
        let result = keyring.trust_on_first_use("test-key", [2u8; 32]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("changed"));
    }

    #[test]
    fn test_SIGNATURE_108_tofu_disabled_error() {
        let mut keyring = Keyring::new();
        // TOFU not enabled

        let result = keyring.trust_on_first_use("test-key", [1u8; 32]);
        assert!(result.is_err());
    }

    #[test]
    fn test_SIGNATURE_108_keyring_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let keyring_path = temp_dir.path().join("keyring.json");

        // Create and save
        {
            let mut keyring = Keyring::with_storage(&keyring_path).unwrap();
            keyring.enable_tofu();
            keyring
                .add_key(TrustedKey::new("key-1", [1u8; 32]))
                .unwrap();
            keyring
                .add_key(TrustedKey::new("key-2", [2u8; 32]))
                .unwrap();
        }

        // Load and verify
        {
            let keyring = Keyring::with_storage(&keyring_path).unwrap();
            assert_eq!(keyring.len(), 2);
            assert!(keyring.is_tofu_enabled());
            assert!(keyring.get_key("key-1").is_some());
            assert!(keyring.get_key("key-2").is_some());
        }
    }

    #[test]
    fn test_SIGNATURE_108_sha256_verify() {
        let content = b"Hello, World!";
        let hash = compute_sha256(content);

        assert!(verify_sha256(content, &hash));
        assert!(!verify_sha256(b"Different content", &hash));
    }

    #[test]
    fn test_SIGNATURE_108_signature_verify() {
        let content = b"Test content";
        let content_hash = compute_sha256(content);
        let public_key = [0x42u8; 32];

        let signature = create_test_signature(&content_hash, &public_key);
        assert!(verify_signature(&content_hash, &signature, &public_key));

        // Wrong key should fail
        let wrong_key = [0x43u8; 32];
        assert!(!verify_signature(&content_hash, &signature, &wrong_key));
    }

    #[test]
    fn test_SIGNATURE_108_zero_signature_invalid() {
        let content_hash = [0u8; 32];
        let signature = [0u8; 64];
        let public_key = [1u8; 32];

        assert!(!verify_signature(&content_hash, &signature, &public_key));
    }

    #[test]
    fn test_SIGNATURE_108_key_fingerprint() {
        let key = TrustedKey::new(
            "test",
            [
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00,
            ],
        );

        assert_eq!(key.fingerprint(), "0102030405060708");
    }

    #[test]
    fn test_SIGNATURE_108_verification_result() {
        let valid = VerificationResult::Valid {
            signer: "test".to_string(),
            content_hash: "abc".to_string(),
        };
        assert!(valid.is_valid());

        let invalid = VerificationResult::Invalid {
            reason: "bad".to_string(),
        };
        assert!(!invalid.is_valid());

        let unsigned = VerificationResult::Unsigned;
        assert!(!unsigned.is_valid());
    }
}
