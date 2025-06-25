mod finality_verification;
mod network_routing_test;
mod state_finality_test;
mod state_transition_test;

pub use network_routing_test::*;
pub use state_finality_test::*;
pub use state_transition_test::*;

pub mod state;
pub mod network;
pub mod finality;

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