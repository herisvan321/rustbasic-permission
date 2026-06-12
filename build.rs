use std::fs;
use std::path::PathBuf;
use std::env;

fn main() {
    // Hanya jalankan jika kita tidak sedang dalam proses rilis atau docs
    if env::var("DOCS_RS").is_ok() {
        return;
    }

    // Ambil direktori kerja saat ini. 
    let project_root = match env::var("PWD") {
        Ok(pwd) => PathBuf::from(pwd),
        Err(_) => match env::current_dir() {
            Ok(dir) => dir,
            Err(_) => return,
        },
    };

    // Pastikan ini adalah proyek RustBasic (minimal ada Cargo.toml)
    if !project_root.join("Cargo.toml").exists() {
        return;
    }

    // JANGAN lakukan scaffolding jika kita sedang men-debug paket rustbasic-permission itu sendiri
    if project_root.join("src/bin/permission.rs").exists() {
        return;
    }

    // 1. Buat Migration
    let migrations_dir = project_root.join("database/migrations");
    fs::create_dir_all(&migrations_dir).ok();

    // Cek apakah sudah ada migrasi RBAC (hindari duplikasi)
    let existing_migrations = fs::read_dir(&migrations_dir)
        .map(|dir| dir.filter_map(|e| e.ok()).any(|e| e.file_name().to_string_lossy().contains("create_rbac_tables")))
        .unwrap_or(false);

    if !existing_migrations {
        let timestamp = get_current_timestamp();
        let migration_name = format!("m{}_create_rbac_tables", timestamp);
        let migration_path = migrations_dir.join(format!("{}.rs", migration_name));

        let migration_template = format!(
r#"use rustbasic_core::{{Schema, SchemaManager, MigrationTrait, DbErr}};
use rustbasic_core::async_trait;

pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {{
    fn name(&self) -> &str {{
        "{migration_name}"
    }}

    async fn up<'a>(&self, manager: &'a SchemaManager<'a>) -> Result<(), DbErr> {{
        // 1. Table roles
        Schema::create(manager, "roles", |table| {{
            table.string("name").unique().not_null();
            table.string("guard_name").default("web").not_null();
        }}).await?;

        // 2. Table permissions
        Schema::create(manager, "permissions", |table| {{
            table.string("name").unique().not_null();
            table.string("guard_name").default("web").not_null();
        }}).await?;

        // 3. Table model_has_roles
        Schema::create(manager, "model_has_roles", |table| {{
            table.no_timestamps();
            table.integer("role_id").not_null();
            table.string("model_type").not_null();
            table.integer("model_id").not_null();
        }}).await?;

        // 4. Table model_has_permissions
        Schema::create(manager, "model_has_permissions", |table| {{
            table.no_timestamps();
            table.integer("permission_id").not_null();
            table.string("model_type").not_null();
            table.integer("model_id").not_null();
        }}).await?;

        // 5. Table role_has_permissions
        Schema::create(manager, "role_has_permissions", |table| {{
            table.no_timestamps();
            table.integer("permission_id").not_null();
            table.integer("role_id").not_null();
        }}).await?;

        Ok(())
    }}

    async fn down<'a>(&self, manager: &'a SchemaManager<'a>) -> Result<(), DbErr> {{
        Schema::drop(manager, "role_has_permissions").await?;
        Schema::drop(manager, "model_has_permissions").await?;
        Schema::drop(manager, "model_has_roles").await?;
        Schema::drop(manager, "permissions").await?;
        Schema::drop(manager, "roles").await?;
        Ok(())
    }}
}}
"#, migration_name = migration_name);

        if fs::write(&migration_path, migration_template).is_ok() {
            update_migration_mod_rs(&project_root, &migration_name);
        }
    }

    // 2. Buat Model
    let models = vec![
        ("role", "roles", "Role", "    pub name: String,\n        pub guard_name: String,"),
        ("permission", "permissions", "Permission", "    pub name: String,\n        pub guard_name: String,"),
        ("model_has_role", "model_has_roles", "ModelHasRole", "    pub role_id: i32,\n        pub model_type: String,\n        pub model_id: i32,"),
        ("model_has_permission", "model_has_permissions", "ModelHasPermission", "    pub permission_id: i32,\n        pub model_type: String,\n        pub model_id: i32,"),
        ("role_has_permission", "role_has_permissions", "RoleHasPermission", "    pub permission_id: i32,\n        pub role_id: i32,"),
    ];

    let models_dir = project_root.join("src/app/models");
    fs::create_dir_all(&models_dir).ok();

    for (name, table, class_name, fields) in models {
        let file_path = models_dir.join(format!("{}.rs", name));
        if !file_path.exists() {
            let model_template = format!(
r#"use rustbasic_core::model;

model! {{
    table: "{table}",
    Model {{
        pub id: i32,
    {fields}
    }}
}}
"#, table = table, fields = fields);

            if fs::write(&file_path, model_template).is_ok() {
                update_model_mod_rs(&project_root, class_name, name);
            }
        }
    }
}

fn update_migration_mod_rs(project_root: &std::path::Path, mod_name: &str) {
    let mod_path = project_root.join("database/migrations/mod.rs");
    if !mod_path.exists() { return; }

    let mut content = fs::read_to_string(&mod_path).unwrap_or_default();

    if !content.contains(&format!("pub mod {};", mod_name)) {
        content.push_str(&format!("\npub mod {};\n", mod_name));
    }

    let search_pattern = "fn migrations() -> Vec<Box<dyn MigrationTrait>> {";
    if let Some(pos) = content.find(search_pattern)
        && let Some(insert_pos) = content[pos..].find("        ]") {
        content.insert_str(pos + insert_pos, &format!("            Box::new({}::Migration),\n", mod_name));
    }

    fs::write(mod_path, content).ok();
}

fn update_model_mod_rs(project_root: &std::path::Path, class_name: &str, snake_name: &str) {
    let mod_path = project_root.join("src/app/models/mod.rs");
    if !mod_path.exists() { return; }

    let content = fs::read_to_string(&mod_path).unwrap_or_default();
    if content.contains(&format!("pub mod {};", snake_name)) {
        return;
    }

    let mut file = fs::OpenOptions::new().append(true).open(mod_path).unwrap();
    use std::io::Write;
    writeln!(file, "pub mod {};", snake_name).ok();
    writeln!(file, "pub use {}::Model as {};", snake_name, class_name).ok();
}

fn get_current_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let secs = now;
    let days = if secs >= 0 { secs / 86400 } else { (secs - 86399) / 86400 };
    let mut rem_secs = (secs - days * 86400) as u32;
    let hour = rem_secs / 3600;
    rem_secs %= 3600;
    let min = rem_secs / 60;
    let sec = rem_secs % 60;

    let z = days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe/1460 + doe/36524 - doe/146096) / 365;
    let mut y = (yoe as i32) + era as i32 * 400;
    let doy = doe - (365*yoe + yoe/4 - yoe/100);
    let mp = (5*doy + 2)/153;
    let d = doy - (153*mp + 2)/5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    if m <= 2 {
        y += 1;
    }
    format!("{:04}{:02}{:02}_{:02}{:02}{:02}", y, m, d, hour, min, sec)
}
