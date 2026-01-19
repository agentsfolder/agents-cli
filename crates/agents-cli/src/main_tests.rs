#[cfg(test)]
mod tests {
    use crate::{AppError, ErrorCategory};

    #[test]
    fn exit_code_mapping_is_stable() {
        let mk = |category: ErrorCategory| AppError {
            category,
            message: "x".to_string(),
            context: vec![],
        };

        assert_eq!(mk(ErrorCategory::InvalidArgs).exit_code(), 2);
        assert_eq!(mk(ErrorCategory::NotInitialized).exit_code(), 3);
        assert_eq!(mk(ErrorCategory::SchemaInvalid).exit_code(), 4);

        assert_eq!(mk(ErrorCategory::Io).exit_code(), 5);
        assert_eq!(mk(ErrorCategory::Conflict).exit_code(), 5);
        assert_eq!(mk(ErrorCategory::PolicyDenied).exit_code(), 5);
        assert_eq!(mk(ErrorCategory::ExternalToolMissing).exit_code(), 5);
    }
}
