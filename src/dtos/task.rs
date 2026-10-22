use serde::Deserialize;
use validator::{Validate, ValidationError};

#[derive(Deserialize, Validate)]
pub struct CreateTask {
    #[validate(length(min = 1), custom(function = "validate_title"))]
    pub title: String,
    pub completed: bool,
}

fn validate_title(title: &str) -> Result<(), ValidationError> {
    if title.trim().is_empty() {
        return Err(ValidationError::new("whitespace_in_title"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_whitespace_only_title() {
        let task = CreateTask {
            title: "   \t\n".to_string(),
            completed: false,
        };

        assert!(task.validate().is_err());
    }
}