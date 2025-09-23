use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct SchemaVersion(String);

const CURRENT_SCHEMA_VERS: &str = "1.1.2";

impl SchemaVersion {
    #[must_use]
    pub fn current() -> Self {
        Self(CURRENT_SCHEMA_VERS.to_string())
    }
}

impl Default for SchemaVersion {
    fn default() -> Self {
        Self::current()
    }
}

impl AsRef<str> for SchemaVersion {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for SchemaVersion {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
