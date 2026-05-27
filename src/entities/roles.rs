use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Role {
    pub id: i32,
    pub name: String,
    pub guard_name: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}
