/*!
# Message Handler Module

This module provides message handling functionality for the FROST protocol's messaging
system. It defines interfaces and types for processing, queuing, and tracking
message status.

## Core Components

### Message Handler
- Message processing
- Queue management
- Status tracking
- Retry handling

### Message Status
- Queue position
- Processing state
- Completion status
- Error handling

### Message Results
- Success tracking
- Timing metrics
- Result metadata
- Error details

## Architecture

The handler system consists of several key components:

1. **Message Handler**
   ```rust
   pub trait MessageHandler: Send + Sync {
       async fn handle_message(&self, message: FrostMessage) -> Result<MessageStatus>;
       async fn queue_message(&self, message: FrostMessage) -> Result<()>;
       async fn message_status(&self, message_id: uuid::Uuid) -> Result<MessageStatus>;
       async fn retry_message(&self, message_id: uuid::Uuid) -> Result<MessageStatus>;
   }
   ```
   - Message processing
   - Queue management
   - Status tracking
   - Retry handling

2. **Message Status**
   ```rust
   pub enum MessageStatus {
       Queued { position: u64, estimated_time: Duration },
       Processing { started_at: SystemTime, progress: f32 },
       Completed { completed_at: SystemTime, result: MessageResult },
       Failed { error: MessageError, can_retry: bool },
   }
   ```
   - Status tracking
   - Progress monitoring
   - Result handling
   - Error management

3. **Message Result**
   ```rust
   pub struct MessageResult {
       success: bool,
       processing_time: Duration,
       metadata: Value,
   }
   ```
   - Success tracking
   - Performance metrics
   - Result metadata
   - Processing details

## Features

### Message Processing
- Async handling
- Queue management
- Status tracking
- Retry support

### Status Management
- Queue position
- Progress tracking
- Result handling
- Error tracking

### Result Handling
- Success tracking
- Timing metrics
- Metadata handling
- Error details

### Retry Management
- Retry decisions
- Queue handling
- Status updates
- Error recovery

## Best Practices

1. **Message Handling**
   - Proper queuing
   - Status tracking
   - Error handling
   - Resource management

2. **Queue Management**
   - Priority handling
   - Resource limits
   - Timeout handling
   - Cleanup routines

3. **Status Tracking**
   - Progress updates
   - Timing metrics
   - Error tracking
   - Resource usage

4. **Retry Handling**
   - Retry decisions
   - Queue position
   - Resource impact
   - Error recovery

## Integration

The handler system integrates with:
1. Message validation
2. Chain management
3. State transitions
4. Protocol operations
*/

#![allow(unused_imports)]
#![allow(unused_variables)]

use async_trait::async_trait;
use crate::message::{FrostMessage, MessageType, MessageError};
use crate::Result;

/// Handler for FROST Protocol messages
#[async_trait]
pub trait MessageHandler: Send + Sync {
    /// Process an incoming message
    async fn handle_message(&self, message: FrostMessage) -> Result<MessageStatus>;
    
    /// Queue a message for processing
    async fn queue_message(&self, message: FrostMessage) -> Result<()>;
    
    /// Get status of a message
    async fn message_status(&self, message_id: uuid::Uuid) -> Result<MessageStatus>;
    
    /// Retry a failed message
    async fn retry_message(&self, message_id: uuid::Uuid) -> Result<MessageStatus>;
}

/// Status of message processing
#[derive(Debug, Clone)]
pub enum MessageStatus {
    Queued {
        position: u64,
        estimated_time: std::time::Duration,
    },
    Processing {
        started_at: std::time::SystemTime,
        progress: f32,
    },
    Completed {
        completed_at: std::time::SystemTime,
        result: MessageResult,
    },
    Failed {
        error: MessageError,
        can_retry: bool,
    },
}

/// Result of message processing
#[derive(Debug, Clone)]
pub struct MessageResult {
    pub success: bool,
    pub processing_time: std::time::Duration,
    pub metadata: serde_json::Value,
} 