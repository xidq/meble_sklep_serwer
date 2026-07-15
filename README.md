# Rust Backend Server (Axum & SQLite)

A high-performance, modular backend service written in Rust. It acts as the final, authoritative layer in a three-tier architecture:

**Frontend (JS) → Middle Server (Go) → Backend (this Rust server)**

The server is responsible for core business logic, secure data persistence, final authorization, and heavy image processing.

---

## Architecture & Key Capabilities

- **HTTP & WebSocket Engine** – Built on Axum 0.8 and Tokio. Provides REST endpoints and real-time WebSocket communication.
- **Three-Tier Authorization** – Enforces the final, most granular security and business-rule checks, serving as the ultimate access-control point behind the JS frontend and the Go intermediary.
- **Dual SQLite Storage**  
  - `rusqlite` (with `bundled-sqlcipher`) handles encrypted database operations.  
  - `sqlx` is used for asynchronous queries and migrations on non-encrypted datasets.
- **Zero-Copy File Streaming** – Axum’s native `multipart` and `Stream` support (via `futures-util`) allows uploading and downloading large files with minimal memory footprint.
- **Modern Image Processing** – A dedicated `avif_image_handler` workspace decodes HEIC/HEIF images and converts them to optimized AVIF or WebP formats using `libheif-rs`, `image`, `webp`, and `png`.
- **Security** – JSON Web Token authentication (`jsonwebtoken`), password hashing (`bcrypt`), and request rate-limiting (`tower_governor`).
- **Roadmap** – Automated invoice generation and an email notification system are under development.

---

## Project Structure (Cargo Workspace)

The codebase is split into focused sub-crates to speed up compilation and enforce clear boundaries (currently temporary view of subcrates):

| Crate                   | Purpose                                                       |
|-------------------------|---------------------------------------------------------------|
| `src/server`            | Application entry point, Axum router, and middleware.         |
| `src/sqlite_serv`       | Database access layer (`rusqlite` + `sqlx`).                  |
| `src/avif_image_handler`| Image decoding and AVIF/WebP conversion.                      |
| `src/auth`, `src/login` | --                                                            |
| `src/models`            | --                                                            |
| `src/id_handling`       | --                                                            |

---

## System Dependencies

Building the project requires several native libraries for image codecs and SQLite encryption.

### 1. Required Packages (Debian / Ubuntu)

```bash
sudo apt-get update && sudo apt-get install -y \
    pkg-config \
    libssl-dev \
    gcc g++ clang cmake git \
    libsqlite3-dev \
    libwebp-dev \
    libde265-dev libx265-dev libaom-dev \
    libjpeg-dev \
    libavif-dev
```

### 2. Compile `libheif` (v1.21.1) from Source

The system `libheif-dev` is often outdated. A custom build with all required codecs is necessary:

```bash
git clone --depth 1 --branch v1.21.1 https://github.com/strukturag/libheif.git /tmp/libheif
cd /tmp/libheif
mkdir build && cd build
cmake -DCMAKE_BUILD_TYPE=Release \
      -DWITH_LIBDE265=ON \
      -DWITH_X265=ON \
      -DWITH_AOM_DECODER=ON \
      -DWITH_AOM_ENCODER=ON \
      ..
make -j$(nproc)
sudo make install
sudo ldconfig
rm -rf /tmp/libheif
```

---

## Build & Run

Custom profiles in `Cargo.toml` control optimization levels:

- **Local development** – fast unoptimized builds for rapid iteration.
- **`release`** – baseline optimizations, panic abort, stripped symbols.
- **`docker`** – inherits from `release` but there's some change for docker.

### Development Server

```bash
cargo run -p server
```

### Production Build
# without docker
```bash
cargo build -r
```
# for docker
```bash
cargo build --profile docker
```

---

## Environment Variables

A `.env` file in the project root is required. Example:

```env
DATABASE_URL=sqlite://data/data.db
JWT_SECRET_KEY=your_secret_key_at_least_32_bytes
FILES_URL=src/api/
FRONTEND_SERVER=http://localhost:3000
PEPPER_KEY=your_password_hash_pepper
USERS_DB_ENCRYPTION_KEY=your_sqlcipher_db_key
```

Adjust the values according for deployment.

---

## Docker Deployment

The service is packaged using a two-stage Docker build:

1. **Build stage** – Full Rust toolchain, system libraries, and a custom-compiled `libheif`. The binary is built with `cargo build --profile docker`.
2. **Runtime stage** – A minimal `debian:bookworm-slim` image containing only the compiled server binary, the `.env` file, and the necessary dynamic libraries (`libheif.so*`, `libde265.so`, `libx265.so`, etc.).

This approach results in a small, secure production image.
