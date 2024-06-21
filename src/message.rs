use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Message {
    pub username: String,
    pub content: String,
    pub room: String,
    pub password: Option<String>,
}
