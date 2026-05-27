use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelHasPermission {
    pub id: i32,
    pub permission_id: i32,
    pub model_type: String,
    pub model_id: i32,
}
