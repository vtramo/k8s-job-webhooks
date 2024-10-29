use std::fmt;
use std::ops::Deref;
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct JobName(String);

#[derive(Debug, Error)]
pub enum JobNameError {
    #[error("Job name must not exceed 253 characters.")]
    TooLong,
    #[error("Job name can only contain alphanumeric characters, '-', '.', and '_'.")]
    InvalidCharacters,
    #[error("Job name must start with an alphanumeric character.")]
    InvalidStartCharacter,
}

impl JobName {
    pub fn new(name: &str) -> Result<Self, JobNameError> {
        if name.len() > 253 {
            return Err(JobNameError::TooLong);
        }
        if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '.' || c == '_') {
            return Err(JobNameError::InvalidCharacters);
        }
        if let Some(first_char) = name.chars().next() {
            if !first_char.is_ascii_alphanumeric() {
                return Err(JobNameError::InvalidStartCharacter);
            }
        }
        Ok(JobName(name.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for JobName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


impl Deref for JobName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&str> for JobName {
    type Error = JobNameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        JobName::new(value)
    }
}