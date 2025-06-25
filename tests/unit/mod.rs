mod state_types;
mod state_error;
mod state_props;
mod finality_verifier;
mod message_validator;
mod chain_metrics;
mod predicate_validator;

pub mod chain;
pub mod validation;
pub mod state;

#[cfg(test)]
mod tests {
    use test_log::test;
    
    #[test]
    fn init_logging() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter("trace")
            .try_init();
    }
} 