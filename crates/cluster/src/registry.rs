use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Registry {
  url: String,
}

impl Registry {
  pub fn new(url: String) -> Self {
    Self { url }
  }
}
