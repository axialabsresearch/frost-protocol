/*!
# Extension Error Types

This module defines the error types and handling for the FROST protocol extension system.
It provides a comprehensive set of error variants that cover various failure modes in
extension management, dependency resolution, and runtime operations.

## Error Categories

### Extension Management
- `NotFound` - Extension lookup failures
- `AlreadyExists` - Duplicate registration attempts
- `InvalidStateTransition` - Invalid lifecycle transitions

### Dependency Management
- `DependencyError` - General dependency issues
- `CircularDependency` - Cyclic dependency detection
- `DependencyResolutionFailed` - Resolution failures

### Version Management
- `VersionParseError` - Version string parsing errors
- `IncompatibleVersion` - Version compatibility issues

### Runtime Errors
- `OperationTimeout` - Operation timeouts
- `ResourceLimitExceeded` - Resource exhaustion
- `Other` - Fallback for general errors

## Usage Example

```rust
use frost_protocol::extensions::errors::{ExtensionError, ExtensionResult};

async fn register_extension(id: String) -> ExtensionResult<()> {
    if extension_exists(&id) {
        return Err(ExtensionError::AlreadyExists(id));
    }
    
    if !validate_dependencies(&id) {
        return Err(ExtensionError::DependencyError(
            format!("Missing dependencies for {}", id)
        ));
    }
    
    // Registration logic...
    Ok(())
}
```

## Error Handling Best Practices

1. **Specific Error Types**
   - Use specific error variants
   - Include context in errors
   - Avoid generic errors
   - Maintain error chain

2. **Error Context**
   - Include relevant identifiers
   - Add state information
   - Preserve error source
   - Add debugging context

3. **Error Recovery**
   - Handle each variant appropriately
   - Implement fallback behavior
   - Clean up resources
   - Maintain consistency

4. **Error Propagation**
   - Use `?` operator with `ExtensionResult`
   - Preserve error context
   - Add context when converting
   - Handle error chains

## Integration

The error system integrates with:
1. Extension lifecycle management
2. Dependency resolution system
3. Version compatibility checks
4. Resource management
*/

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