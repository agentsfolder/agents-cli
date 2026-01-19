use std::fmt;

use serde::{Deserialize, Serialize};

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, IdError> {
                let v = value.into();
                if v.trim().is_empty() {
                    return Err(IdError::Empty);
                }
                Ok(Self(v))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum IdError {
    #[error("id must not be empty")]
    Empty,
}

id_type!(ModeId);
id_type!(PolicyId);
id_type!(SkillId);
id_type!(AdapterId);
id_type!(ScopeId);
