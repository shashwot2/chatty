use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct Message {
    pub username: String,
    pub content: String,
}
