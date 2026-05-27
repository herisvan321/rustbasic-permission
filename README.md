# 🔐 rustbasic-permission

Package **Role-Based Access Control (RBAC)** yang sangat elegan, cepat, dan terintegrasi penuh untuk framework **RustBasic**, menghadirkan fungsionalitas tingkat tinggi dengan otomatisasi berarsitektur **Global View Interceptor**.

Dirancang khusus untuk ekosistem **RustBasic** dan **sqlx**, package ini memungkinkan Anda mengelola Role dan Permission dengan kemudahan maksimal, keamanan tipe (type-safety) khas Rust, dan **tanpa perlu konfigurasi atau pengiriman variabel secara berulang di setiap Controller**.

---

## ✨ Fitur Premium

- **Intuitive API**: Pola desain manajemen entitas RBAC yang sangat intuitif, terstruktur, dan mudah dipahami.
- **Auto-Migration**: Pembuatan 5 tabel utama dan pivot secara instan saat inisialisasi koneksi database.
- **Pewarisan Hak Akses (Inheritance)**: Resolusi hak akses berlapis, memverifikasi izin langsung entitas maupun warisan dari peran aktifnya.
- **Global View Interceptor**: Otomatisasi injeksi kapabilitas pengguna aktif langsung ke seluruh *template rendering* secara global.
- **Zero-Config Controllers**: Controller Anda tetap bersih dan murni tanpa polusi kode ekstraksi izin.

---

## 📦 Struktur Skema Database Normalisasi

Secara otomatis mengelola 5 tabel standar:
1. `roles`
2. `permissions`
3. `model_has_roles`
4. `model_has_permissions`
5. `role_has_permissions`

---

## 🚀 Panduan Instalasi & Penggunaan Lengkap

### 1. Instalasi
Tambahkan `rustbasic-permission` ke dalam berkas `Cargo.toml` pada root proyek aplikasi Anda:

```toml
[dependencies]
rustbasic-permission = "0.0.7"
```

---

### 2. Inisialisasi Otomatis (Magic Scaffolding)
Cukup jalankan build pada proyek Anda, dan `rustbasic-permission` akan secara otomatis membuat migrasi dan model yang diperlukan jika belum ada:

```bash
cargo build
```

Perintah ini akan secara otomatis membuat:
- 📂 **Migration**: File migrasi baru di `database/migrations/` untuk 5 tabel RBAC.
- 📂 **Models**: File model di `src/app/models/` (`Role`, `Permission`, dll).

Setelah menjalankan perintah di atas, jalankan migrasi database:
```bash
rustbasic migrate
```

---

### 3. Otomatisasi Caching Sesi saat Login
Agar seluruh halaman langsung mengetahui kapabilitas pengguna tanpa membebani kueri database pada setiap *render*, tambahkan penyimpanan *cache* kapabilitas ke dalam sesi saat pengguna berhasil login (misal di `AuthController::login`):

```rust
use rustbasic_permission::PermissionChecker;

// Setelah verifikasi password berhasil:
req.session.set("user_id", u.id);

// Ekstrak & Cache kapabilitas RBAC secara asinkron satu kali saja
let checker = PermissionChecker::new(&state.db, &req.session);
let perms = checker.user_permissions().await;
let roles = checker.user_roles().await;

req.session.set("user_permissions", perms);
req.session.set("user_roles", roles);

// Redirect ke dashboard
return ResponseHelper::redirect_with_success("/dashboard", "Selamat datang!", req.session);
```

> **💡 Tips Manajemen Matriks**: Jangan lupa untuk memanggil logika pembaruan sesi yang sama setelah Administrator memperbarui matriks RBAC agar pemetaan menu langsung diperbarui secara *real-time*.

---

### 4. Membuat View Interceptor Proyek (Tanpa Menyentuh Core)
Untuk menyuntikkan daftar *permission* dan *role* ke seluruh *template* tanpa mengganggu pustaka inti framework, buat fungsi penyekat (interceptor) `view` pada tingkat proyek Anda (misalnya di `src/app/mod.rs`):

```rust
pub use rustbasic_core::view::render;
use rustbasic_core::serde_json;

/// Custom view wrapper untuk proyek ini yang otomatis menginjeksi variabel global
/// user_permissions dan user_roles dari session tanpa perlu di-set manual di controller.
pub fn view(
    req: &rustbasic_core::requests::Request,
    template: &str,
    ctx: rustbasic_core::minijinja::Value,
) -> rustbasic_core::router::Response {
    let mut obj = match serde_json::to_value(&ctx) {
        Ok(serde_json::Value::Object(map)) => map,
        _ => serde_json::Map::new(),
    };

    // Baca langsung dari session cache yang super cepat
    let perms = req.session.get::<Vec<String>>("user_permissions").unwrap_or_default();
    let roles = req.session.get::<Vec<String>>("user_roles").unwrap_or_default();
    obj.insert("user_permissions".to_string(), serde_json::json!(perms));
    obj.insert("user_roles".to_string(), serde_json::json!(roles));

    // Lanjutkan ke fungsi render core bawaan
    rustbasic_core::view::view(req, template, rustbasic_core::minijinja::Value::from_serialize(obj))
}
```

---

### 5. Penggunaan di Controller (Sangat Murni & Bersih)
Dengan adanya interseptor di atas, *controller* Anda terbebas sepenuhnya dari polusi pengambilan izin manual. Anda hanya perlu memanggil fungsi `view` dari modul lokal:

```rust
use crate::app::view; // Import fungsi view lokal hasil sekatan di atas
use rustbasic_core::minijinja::context;

pub async fn index(State(state): State<AppState>, req: Request) -> impl IntoResponse {
    // Ambil data bisnis spesifik halaman
    let total_users = users::Model::count(&state.db).await.unwrap_or(0);

    // Langsung render! Daftar kapabilitas diinjeksi secara transparan di balik layar.
    view(&req, "dashboard.rb.html", context! {
        title => "Dashboard Overview",
        total_users => total_users,
    })
}
```

Jika Anda ingin memproteksi rute atau aksi tertentu agar **hanya bisa diakses oleh Role/Permission tertentu**, gunakan pemblokiran `require_role` / `require_permission`:

```rust
pub async fn create_user(State(state): State<AppState>, req: Request) -> impl IntoResponse {
    let checker = PermissionChecker::new(&state.db, &req.session);
    
    // Otomatis menolak dan mengalihkan jika peran tidak sesuai
    if let Err(resp) = checker.require_role("admin", "/dashboard").await {
        return resp;
    }
    
    // Lanjutkan logika penambahan user...
}
```

---


## ⚛️ Integrasi dengan React.js + Inertia.js (SPA)

Jika aplikasi Anda menggunakan arsitektur **React.js + Inertia.js SPA** (menggantikan *engine template* MiniJinja), maka mekanisme interseptor view dan pengecekan di template dialihkan menggunakan **Inertia Shared Props** dan reaktivitas komponen React.

### 1. Membagikan Variabel secara Global via Inertia (Shared Props)
Buka berkas helper Inertia Anda (misalnya di `src/app/inertia.rs`), lalu perbarui fungsi `inertia` untuk secara otomatis menginjeksi kapabilitas pengguna aktif ke dalam data `props` secara global:

```rust
use serde_json::{json, Value};

pub fn inertia(req: &Request, component: &str, mut props: Value) -> Response {
    // 1. Baca data RBAC dari session cache yang super cepat
    let perms = req.session.get::<Vec<String>>("user_permissions").unwrap_or_default();
    let roles = req.session.get::<Vec<String>>("user_roles").unwrap_or_default();
    
    // 2. Suntikkan ke dalam objek props sebagai data 'auth' global
    if let Value::Object(ref mut map) = props {
        let mut auth_obj = serde_json::Map::new();
        if let Some(user_id) = req.session.get::<i32>("user_id") {
            auth_obj.insert("user".to_string(), json!({
                "id": user_id,
                "permissions": perms,
                "roles": roles
            }));
        } else {
            auth_obj.insert("user".to_string(), Value::Null);
        }
        map.insert("auth".to_string(), Value::Object(auth_obj));
    }

    // ... Lanjutkan logika rendering Inertia standar Anda ...
}
```

### 2. Penggunaan di Controller (Inertia SPA Style)
Controller Anda akan merender halaman menggunakan helper `inertia` dan tidak perlu menyertakan parameter izin secara manual:

```rust
use crate::app::inertia;

pub async fn index(State(state): State<AppState>, req: Request) -> impl IntoResponse {
    let total_users = users::Model::count(&state.db).await.unwrap_or(0);

    // Render komponen halaman React "Dashboard"
    inertia(&req, "Dashboard", json!({
        "title": "Dashboard Overview",
        "total_users": total_users
    }))
}
```

### 3. Pengecekan Hak Akses Langsung di Komponen React
Di sisi React, Anda dapat mengakses hak akses pengguna global dari properti halaman (`props`) menggunakan hook `usePage` bawaan dari `@inertiajs/react` secara sangat deklaratif:

```jsx
import React from 'react';
import { usePage, Link } from '@inertiajs/react';

export default function Sidebar() {
  // Ambil data auth yang secara otomatis diinjeksi oleh backend
  const { auth } = usePage().props;
  const user = auth?.user;

  // Helper fungsi pengecekan
  const hasPermission = (permission) => user?.permissions?.includes(permission);
  const hasRole = (role) => user?.roles?.includes(role);

  return (
    <nav className="flex flex-col gap-2 p-4 bg-gray-900 text-white rounded-xl">
      <Link href="/dashboard" className="px-4 py-2 hover:bg-white/10 rounded">
        📊 Dashboard
      </Link>

      {/* 📝 Render Bersyarat Berdasarkan Permission */}
      {hasPermission('edit articles') && (
        <Link href="/articles/create" className="px-4 py-2 hover:bg-white/10 rounded">
          📝 Tulis Artikel
        </Link>
      )}

      {hasPermission('manage users') && (
        <Link href="/users" className="px-4 py-2 hover:bg-white/10 rounded">
          👥 Manajemen User
        </Link>
      )}

      {/* 🔐 Render Bersyarat Berdasarkan Role */}
      {hasRole('admin') && (
        <Link href="/roles-permissions" className="px-4 py-2 bg-indigo-600 hover:bg-indigo-700 rounded text-center font-bold">
          🔐 Otorisasi RBAC
        </Link>
      )}
    </nav>
  );
}
```

---

## 📄 Lisensi

Package ini dirilis di bawah lisensi **MIT**. Anda bebas memodifikasi dan mendistribusikannya untuk kebutuhan komersial maupun sumber terbuka.
