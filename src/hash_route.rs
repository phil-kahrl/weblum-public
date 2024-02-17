use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct HashState {
    pub current_image: Option<String>,
}

