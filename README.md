# REAUTH

A Rust backend powered by [Diesel](https://diesel.rs/) ORM and SQLite.  
This project uses database migrations for schema management and is fully reproducible.

---

## 🛠 Prerequisites

Make sure you have the following installed:

- [Rust](https://www.rust-lang.org/) (via [rustup](https://rustup.rs/))
- [Diesel CLI](https://diesel.rs/guides/getting-started/)  
  Install with SQLite support:
  ```bash
  cargo install diesel_cli --no-default-features --features sqlite
  ```

- SQLite (usually pre-installed on macOS/Linux; check with `sqlite3 --version`)

---

## 📦 Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/your-org/reauth.git
   cd reauth
   ```

2. **Set up environment variables**  
   Copy the example environment file:
   ```bash
   cp .env.example .env
   ```
   Make sure `.env` contains:
   ```env
   DATABASE_URL=reauth.db
   ```

3. **Run database migrations**
   ```bash
   diesel setup
   diesel migration run
   ```

4. **Generate the schema file**
   ```bash
   diesel print-schema > src/schema.rs
   ```

---

## 🚀 Running the Project

Start the app:
```bash
cargo run
```

---

## 🧪 Development Workflow

- **Add a new migration**
  ```bash
  diesel migration generate create_users
  ```

- **Run migrations**
  ```bash
  diesel migration run
  ```

- **Revert last migration**
  ```bash
  diesel migration revert
  ```

- **Regenerate schema.rs**
  ```bash
  diesel print-schema > src/schema.rs
  ```

---

## 📂 Project Structure

```
REAUTH/
├─ Cargo.toml              # Workspace manifest (root, can manage multiple crates)
├─ diesel.toml             # Diesel config (database_url, etc.)
├─ .env.example            # Example environment variables
├─ migrations/             # Database migrations (Diesel-managed)
│   ├─ 2025XXXX_create_users/
│   │   ├─ up.sql
│   │   └─ down.sql
│   └─ ... (future migrations)
│
├─ core/                   # Main Rust backend executable
│   ├─ Cargo.toml
│   └─ src/
│       ├─ main.rs         # bootstrap: load config, logging, DB, start server
│       ├─ lib.rs          # core library (optional, reusable code)
│       │
│       ├─ config/
│       │   ├─ mod.rs      # central config module
│       │   └─ settings.rs # typed config structs, feature toggles
│       │
│       ├─ logging/
│       │   ├─ mod.rs      # centralized logging (tracing)
│       │   └─ banner.rs   # pretty CLI banner
│       │
│       ├─ database/
│       │   ├─ mod.rs
│       │   ├─ connection.rs   # SQLite initialization (Diesel/SQLx)
│       │   ├─ migrate.rs      # migration runner (with safety checks)
│       │   └─ repository.rs   # DB queries & data access
│       │
│       ├─ server/
│       │   ├─ mod.rs
│       │   ├─ routes.rs   # API routes
│       │   ├─ handlers.rs # endpoint handlers
│       │   └─ static.rs   # embedded React UI (serve SPA)
│       │
│       └─ schema.rs       # Diesel schema (auto-generated, don't edit)
│
├─ plugins/                # Dynamic plugins (Wasm + React components)
│   └─ ...
│
├─ ui/                     # React frontend (core + plugin loader)
│   └─ ...
│
└─ scripts/                # Bash scripts (dev, prod, build, migrations, etc.)
    ├─ dev.sh
    ├─ build.sh
    ├─ migrate.sh
    └─ ...

```

---

## 🛑 What Not to Commit

- The SQLite database file (`reauth.db`)
- `.env` (use `.env.example` instead)
- `src/schema.rs` (can always be regenerated)

---

## 👥 Contributing

1. Fork the repo
2. Create your feature branch (`git checkout -b feature/amazing`)
3. Commit changes (`git commit -m "feat: add amazing feature"`)
4. Push to branch (`git push origin feature/amazing`)
5. Open a Pull Request 🚀

---

## 📜 License


