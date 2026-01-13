// Comprehensive Error Handling Module
// Standardized error types for the entire system

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TradingError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Division by zero in {0}")]
    DivisionByZero(String),
    
    #[error("API error: {0}")]
    ApiError(String),
    
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Risk limit exceeded: {0}")]
    RiskLimitExceeded(String),
    
    #[error("Insufficient capital: required {required}, available {available}")]
    InsufficientCapital { required: f64, available: f64 },
    
    #[error("Position size exceeds limit: {size} > {limit}")]
    PositionSizeExceeded { size: f64, limit: f64 },
    
    #[error("Leverage exceeds maximum: {leverage}x > {max_leverage}x")]
    LeverageExceeded { leverage: f64, max_leverage: f64 },
    
    #[error("Confidence too low: {confidence} < {minimum}")]
    ConfidenceTooLow { confidence: f64, minimum: f64 },
    
    #[error("Execution timeout: {operation} took longer than {timeout_ms}ms")]
    ExecutionTimeout { operation: String, timeout_ms: u64 },
    
    #[error("Circuit breaker triggered: {reason}")]
    CircuitBreakerTriggered { reason: String },
    
    #[error("Data validation failed: {field} - {reason}")]
    ValidationError { field: String, reason: String },
    
    #[error("Memory limit exceeded: {current} > {limit}")]
    MemoryLimitExceeded { current: usize, limit: usize },
    
    #[error("Rate limit exceeded: {service} - retry after {retry_after_secs}s")]
    RateLimitExceeded { service: String, retry_after_secs: u64 },
}

pub type TradingResult<T> = Result<T, TradingError>;

// Validation helpers
pub fn validate_positive(value: f64, name: &str) -> TradingResult<f64> {
    if value.is_nan() || value.is_infinite() {
        return Err(TradingError::InvalidInput(format!("{} must be finite", name)));
    }
    if value <= 0.0 {
        return Err(TradingError::InvalidInput(format!("{} must be positive, got {}", name, value)));
    }
    Ok(value)
}

pub fn validate_non_negative(value: f64, name: &str) -> TradingResult<f64> {
    if value.is_nan() || value.is_infinite() {
        return Err(TradingError::InvalidInput(format!("{} must be finite", name)));
    }
    if value < 0.0 {
        return Err(TradingError::InvalidInput(format!("{} must be non-negative, got {}", name, value)));
    }
    Ok(value)
}

pub fn validate_bounds(value: f64, min: f64, max: f64, name: &str) -> TradingResult<f64> {
    validate_non_negative(value, name)?;
    if value < min || value > max {
        return Err(TradingError::InvalidInput(
            format!("{} must be between {} and {}, got {}", name, min, max, value)
        ));
    }
    Ok(value)
}

pub fn safe_divide(numerator: f64, denominator: f64, context: &str) -> TradingResult<f64> {
    if denominator == 0.0 {
        return Err(TradingError::DivisionByZero(context.to_string()));
    }
    if denominator.is_nan() || denominator.is_infinite() {
        return Err(TradingError::InvalidInput(format!("Denominator in {} is invalid", context)));
    }
    Ok(numerator / denominator)
}

pub fn validate_capital(required: f64, available: f64) -> TradingResult<()> {
    if required > available {
        return Err(TradingError::InsufficientCapital { required, available });
    }
    Ok(())
}

pub fn validate_leverage(leverage: f64, max_leverage: f64) -> TradingResult<()> {
    if leverage > max_leverage {
        return Err(TradingError::LeverageExceeded { leverage, max_leverage });
    }
    if leverage < 1.0 {
        return Err(TradingError::InvalidInput(format!("Leverage must be >= 1.0, got {}", leverage)));
    }
    Ok(())
}

pub fn validate_confidence(confidence: f64, minimum: f64) -> TradingResult<()> {
    if confidence < minimum {
        return Err(TradingError::ConfidenceTooLow { confidence, minimum });
    }
    if confidence > 1.0 {
        return Err(TradingError::InvalidInput(format!("Confidence cannot exceed 1.0, got {}", confidence)));
    }
    Ok(())
}
