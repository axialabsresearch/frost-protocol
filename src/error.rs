/*!
# Error Module

This module implements the core error handling system for the FROST protocol,
providing a comprehensive error type hierarchy and error management utilities.

## Core Components

### Error Types
The error system includes:
- Message errors
- State errors
- Network errors
- Routing errors
- Finality errors
- IO errors
- Connection errors

### Error Management
Error handling features:
- Error categorization
- Retry handling
- Error conversion
- Error propagation

### Error Integration
Integration points:
- Message system
- State system
- Network layer
- Routing system
- Finality system

### Error Recovery
Recovery features:
- Retry detection
- Error categorization
- Recovery strategies
- Error propagation

## Architecture

The error system implements several key components:

1. **Core Error Type**
   ```rust
   use frost_protocol::finality::FinalityError;
   pub enum Error {
       Message(String),
       State(String),
       Network(String),
       Routing(String),
       Finality(FinalityError),
       Io(std::io::Error),
       Other(String),
       ConnectionDenied(String),
       MessageProcessing(String),
       Generic(String),
   }
   ```
   - Error categories
   - Error context
   - Error conversion
   - Error handling

2. **Error Handling**
   ```rust
   impl Error {
       pub fn is_retryable(&self) -> bool {
           match self {
               Error::Message(_) => true,
               Error::State(_) => false,
               // ... other cases
           }
       }
   }
   ```
   - Retry detection
   - Error analysis
   - Recovery handling
   - Error management

3. **Error Conversion**
   ```rust
   impl From<libp2p::swarm::ConnectionDenied> for Error {
       fn from(err: libp2p::swarm::ConnectionDenied) -> Self {
           Error::ConnectionDenied(err.to_string())
       }
   }
   ```
   - Type conversion
   - Error mapping
   - Context preservation
   - Error propagation

## Features

### Error Categories
- Message errors
- State errors
- Network errors
- System errors
- Custom errors

### Error Management
- Error detection
- Error handling
- Error recovery
- Error propagation

### Error Integration
- System integration
- Error conversion
- Error mapping
- Error handling

### Error Recovery
- Retry handling
- Recovery strategies
- Error analysis
- Error resolution

## Best Practices

### Error Handling
1. Error Creation
   - Proper categorization
   - Context inclusion
   - Error mapping
   - Recovery options

2. Error Processing
   - Error analysis
   - Recovery planning
   - Retry handling
   - Error propagation

3. Error Conversion
   - Type mapping
   - Context preservation
   - Error transformation
   - Error propagation

4. Error Recovery
   - Strategy selection
   - Retry handling
   - Error resolution
   - Recovery tracking

## Integration

### Message System
- Message errors
- Processing errors
- Conversion handling
- Error propagation

### State System
- State errors
- Validation errors
- Processing errors
- Error handling

### Network System
- Connection errors
- Network errors
- Protocol errors
- Error handling

### Routing System
- Routing errors
- Path errors
- Processing errors
- Error handling

## Performance Considerations

### Error Creation
- Efficient construction
- Context management
- Memory usage
- Resource sharing

### Error Processing
- Quick analysis
- Fast categorization
- Efficient handling
- Resource management

### Error Recovery
- Strategy selection
- Resource allocation
- Performance impact
- System stability

### Error Propagation
- Efficient propagation
- Context preservation
- Resource management
- System stability

## Implementation Notes

### Error Categories
Error types include:
- Message handling
- State management
- Network operations
- System operations
- Custom handling

### Error Handling
Error processing includes:
- Category detection
- Retry analysis
- Recovery planning
- Error resolution

### Error Recovery
Recovery handling includes:
- Strategy selection
- Retry management
- Resource handling
- System stability

### Error Integration
Integration includes:
- System binding
- Error mapping
- Context handling
- Error propagation
*/

use thiserror::Error;

/// Core protocol error type
#[derive(Error, Debug)]
pub enum Error {
    /// Message error
    #[error("Message error: {0}")]
    Message(String),

    /// State error
    #[error("State error: {0}")]
    State(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Routing error
    #[error("Routing error: {0}")]
    Routing(String),

    /// Finality error
    #[error("Finality error: {0}")]
    Finality(#[from] crate::finality::FinalityError),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Other error
    #[error("Other error: {0}")]
    Other(String),

    /// Connection denied error
    #[error("Connection denied error: {0}")]
    ConnectionDenied(String),

    /// Message processing error
    #[error("Message processing error: {0}")]
    MessageProcessing(String),

    /// Generic error with message
    #[error("Generic error: {0}")]
    Generic(String),
}

impl Error {
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            Error::Message(_) => true,
            Error::State(_) => false,
            Error::Network(_) => true,
            Error::Routing(_) => true,
            Error::Finality(e) => e.is_retryable(),
            Error::Io(_) => true,
            Error::Other(_) => false,
            Error::ConnectionDenied(_) => false,
            Error::MessageProcessing(_) => false,
            Error::Generic(_) => false,
        }
    }
}

impl From<libp2p::swarm::ConnectionDenied> for Error {
    fn from(err: libp2p::swarm::ConnectionDenied) -> Self {
        Error::ConnectionDenied(err.to_string())
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Generic(s.to_string())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Generic(s)
    }
}

impl From<crate::message::MessageError> for Error {
    fn from(err: crate::message::MessageError) -> Self {
        Error::Message(err.to_string())
    }
} 