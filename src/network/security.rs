use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::time::Duration;
use crate::network::{Peer, NetworkError};
use crate::Result;

/// Network security manager
#[async_trait]
pub trait SecurityManager: Send + Sync {
    /// Initialize security manager
    async fn init(&mut self, config: SecurityConfig) -> Result<()>;

    /// Authenticate a peer
    async fn authenticate_peer(&self, peer: &Peer) -> Result<AuthenticationResult>;

    /// Authorize an action
    async fn authorize_action(&self, action: &Action, peer: &Peer) -> Result<bool>;

    /// Generate session keys
    async fn generate_session(&self, peer: &Peer) -> Result<SessionKeys>;

    /// Validate message signature
    async fn validate_signature(&self, message: &[u8], signature: &[u8], peer: &Peer) -> Result<bool>;

    /// Get security metrics
    fn metrics(&self) -> SecurityMetrics;
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub authentication_method: AuthenticationMethod,
    pub key_rotation_interval: Duration,
    pub signature_algorithm: String,
    pub tls_config: Option<TlsConfig>,
    pub rate_limiting: RateLimitConfig,
}

/// Authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthenticationMethod {
    Certificate {
        ca_cert: String,
        client_cert: String,
        client_key: String,
    },
    Token {
        token_type: String,
        token_value: String,
    },
    MultiFactor {
        methods: Vec<String>,
        required_factors: usize,
    },
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
    pub ca_path: Option<String>,
    pub verify_peer: bool,
    pub cipher_suites: Vec<String>,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub max_requests: u32,
    pub window_size: Duration,
    pub per_ip_limit: bool,
    pub burst_size: u32,
}

/// Network action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub action_type: ActionType,
    pub resource: String,
    pub timestamp: std::time::SystemTime,
    pub metadata: serde_json::Value,
}

/// Action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Connect,
    Disconnect,
    SendMessage,
    ReceiveMessage,
    BroadcastMessage,
    ModifyPeer,
    AccessResource,
}

/// Authentication result
#[derive(Debug, Clone)]
pub struct AuthenticationResult {
    pub success: bool,
    pub session_id: Option<uuid::Uuid>,
    pub permissions: Vec<String>,
    pub expiry: Option<std::time::SystemTime>,
}

/// Session keys
#[derive(Debug, Clone)]
pub struct SessionKeys {
    pub session_id: uuid::Uuid,
    pub encryption_key: Vec<u8>,
    pub signing_key: Vec<u8>,
    pub created_at: std::time::SystemTime,
    pub expires_at: std::time::SystemTime,
}

/// Security metrics
#[derive(Debug, Clone, Default)]
pub struct SecurityMetrics {
    pub authentication_attempts: u64,
    pub failed_authentications: u64,
    pub active_sessions: usize,
    pub revoked_sessions: u64,
    pub rate_limit_hits: u64,
    pub signature_validations: u64,
    pub failed_validations: u64,
} 