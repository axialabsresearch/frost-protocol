use frost_protocol::state::{
    BlockRef,
    ChainId,
    error::{StateError, ErrorSeverity},
};

#[test]
fn test_state_error_display() {
    let chain_id = ChainId::new("ethereum");
    let block_ref = BlockRef::new(chain_id, 100, [0u8; 32]);
    
    let errors = vec![
        StateError::InvalidTransition("invalid state".into()),
        StateError::ProofVerificationFailed("bad proof".into()),
        StateError::InvalidBlockRef("missing block".into()),
        StateError::RootMismatch {
            block_ref: block_ref.clone(),
            expected: "0x123".into(),
            actual: "0x456".into(),
        },
        StateError::ChainSpecific("network error".into()),
        StateError::Internal("system error".into()),
    ];
    
    for error in errors {
        let error_str = error.to_string();
        assert!(!error_str.is_empty());
        
        // Verify error contains relevant information
        match &error {
            StateError::RootMismatch { expected, actual, .. } => {
                assert!(error_str.contains(expected));
                assert!(error_str.contains(actual));
                assert!(error_str.contains(&block_ref.to_string()));
            }
            _ => {
                // Other errors should contain their message
                if let Some(msg) = error.to_string().split(": ").nth(1) {
                    assert!(!msg.is_empty());
                }
            }
        }
    }
}

#[test]
fn test_error_retryability() {
    let retryable = StateError::ChainSpecific("network timeout".into());
    assert!(retryable.is_retryable());
    
    let non_retryable = vec![
        StateError::InvalidTransition("invalid".into()),
        StateError::ProofVerificationFailed("bad proof".into()),
        StateError::InvalidBlockRef("missing".into()),
        StateError::RootMismatch {
            block_ref: BlockRef::default(),
            expected: "0x123".into(),
            actual: "0x456".into(),
        },
        StateError::Internal("error".into()),
    ];
    
    for error in non_retryable {
        assert!(!error.is_retryable());
    }
}

#[test]
fn test_error_severity() {
    let test_cases = vec![
        (StateError::InvalidTransition("test".into()), ErrorSeverity::Error),
        (StateError::ProofVerificationFailed("test".into()), ErrorSeverity::Critical),
        (StateError::InvalidBlockRef("test".into()), ErrorSeverity::Error),
        (StateError::RootMismatch {
            block_ref: BlockRef::default(),
            expected: "test".into(),
            actual: "test".into(),
        }, ErrorSeverity::Critical),
        (StateError::ChainSpecific("test".into()), ErrorSeverity::Warning),
        (StateError::Internal("test".into()), ErrorSeverity::Critical),
    ];
    
    for (error, expected_severity) in test_cases {
        assert_eq!(error.severity(), expected_severity);
    }
}

#[test]
fn test_error_severity_ordering() {
    // Test using partial_cmp instead of direct comparison
    assert!(matches!(ErrorSeverity::Critical.partial_cmp(&ErrorSeverity::Error), Some(std::cmp::Ordering::Greater)));
    assert!(matches!(ErrorSeverity::Error.partial_cmp(&ErrorSeverity::Warning), Some(std::cmp::Ordering::Greater)));
    assert!(matches!(ErrorSeverity::Critical.partial_cmp(&ErrorSeverity::Warning), Some(std::cmp::Ordering::Greater)));
    
    let severities = vec![
        ErrorSeverity::Warning,
        ErrorSeverity::Critical,
        ErrorSeverity::Error,
    ];
    
    let mut sorted = severities.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    assert_eq!(sorted, vec![
        ErrorSeverity::Warning,
        ErrorSeverity::Error,
        ErrorSeverity::Critical,
    ]);
} 