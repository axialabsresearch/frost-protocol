#[derive(Debug, thiserror::Error)]
pub enum ExtensionError {
    #[error("Extension not found: {0}")]
    NotFound(String),
    
    #[error("Extension {0} already exists")]
    AlreadyExists(String),
    
    #[error("Invalid state transition from {from} to {to}")]
    InvalidStateTransition {
        from: String,
        to: String,
    },
    
    #[error("Dependency error: {0}")]
    DependencyError(String),
    
    #[error("Operation timeout")]
    OperationTimeout,
    
    #[error("Resource limit exceeded")]
    ResourceLimitExceeded,
    
    #[error("Circular dependency detected between {0} and {1}")]
    CircularDependency(String, String),
    
    #[error("Failed to parse version: {0}")]
    VersionParseError(String),
    
    #[error("Extension {extension} version {actual} is incompatible (requires {required})")]
    IncompatibleVersion {
        extension: String,
        required: String,
        actual: String,
    },
    
    #[error("Dependency resolution failed")]
    DependencyResolutionFailed,
    
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type ExtensionResult<T> = Result<T, ExtensionError>; 