//! Stable error codes for programmatic error handling.
//!
//! [`ErrorCode`] is the primary contract for identifying pipeline errors.
//! Codes are serialized as `snake_case` strings and are guaranteed to be
//! backward-compatible: new codes may be added, but existing codes never
//! change meaning.

use serde::{Deserialize, Serialize};

/// Stable, public error code identifying a category of pipeline error.
///
/// Match on this enum to handle errors programmatically. Because the enum is
/// `#[non_exhaustive]`, always include a wildcard arm to remain forward-compatible
/// with new codes added in future versions.
///
/// # Serialization
///
/// Serializes to and from `snake_case` strings via serde:
///
/// ```text
/// MissingStage      → "missing_stage"
/// InvalidCombo      → "invalid_combo"
/// ConvergenceFailed → "convergence_failed"
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ErrorCode {
    /// A required pipeline stage is missing from the spec.
    MissingStage,

    /// Two or more configuration options conflict with each other.
    InvalidCombo,

    /// A referenced module or component is not available.
    ModuleUnavailable,

    /// A numeric value exceeds its allowed range.
    LimitExceeded,

    /// The spec contains a field name that is not recognized.
    UnknownField,

    /// A field value is invalid for its expected type or context.
    InvalidValue,

    /// Two or more selected modules cannot be used together.
    IncompatibleModules,

    /// General validation failure that does not fit a more specific code.
    ValidationFailed,

    /// A pipeline stage failed during execution.
    StageFailed,

    /// PageRank iteration did not converge within the allowed iterations.
    ConvergenceFailed,
}

impl ErrorCode {
    /// Returns the canonical `snake_case` string form of this code.
    ///
    /// This matches the serde serialization output and is suitable for
    /// logging, JSON error responses, and Python-side error matching.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MissingStage => "missing_stage",
            Self::InvalidCombo => "invalid_combo",
            Self::ModuleUnavailable => "module_unavailable",
            Self::LimitExceeded => "limit_exceeded",
            Self::UnknownField => "unknown_field",
            Self::InvalidValue => "invalid_value",
            Self::IncompatibleModules => "incompatible_modules",
            Self::ValidationFailed => "validation_failed",
            Self::StageFailed => "stage_failed",
            Self::ConvergenceFailed => "convergence_failed",
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_matches_serde() {
        // Verify Display output is identical to serde serialization for every variant.
        let codes = [
            ErrorCode::MissingStage,
            ErrorCode::InvalidCombo,
            ErrorCode::ModuleUnavailable,
            ErrorCode::LimitExceeded,
            ErrorCode::UnknownField,
            ErrorCode::InvalidValue,
            ErrorCode::IncompatibleModules,
            ErrorCode::ValidationFailed,
            ErrorCode::StageFailed,
            ErrorCode::ConvergenceFailed,
        ];

        for code in &codes {
            let display = code.to_string();
            let serde_json = serde_json::to_value(code).unwrap();
            assert_eq!(
                display,
                serde_json.as_str().unwrap(),
                "Display and serde disagree for {code:?}"
            );
        }
    }

    #[test]
    fn test_serde_roundtrip() {
        let code = ErrorCode::LimitExceeded;
        let json = serde_json::to_string(&code).unwrap();
        assert_eq!(json, r#""limit_exceeded""#);

        let back: ErrorCode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, code);
    }

    #[test]
    fn test_as_str_all_variants() {
        assert_eq!(ErrorCode::MissingStage.as_str(), "missing_stage");
        assert_eq!(ErrorCode::InvalidCombo.as_str(), "invalid_combo");
        assert_eq!(ErrorCode::ModuleUnavailable.as_str(), "module_unavailable");
        assert_eq!(ErrorCode::LimitExceeded.as_str(), "limit_exceeded");
        assert_eq!(ErrorCode::UnknownField.as_str(), "unknown_field");
        assert_eq!(ErrorCode::InvalidValue.as_str(), "invalid_value");
        assert_eq!(
            ErrorCode::IncompatibleModules.as_str(),
            "incompatible_modules"
        );
        assert_eq!(ErrorCode::ValidationFailed.as_str(), "validation_failed");
        assert_eq!(ErrorCode::StageFailed.as_str(), "stage_failed");
        assert_eq!(ErrorCode::ConvergenceFailed.as_str(), "convergence_failed");
    }

    #[test]
    fn test_copy_semantics() {
        let a = ErrorCode::StageFailed;
        let b = a; // Copy, not move
        assert_eq!(a, b); // `a` is still usable
    }

    #[test]
    fn test_deserialize_rejects_unknown_code() {
        let result = serde_json::from_str::<ErrorCode>(r#""nonexistent_code""#);
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_in_collections() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(ErrorCode::MissingStage);
        set.insert(ErrorCode::MissingStage); // duplicate
        set.insert(ErrorCode::StageFailed);
        assert_eq!(set.len(), 2);
    }
}
