mod state_types;
mod state_error;
mod state_props;

pub mod chain;
pub mod validation;

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