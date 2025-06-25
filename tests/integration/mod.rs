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