pub mod logging {
    use tracing_subscriber::EnvFilter;
    use serde::{Deserialize, Deserializer, Serializer};

    pub use tracing::{trace, error, info, warn, debug};
    pub use tracing::level_filters::LevelFilter;
    
    pub fn serialize<S>(level: &LevelFilter, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match *level {
            LevelFilter::OFF => "off",
            LevelFilter::ERROR => "error",
            LevelFilter::WARN => "warn",
            LevelFilter::INFO => "info",
            LevelFilter::DEBUG => "debug",
            LevelFilter::TRACE => "trace",
        })
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<LevelFilter, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "off" => Ok(LevelFilter::OFF),
            "error" => Ok(LevelFilter::ERROR),
            "warn" => Ok(LevelFilter::WARN),
            "info" => Ok(LevelFilter::INFO),
            "debug" => Ok(LevelFilter::DEBUG),
            "trace" => Ok(LevelFilter::TRACE),
            _ => Err(serde::de::Error::custom("Invalid log level")),
        }
    }

    pub fn init(level: &LevelFilter) {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::new(level.to_string()))
            .init();
    }
}