use std::ops::Deref;

#[derive(Clone, Debug)]
pub struct JobName(String);

impl JobName {
    pub fn new(name: &str) -> Result<Self, &'static str> {
        if name.len() > 253 {
            return Err("Job name must not exceed 253 characters.");
        }
        if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '.' || c == '_') {
            return Err("Job name can only contain alphanumeric characters, '-', '.', and '_'.");
        }
        if let Some(first_char) = name.chars().next() {
            if !first_char.is_ascii_alphanumeric() {
                return Err("Job name must start with an alphanumeric character.");
            }
        }
        Ok(JobName(name.to_string()))
    }
}

impl Deref for JobName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&str> for JobName {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        JobName::new(value)
    }
}