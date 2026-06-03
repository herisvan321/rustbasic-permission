use rustbasic_core::model;

model! {
    table: "role_has_permissions",
    timestamps: false,
    fillable: [permission_id, role_id],
    guarded: [],
    Model {
        pub id: i32,
        pub permission_id: i32,
        pub role_id: i32,
    }
}
