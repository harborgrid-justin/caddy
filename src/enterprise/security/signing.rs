// CADDY v0.1.5 - Digital Signatures and Certificate Management
// Document signing, verification, and PKI operations

use crate::enterprise::security::{SecurityError, SecurityResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Digital signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub algorithm: SignatureAlgorithm,
    #[serde(with = "hex")]
    pub signature_bytes: Vec<u8>,
    pub signer: String,
    pub timestamp: i64,
    pub certificate_id: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Signature algorithm
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SignatureAlgorithm {
    RsaSha256,
    RsaSha512,
    EcdsaP256,
    EcdsaP384,
    Ed25519,
}

/// X.509 Certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    pub id: String,
    pub subject: CertificateSubject,
    pub issuer: CertificateSubject,
    pub serial_number: String,
    pub not_before: i64,
    pub not_after: i64,
    #[serde(with = "hex")]
    pub public_key: Vec<u8>,
    pub signature_algorithm: SignatureAlgorithm,
    #[serde(with = "hex")]
    pub signature: Vec<u8>,
    pub extensions: HashMap<String, String>,
    pub is_ca: bool,
}

/// Certificate subject/issuer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateSubject {
    pub common_name: String,
    pub organization: Option<String>,
    pub organizational_unit: Option<String>,
    pub country: Option<String>,
    pub state: Option<String>,
    pub locality: Option<String>,
    pub email: Option<String>,
}

/// Certificate signing request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateRequest {
    pub subject: CertificateSubject,
    pub key_algorithm: SignatureAlgorithm,
    pub key_size: usize,
    pub validity_days: u32,
    pub extensions: HashMap<String, String>,
}

/// Certificate chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateChain {
    pub leaf: Certificate,
    pub intermediates: Vec<Certificate>,
    pub root: Option<Certificate>,
}

/// Timestamp token (RFC 3161)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampToken {
    pub timestamp: i64,
    pub accuracy_micros: u32,
    pub tsa_name: String,
    #[serde(with = "hex")]
    pub digest: Vec<u8>,
    #[serde(with = "hex")]
    pub token_signature: Vec<u8>,
}

/// Signed document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedDocument {
    #[serde(with = "hex")]
    pub document_hash: Vec<u8>,
    pub signatures: Vec<Signature>,
    pub timestamp_tokens: Vec<TimestampToken>,
    pub certificate_chain: Option<CertificateChain>,
    pub metadata: DocumentMetadata,
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub document_id: String,
    pub document_name: String,
    pub document_type: String,
    pub created_at: i64,
    pub signers: Vec<String>,
    pub properties: HashMap<String, String>,
}

/// Signature verification result
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub is_valid: bool,
    pub signer: String,
    pub signed_at: i64,
    pub certificate_valid: bool,
    pub timestamp_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Main signing service
pub struct SigningService {
    certificates: Arc<RwLock<HashMap<String, Certificate>>>,
    private_keys: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    trusted_roots: Arc<RwLock<Vec<Certificate>>>,
    tsa_endpoints: Arc<RwLock<Vec<String>>>,
}

impl SigningService {
    /// Create a new signing service
    pub fn new() -> Self {
        Self {
            certificates: Arc::new(RwLock::new(HashMap::new())),
            private_keys: Arc::new(RwLock::new(HashMap::new())),
            trusted_roots: Arc::new(RwLock::new(Vec::new())),
            tsa_endpoints: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Sign data with a private key
    pub fn sign(
        &self,
        data: &[u8],
        signer_id: &str,
        algorithm: SignatureAlgorithm,
    ) -> SecurityResult<Signature> {
        let private_keys = self.private_keys.read()
            .map_err(|e| SecurityError::Signature(format!("Lock error: {}", e)))?;

        let private_key = private_keys.get(signer_id)
            .ok_or_else(|| SecurityError::Signature(format!("Private key not found: {}", signer_id)))?;

        // Compute signature
        let signature_bytes = self.compute_signature(data, private_key, algorithm)?;

        let certificates = self.certificates.read()
            .map_err(|e| SecurityError::Signature(format!("Lock error: {}", e)))?;

        let certificate_id = certificates.get(signer_id).map(|c| c.id.clone());

        Ok(Signature {
            algorithm,
            signature_bytes,
            signer: signer_id.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            certificate_id,
            metadata: HashMap::new(),
        })
    }

    /// Verify a signature
    pub fn verify(
        &self,
        data: &[u8],
        signature: &Signature,
    ) -> SecurityResult<VerificationResult> {
        let certificates = self.certificates.read()
            .map_err(|e| SecurityError::Signature(format!("Lock error: {}", e)))?;

        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Get signer's certificate
        let certificate = if let Some(cert_id) = &signature.certificate_id {
            certificates.get(cert_id)
        } else {
            certificates.get(&signature.signer)
        };

        let certificate_valid = if let Some(cert) = certificate {
            self.verify_certificate(cert)?
        } else {
            warnings.push("Certificate not found".to_string());
            false
        };

        // Verify signature
        let is_valid = if let Some(cert) = certificate {
            match self.verify_signature(data, &signature.signature_bytes, &cert.public_key, signature.algorithm) {
                Ok(valid) => valid,
                Err(e) => {
                    errors.push(format!("Signature verification failed: {}", e));
                    false
                }
            }
        } else {
            errors.push("Cannot verify without certificate".to_string());
            false
        };

        Ok(VerificationResult {
            is_valid,
            signer: signature.signer.clone(),
            signed_at: signature.timestamp,
            certificate_valid,
            timestamp_valid: true, // Would verify timestamp token
            errors,
            warnings,
        })
    }

    /// Sign a document
    pub fn sign_document(
        &self,
        document: &[u8],
        signer_id: &str,
        algorithm: SignatureAlgorithm,
        metadata: DocumentMetadata,
    ) -> SecurityResult<SignedDocument> {
        // Compute document hash
        let document_hash = self.hash_sha256(document);

        // Sign the hash
        let signature = self.sign(&document_hash, signer_id, algorithm)?;

        // Get timestamp token
        let timestamp_token = self.get_timestamp_token(&document_hash)?;

        // Get certificate chain
        let certificate_chain = self.get_certificate_chain(signer_id)?;

        Ok(SignedDocument {
            document_hash,
            signatures: vec![signature],
            timestamp_tokens: vec![timestamp_token],
            certificate_chain: Some(certificate_chain),
            metadata,
        })
    }

    /// Verify a signed document
    pub fn verify_document(
        &self,
        document: &[u8],
        signed_doc: &SignedDocument,
    ) -> SecurityResult<Vec<VerificationResult>> {
        // Verify document hash
        let computed_hash = self.hash_sha256(document);
        if computed_hash != signed_doc.document_hash {
            return Err(SecurityError::Integrity("Document hash mismatch".to_string()));
        }

        // Verify all signatures
        let mut results = Vec::new();
        for signature in &signed_doc.signatures {
            let result = self.verify(&signed_doc.document_hash, signature)?;
            results.push(result);
        }

        // Verify certificate chain
        if let Some(ref chain) = signed_doc.certificate_chain {
            self.verify_certificate_chain(chain)?;
        }

        // Verify timestamps
        for token in &signed_doc.timestamp_tokens {
            if token.digest != signed_doc.document_hash {
                return Err(SecurityError::Signature("Timestamp digest mismatch".to_string()));
            }
        }

        Ok(results)
    }

    /// Generate a self-signed certificate
    pub fn generate_self_signed_certificate(
        &self,
        request: CertificateRequest,
    ) -> SecurityResult<(Certificate, Vec<u8>)> {
        // Generate key pair
        let (public_key, private_key) = self.generate_keypair(request.key_algorithm, request.key_size)?;

        let now = chrono::Utc::now().timestamp();
        let not_after = now + (request.validity_days as i64 * 86400);

        let certificate = Certificate {
            id: self.generate_certificate_id(),
            subject: request.subject.clone(),
            issuer: request.subject.clone(), // Self-signed
            serial_number: self.generate_serial_number(),
            not_before: now,
            not_after,
            public_key: public_key.clone(),
            signature_algorithm: request.key_algorithm,
            signature: Vec::new(), // Will be computed
            extensions: request.extensions,
            is_ca: false,
        };

        // Sign the certificate
        let cert_data = self.encode_certificate_data(&certificate)?;
        let signature = self.compute_signature(&cert_data, &private_key, request.key_algorithm)?;

        let mut signed_cert = certificate;
        signed_cert.signature = signature;

        // Store certificate and private key
        let cert_id = signed_cert.id.clone();

        let mut certs = self.certificates.write()
            .map_err(|e| SecurityError::Certificate(format!("Lock error: {}", e)))?;
        certs.insert(cert_id.clone(), signed_cert.clone());
        drop(certs);

        let mut keys = self.private_keys.write()
            .map_err(|e| SecurityError::Certificate(format!("Lock error: {}", e)))?;
        keys.insert(cert_id, private_key.clone());

        Ok((signed_cert, private_key))
    }

    /// Import a certificate
    pub fn import_certificate(&self, certificate: Certificate) -> SecurityResult<()> {
        let mut certs = self.certificates.write()
            .map_err(|e| SecurityError::Certificate(format!("Lock error: {}", e)))?;

        certs.insert(certificate.id.clone(), certificate);
        Ok(())
    }

    /// Import a private key
    pub fn import_private_key(&self, cert_id: &str, private_key: Vec<u8>) -> SecurityResult<()> {
        let mut keys = self.private_keys.write()
            .map_err(|e| SecurityError::Certificate(format!("Lock error: {}", e)))?;

        keys.insert(cert_id.to_string(), private_key);
        Ok(())
    }

    /// Add trusted root certificate
    pub fn add_trusted_root(&self, certificate: Certificate) -> SecurityResult<()> {
        if !certificate.is_ca {
            return Err(SecurityError::Certificate("Not a CA certificate".to_string()));
        }

        let mut roots = self.trusted_roots.write()
            .map_err(|e| SecurityError::Certificate(format!("Lock error: {}", e)))?;

        roots.push(certificate);
        Ok(())
    }

    /// Verify certificate validity
    pub fn verify_certificate(&self, certificate: &Certificate) -> SecurityResult<bool> {
        let now = chrono::Utc::now().timestamp();

        // Check validity period
        if now < certificate.not_before || now > certificate.not_after {
            return Ok(false);
        }

        // For self-signed certificates, verify signature with own public key
        if certificate.subject.common_name == certificate.issuer.common_name {
            let cert_data = self.encode_certificate_data(certificate)?;
            return self.verify_signature(
                &cert_data,
                &certificate.signature,
                &certificate.public_key,
                certificate.signature_algorithm,
            );
        }

        // Would verify against issuer's certificate
        Ok(true)
    }

    /// Get certificate chain
    pub fn get_certificate_chain(&self, cert_id: &str) -> SecurityResult<CertificateChain> {
        let certs = self.certificates.read()
            .map_err(|e| SecurityError::Certificate(format!("Lock error: {}", e)))?;

        let leaf = certs.get(cert_id)
            .ok_or_else(|| SecurityError::Certificate(format!("Certificate not found: {}", cert_id)))?
            .clone();

        // For now, return just the leaf certificate
        // In production, would build full chain
        Ok(CertificateChain {
            leaf,
            intermediates: Vec::new(),
            root: None,
        })
    }

    /// Verify certificate chain
    pub fn verify_certificate_chain(&self, chain: &CertificateChain) -> SecurityResult<bool> {
        // Verify leaf certificate
        if !self.verify_certificate(&chain.leaf)? {
            return Ok(false);
        }

        // Verify intermediate certificates
        for cert in &chain.intermediates {
            if !self.verify_certificate(cert)? {
                return Ok(false);
            }
        }

        // Verify root if present
        if let Some(ref root) = chain.root {
            if !self.verify_certificate(root)? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Configure timestamp authority
    pub fn add_tsa_endpoint(&self, endpoint: String) -> SecurityResult<()> {
        let mut endpoints = self.tsa_endpoints.write()
            .map_err(|e| SecurityError::Signature(format!("Lock error: {}", e)))?;

        endpoints.push(endpoint);
        Ok(())
    }

    // Helper methods

    fn compute_signature(
        &self,
        data: &[u8],
        private_key: &[u8],
        algorithm: SignatureAlgorithm,
    ) -> SecurityResult<Vec<u8>> {
        // Simulate signature computation
        // In production, use ring, rsa, or ed25519-dalek crates
        let hash = self.hash_for_algorithm(data, algorithm)?;

        let mut signature = vec![0u8; self.signature_size(algorithm)];
        for i in 0..signature.len() {
            signature[i] = hash[i % hash.len()] ^ private_key[i % private_key.len()];
        }

        Ok(signature)
    }

    fn verify_signature(
        &self,
        data: &[u8],
        signature: &[u8],
        public_key: &[u8],
        algorithm: SignatureAlgorithm,
    ) -> SecurityResult<bool> {
        // Simulate signature verification
        let hash = self.hash_for_algorithm(data, algorithm)?;

        let mut expected = vec![0u8; signature.len()];
        for i in 0..expected.len() {
            expected[i] = hash[i % hash.len()] ^ public_key[i % public_key.len()];
        }

        Ok(signature == expected.as_slice())
    }

    fn generate_keypair(
        &self,
        algorithm: SignatureAlgorithm,
        key_size: usize,
    ) -> SecurityResult<(Vec<u8>, Vec<u8>)> {
        // Simulate key pair generation
        let mut public_key = vec![0u8; key_size / 8];
        let mut private_key = vec![0u8; key_size / 4];

        self.fill_random(&mut public_key)?;
        self.fill_random(&mut private_key)?;

        Ok((public_key, private_key))
    }

    fn hash_sha256(&self, data: &[u8]) -> Vec<u8> {
        // Simulate SHA-256 hash
        let mut hash = vec![0u8; 32];
        for (i, byte) in data.iter().enumerate() {
            hash[i % 32] = hash[i % 32].wrapping_add(*byte).wrapping_mul(251);
        }
        hash
    }

    fn hash_for_algorithm(&self, data: &[u8], algorithm: SignatureAlgorithm) -> SecurityResult<Vec<u8>> {
        match algorithm {
            SignatureAlgorithm::RsaSha256 | SignatureAlgorithm::EcdsaP256 | SignatureAlgorithm::Ed25519 => {
                Ok(self.hash_sha256(data))
            }
            SignatureAlgorithm::RsaSha512 | SignatureAlgorithm::EcdsaP384 => {
                // Simulate SHA-512
                let mut hash = vec![0u8; 64];
                for (i, byte) in data.iter().enumerate() {
                    hash[i % 64] = hash[i % 64].wrapping_add(*byte).wrapping_mul(251);
                }
                Ok(hash)
            }
        }
    }

    fn signature_size(&self, algorithm: SignatureAlgorithm) -> usize {
        match algorithm {
            SignatureAlgorithm::RsaSha256 | SignatureAlgorithm::RsaSha512 => 256,
            SignatureAlgorithm::EcdsaP256 => 64,
            SignatureAlgorithm::EcdsaP384 => 96,
            SignatureAlgorithm::Ed25519 => 64,
        }
    }

    fn encode_certificate_data(&self, cert: &Certificate) -> SecurityResult<Vec<u8>> {
        // Simulate DER encoding
        let mut data = Vec::new();
        data.extend_from_slice(cert.subject.common_name.as_bytes());
        data.extend_from_slice(cert.issuer.common_name.as_bytes());
        data.extend_from_slice(&cert.serial_number.as_bytes());
        data.extend_from_slice(&cert.public_key);
        Ok(data)
    }

    fn generate_certificate_id(&self) -> String {
        format!("cert_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0))
    }

    fn generate_serial_number(&self) -> String {
        format!("{:016x}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0))
    }

    fn get_timestamp_token(&self, digest: &[u8]) -> SecurityResult<TimestampToken> {
        // Simulate TSA timestamp token
        let token_data = [digest, b"tsa_signature"].concat();
        let token_signature = self.hash_sha256(&token_data);

        Ok(TimestampToken {
            timestamp: chrono::Utc::now().timestamp(),
            accuracy_micros: 1000,
            tsa_name: "CADDY TSA".to_string(),
            digest: digest.to_vec(),
            token_signature,
        })
    }

    fn fill_random(&self, buf: &mut [u8]) -> SecurityResult<()> {
        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hash, Hasher};

        let rs = RandomState::new();
        for (i, byte) in buf.iter_mut().enumerate() {
            let mut hasher = rs.build_hasher();
            (i, std::time::SystemTime::now()).hash(&mut hasher);
            *byte = (hasher.finish() & 0xFF) as u8;
        }
        Ok(())
    }
}

impl Default for SigningService {
    fn default() -> Self {
        Self::new()
    }
}

mod hex {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex_encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        hex_decode(&s).map_err(serde::de::Error::custom)
    }

    fn hex_encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
        if s.len() % 2 != 0 {
            return Err("Odd hex string length".to_string());
        }
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify() {
        let service = SigningService::new();

        let request = CertificateRequest {
            subject: CertificateSubject {
                common_name: "Test User".to_string(),
                organization: Some("CADDY".to_string()),
                organizational_unit: None,
                country: Some("US".to_string()),
                state: None,
                locality: None,
                email: Some("test@caddy.local".to_string()),
            },
            key_algorithm: SignatureAlgorithm::RsaSha256,
            key_size: 2048,
            validity_days: 365,
            extensions: HashMap::new(),
        };

        let (cert, _) = service.generate_self_signed_certificate(request).unwrap();

        let data = b"Test document";
        let signature = service.sign(data, &cert.id, SignatureAlgorithm::RsaSha256).unwrap();
        let result = service.verify(data, &signature).unwrap();

        assert!(result.is_valid);
    }
}
