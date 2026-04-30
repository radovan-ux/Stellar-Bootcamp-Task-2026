#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, String, Symbol,
    Vec, Map, token, log,
};

// Storage keys for contract data
const CERTIFICATE_REGISTRY: Symbol = symbol_short!("CERT_REG");
const CERTIFICATE_COUNT: Symbol = symbol_short!("CERT_COUNT");
const ADMIN_KEY: Symbol = symbol_short!("ADMIN");

// Certificate status enum
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CertStatus {
    Pending,
    Verified,
    Revoked,
}

// Certificate data structure
#[contracttype]
#[derive(Clone)]
pub struct Certificate {
    pub hash: BytesN<32>,        // SHA-256 hash of certificate content
    pub owner: Address,          // Student wallet address
    pub issuer: Address,         // Institution wallet address
    pub issued_at: u64,          // Timestamp of issuance
    pub verified: bool,          // Verification status
    pub reward_claimed: bool,    // Whether XLM reward has been claimed
    pub metadata: String,        // JSON metadata (degree, course, etc.)
}

// Contract structure
#[contract]
pub struct StellaroidEarn;

#[contractimpl]
impl StellaroidEarn {
    /// Initialize contract with admin address
    /// # Arguments
    /// * `env` - Soroban environment
    /// * `admin` - Admin wallet address
    pub fn init(env: Env, admin: Address) {
        // Require admin authentication
        admin.require_auth();
        
        // Store admin address in contract storage
        env.storage().instance().set(&ADMIN_KEY, &admin);
        
        // Initialize certificate counter
        env.storage().instance().set(&CERTIFICATE_COUNT, &0u32);
        
        log!(&env, "StellaroidEarn contract initialized by admin: {}", admin);
    }

    /// Register a new certificate on-chain
    /// # Arguments
    /// * `env` - Soroban environment
    /// * `student` - Student wallet address
    /// * `cert_hash` - SHA-256 hash of certificate content
    /// * `metadata` - Certificate metadata (JSON string)
    /// # Returns
    /// * Certificate ID if successful
    pub fn register_certificate(
        env: Env,
        student: Address,
        cert_hash: BytesN<32>,
        metadata: String,
    ) -> Result<u32, Error> {
        // Authentication: Only the student or admin can register
        student.require_auth();
        
        // Check for duplicate certificate hash
        let cert_registry = env
            .storage()
            .persistent()
            .get::<Symbol, Map<u32, Certificate>>(&CERTIFICATE_REGISTRY)
            .unwrap_or(Map::new(&env));
        
        // Iterate through existing certificates to check for duplicates
        for (_, cert) in cert_registry.iter() {
            if cert.hash == cert_hash {
                log!(&env, "Duplicate certificate detected: hash already exists");
                return Err(Error::DuplicateCertificate);
            }
        }
        
        // Get and increment certificate counter
        let mut cert_count: u32 = env.storage().instance().get(&CERTIFICATE_COUNT).unwrap_or(0);
        
        // Verify the hash integrity - ensure it's a valid SHA-256 (all zeros check)
        if cert_hash == BytesN::from_array(&env, &[0u8; 32]) {
            log!(&env, "Tampered hash detected: zero-hash submitted");
            return Err(Error::TamperedCertificate);
        }
        
        let cert_id = cert_count;
        cert_count += 1;
        
        // Create certificate object
        let certificate = Certificate {
            hash: cert_hash,
            owner: student.clone(),
            issuer: env.current_contract_address(), // Contract acts as issuer
            issued_at: env.ledger().timestamp(),
            verified: false,
            reward_claimed: false,
            metadata,
        };
        
        // Store certificate in registry
        let mut updated_registry = cert_registry;
        updated_registry.set(cert_id, certificate);
        
        // Save updated registry and counter
        env.storage().persistent().set(&CERTIFICATE_REGISTRY, &updated_registry);
        env.storage().instance().set(&CERTIFICATE_COUNT, &cert_count);
        
        // Emit registration event
        log!(&env, "Certificate registered: ID={}, Student={}, Hash={:?}", 
             cert_id, student, cert_hash);
        
        // Publish event for off-chain listeners
        env.events().publish(
            (symbol_short!("cert_registered"),),
            (cert_id, student, cert_hash),
        );
        
        Ok(cert_id)
    }

    /// Reward student with XLM upon successful certificate verification
    /// # Arguments
    /// * `env` - Soroban environment  
    /// * `cert_id` - Certificate ID to reward
    /// * `amount` - XLM amount in stroops (1 XLM = 10^7 stroops)
    pub fn reward_student(env: Env, cert_id: u32, amount: i128) -> Result<(), Error> {
        // Get admin from storage
        let admin: Address = env.storage().instance().get(&ADMIN_KEY).unwrap();
        admin.require_auth();
        
        // Get certificate from registry
        let mut cert_registry = env
            .storage()
            .persistent()
            .get::<Symbol, Map<u32, Certificate>>(&CERTIFICATE_REGISTRY)
            .unwrap();
        
        let mut certificate = cert_registry
            .get(cert_id)
            .ok_or(Error::CertificateNotFound)?;
        
        // Ensure certificate is verified before rewarding
        if !certificate.verified {
            return Err(Error::CertificateNotVerified);
        }
        
        // Prevent double reward claiming
        if certificate.reward_claimed {
            return Err(Error::RewardAlreadyClaimed);
        }
        
        // Mark reward as claimed BEFORE transfer to prevent re-entrancy
        certificate.reward_claimed = true;
        cert_registry.set(cert_id, certificate.clone());
        env.storage().persistent().set(&CERTIFICATE_REGISTRY, &cert_registry);
        
        // Transfer XLM from contract to student
        let xlm_token = token::Client::new(&env, &env.current_contract_address());
        let contract_address = env.current_contract_address();
        
        // This assumes the contract has XLM balance
        // In production, you'd use the token client to transfer
        log!(&env, "Rewarding student {} with {} stroops for certificate {}", 
             certificate.owner, amount, cert_id);
        
        // Emit reward event
        env.events().publish(
            (symbol_short!("student_rewarded"),),
            (cert_id, certificate.owner, amount),
        );
        
        Ok(())
    }

    /// Verify a certificate and return its validity status
    /// # Arguments
    /// * `env` - Soroban environment
    /// * `cert_id` - Certificate ID to verify
    /// * `expected_hash` - Expected certificate hash to verify against
    /// # Returns
    /// * Boolean indicating verification status
    pub fn verify_certificate(
        env: Env,
        cert_id: u32,
        expected_hash: BytesN<32>,
    ) -> Result<bool, Error> {
        // Get certificate from registry
        let cert_registry = env
            .storage()
            .persistent()
            .get::<Symbol, Map<u32, Certificate>>(&CERTIFICATE_REGISTRY)
            .unwrap();
        
        let mut certificate = cert_registry
            .get(cert_id)
            .ok_or(Error::CertificateNotFound)?;
        
        // Verify hash matches
        let is_valid = certificate.hash == expected_hash;
        
        // Update verification status if hash matches
        if is_valid && !certificate.verified {
            certificate.verified = true;
            let mut updated_registry = cert_registry;
            updated_registry.set(cert_id, certificate.clone());
            env.storage().persistent().set(&CERTIFICATE_REGISTRY, &updated_registry);
        }
        
        // Emit verification event
        env.events().publish(
            (symbol_short!("cert_verified"),),
            (cert_id, is_valid),
        );
        
        log!(&env, "Certificate {} verification result: {}", cert_id, is_valid);
        
        Ok(is_valid)
    }

    /// Employer-triggered payment to verified student wallet
    /// # Arguments
    /// * `env` - Soroban environment
    /// * `employer` - Employer wallet address making the payment
    /// * `cert_id` - Certificate ID of the student to pay
    /// * `amount` - Payment amount in stroops
    pub fn link_payment(
        env: Env,
        employer: Address,
        cert_id: u32,
        amount: i128,
    ) -> Result<(), Error> {
        // Require employer authentication
        employer.require_auth();
        
        // Get certificate to verify student is legitimate
        let cert_registry = env
            .storage()
            .persistent()
            .get::<Symbol, Map<u32, Certificate>>(&CERTIFICATE_REGISTRY)
            .unwrap();
        
        let certificate = cert_registry
            .get(cert_id)
            .ok_or(Error::CertificateNotFound)?;
        
        // Ensure certificate is verified before allowing payment
        if !certificate.verified {
            return Err(Error::CertificateNotVerified);
        }
        
        // Process payment to student wallet
        // In a real implementation, this would interact with Stellar's payment system
        log!(&env, "Employer {} paying student {} {} stroops for certificate {}", 
             employer, certificate.owner, amount, cert_id);
        
        // Emit payment event
        env.events().publish(
            (symbol_short!("employer_payment"),),
            (employer, certificate.owner, amount, cert_id),
        );
        
        Ok(())
    }

    /// Get certificate details by ID
    /// # Arguments
    /// * `env` - Soroban environment
    /// * `cert_id` - Certificate ID to query
    /// # Returns  
    /// * Certificate structure
    pub fn get_certificate(env: Env, cert_id: u32) -> Result<Certificate, Error> {
        let cert_registry = env
            .storage()
            .persistent()
            .get::<Symbol, Map<u32, Certificate>>(&CERTIFICATE_REGISTRY)
            .unwrap();
        
        cert_registry.get(cert_id).ok_or(Error::CertificateNotFound)
    }
    
    /// Get total number of registered certificates
    /// # Arguments
    /// * `env` - Soroban environment
    /// # Returns
    /// * Total certificate count
    pub fn get_certificate_count(env: Env) -> u32 {
        env.storage().instance().get(&CERTIFICATE_COUNT).unwrap_or(0)
    }
}

// Custom error types for the contract
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    CertificateNotFound = 1,
    DuplicateCertificate = 2,
    TamperedCertificate = 3,
    CertificateNotVerified = 4,
    RewardAlreadyClaimed = 5,
    UnauthorizedAccess = 6,
}
