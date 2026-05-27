use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoleHasPermission {
    pub id: i32,
    pub permission_id: i32,
    pub role_id: i32,
}
