use rustbasic_core::model;

model! {
    table: "roles",
    timestamps: true,
    fillable: [name, guard_name],
    guarded: [],
    Model {
        pub id: i32,
        pub name: String,
        pub guard_name: String,
        pub created_at: Option<String>,
        pub updated_at: Option<String>,
    }
}
