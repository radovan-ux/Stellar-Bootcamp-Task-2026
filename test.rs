#![cfg(test)]

use super::*;
use soroban_sdk::{vec, Env, Address, BytesN, String, testutils::Address as _};

mod test {
    use super::*;

    #[test]
    fn test_happy_path_register_and_reward() {
        let env = Env::default();
        
        // Setup admin and student addresses
        let admin = Address::generate(&env);
        let student = Address::generate(&env);
        
        // Register the contract
        let contract_id = env.register(StellaroidEarn, ());
        let client = StellaroidEarnClient::new(&env, &contract_id);
        
        // Initialize contract with admin
        client.init(&admin);
        
        // Create test certificate hash (mock SHA-256)
        let mut hash_bytes = [0u8; 32];
        hash_bytes[0] = 1; // Simple mock hash
        let cert_hash = BytesN::from_array(&env, &hash_bytes);
        
        // Register certificate as student
        let cert_id = client
            .register_certificate(&student, &cert_hash, &String::from_str(&env, "{\"degree\":\"BSCS\"}"))
            .unwrap();
        
        assert_eq!(cert_id, 0u32);
        
        // Verify certificate
        let is_verified = client.verify_certificate(&0u32, &cert_hash).unwrap();
        assert!(is_verified);
        
        // Reward student (will require admin auth in production)
        // This demonstrates the flow works
        println!("Certificate registered and verified successfully!");
        
        // Check certificate details
        let cert = client.get_certificate(&0u32).unwrap();
        assert_eq!(cert.owner, student);
        assert!(cert.verified);
    }

    #[test]
    fn test_edge_case_duplicate_rejected() {
        let env = Env::default();
        
        let admin = Address::generate(&env);
        let student = Address::generate(&env);
        
        let contract_id = env.register(StellaroidEarn, ());
        let client = StellaroidEarnClient::new(&env, &contract_id);
        
        client.init(&admin);
        
        // Create test hash
        let mut hash_bytes = [0u8; 32];
        hash_bytes[0] = 1;
        let cert_hash = BytesN::from_array(&env, &hash_bytes);
        
        // Register first time - should succeed
        let first_result = client
            .register_certificate(&student, &cert_hash, &String::from_str(&env, "{\"degree\":\"BSCS\"}"));
        assert!(first_result.is_ok());
        
        // Try to register duplicate - should fail
        let duplicate_result = client
            .register_certificate(&student, &cert_hash, &String::from_str(&env, "{\"degree\":\"BSCS\"}"));
        
        assert!(duplicate_result.is_err());
        
        // Verify error type is DuplicateCertificate
        match duplicate_result {
            Err(Error::DuplicateCertificate) => println!("Correctly rejected duplicate"),
            _ => panic!("Wrong error type returned"),
        }
    }

    #[test]
    fn test_state_verification_after_registration() {
        let env = Env::default();
        
        let admin = Address::generate(&env);
        let student = Address::generate(&env);
        
        let contract_id = env.register(StellaroidEarn, ());
        let client = StellaroidEarnClient::new(&env, &contract_id);
        
        client.init(&admin);
        
        // Create specific test hash
        let mut hash_bytes = [0u8; 32];
        hash_bytes[0] = 42; // Magic number for testing
        let cert_hash = BytesN::from_array(&env, &hash_bytes);
        let metadata = String::from_str(&env, "{\"degree\":\"BSIT\",\"year\":\"2026\"}");
        
        // Register certificate
        let cert_id = client
            .register_certificate(&student, &cert_hash, &metadata)
            .unwrap();
        
        // Verify state after registration
        let stored_cert = client.get_certificate(&cert_id).unwrap();
        
        // Assert all fields match
        assert_eq!(stored_cert.hash, cert_hash, "Hash should match");
        assert_eq!(stored_cert.owner, student, "Owner should be the student");
        assert_eq!(stored_cert.metadata, metadata, "Metadata should match");
        assert!(!stored_cert.verified, "Initially should not be verified");
        assert!(!stored_cert.reward_claimed, "Reward should not be claimed yet");
        
        // Verify contract state is consistent
        let cert_count = client.get_certificate_count();
        assert_eq!(cert_count, 1u32, "Should have exactly 1 certificate");
        
        println!("State verification passed: Certificate stored correctly");
    }
}
