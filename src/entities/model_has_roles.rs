use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelHasRole {
    pub id: i32,
    pub role_id: i32,
    pub model_type: String,
    pub model_id: i32,
}
