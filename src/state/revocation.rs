use std::time::SystemTime;
use serde::{Serialize, Deserialize};
use dashmap::DashSet;

use super::{
    proof::{ProofType, StateProof},
    error::{ProofError, ProofErrorCategory, ErrorSeverity},
};

/// Reason for proof revocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RevocationReason {
    /// Security vulnerability found
    SecurityVulnerability {
        severity: String,
        details: String,
    },
    /// Chain reorganization
    ChainReorg {
        old_block: u64,
        new_block: u64,
    },
    /// Proof algorithm deprecated
    AlgorithmDeprecated {
        algorithm: String,
        replacement: Option<String>,
    },
    /// Manual revocation
    Manual {
        reason: String,
        revoked_by: String,
    },
}

/// Record of a proof revocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevocationRecord {
    /// When the proof was revoked
    pub revoked_at: SystemTime,
    /// Why the proof was revoked
    pub reason: RevocationReason,
    /// Affected proof types
    pub affected_types: Vec<ProofType>,
    /// Whether dependent proofs should also be revoked
    pub cascade: bool,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Registry for managing proof revocations
pub struct RevocationRegistry {
    /// Set of revoked proof hashes
    revoked_proofs: DashSet<String>,
    /// History of revocations
    revocation_history: Vec<RevocationRecord>,
}

impl RevocationRegistry {
    /// Create new revocation registry
    pub fn new() -> Self {
        Self {
            revoked_proofs: DashSet::new(),
            revocation_history: Vec::new(),
        }
    }

    /// Revoke a proof
    pub fn revoke_proof(
        &mut self,
        proof: &StateProof,
        reason: RevocationReason,
        cascade: bool,
    ) -> Result<(), ProofError> {
        let proof_hash = format!("{:?}:{:?}", proof.transition, proof.proof);
        
        // Create revocation record
        let record = RevocationRecord {
            revoked_at: SystemTime::now(),
            reason: reason.clone(),
            affected_types: vec![proof.proof_type().clone()],
            cascade,
            metadata: None,
        };

        // Add to registry
        self.revoked_proofs.insert(proof_hash);
        self.revocation_history.push(record);

        Ok(())
    }

    /// Check if a proof is revoked
    pub fn is_revoked(&self, proof: &StateProof) -> bool {
        let proof_hash = format!("{:?}:{:?}", proof.transition, proof.proof);
        self.revoked_proofs.contains(&proof_hash)
    }

    /// Get revocation record for a proof
    pub fn get_revocation(&self, proof: &StateProof) -> Option<RevocationRecord> {
        let proof_hash = format!("{:?}:{:?}", proof.transition, proof.proof);
        if self.revoked_proofs.contains(&proof_hash) {
            self.revocation_history
                .iter()
                .find(|r| r.affected_types.contains(proof.proof_type()))
                .cloned()
        } else {
            None
        }
    }

    /// Revoke all proofs of a specific type
    pub fn revoke_proof_type(
        &mut self,
        proof_type: ProofType,
        reason: RevocationReason,
    ) -> Result<(), ProofError> {
        let record = RevocationRecord {
            revoked_at: SystemTime::now(),
            reason,
            affected_types: vec![proof_type],
            cascade: true,
            metadata: None,
        };

        self.revocation_history.push(record);
        Ok(())
    }

    /// Get all revocations in a time range
    pub fn get_revocations_in_range(
        &self,
        start: SystemTime,
        end: SystemTime,
    ) -> Vec<&RevocationRecord> {
        self.revocation_history
            .iter()
            .filter(|r| r.revoked_at >= start && r.revoked_at <= end)
            .collect()
    }
} 