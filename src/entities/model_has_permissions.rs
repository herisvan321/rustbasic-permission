use rustbasic_core::model;

model! {
    table: "model_has_permissions",
    timestamps: false,
    fillable: [permission_id, model_type, model_id],
    guarded: [],
    Model {
        pub id: i32,
        pub permission_id: i32,
        pub model_type: String,
        pub model_id: i32,
    }
}
