# --- ETAP 1: Budowanie ---
FROM rust:slim AS builder
ENV DEBIAN_FRONTEND=noninteractive
WORKDIR /app

# Instalacja narzędzi systemowych i bibliotek potrzebnych do kompilacji
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    gcc \
    g++ \
    clang \
    cmake \
    git \
    libsqlite3-dev \
    libavif-dev \
    libheif-dev \
    && rm -rf /var/lib/apt/lists/*


RUN git clone --depth 1 --branch v1.21.1 https://github.com/strukturag/libheif.git /tmp/libheif && \
    cd /tmp/libheif && \
    mkdir build && cd build && \
    cmake -DCMAKE_BUILD_TYPE=Release .. && \
    make -j$(nproc) && \
    make install && \
    ldconfig && \
    rm -rf /tmp/libheif

# Kopiowana cała struktura projektu (ze względu na workspace)
COPY . .

# wstrzykiwanie flag kompilatorowi
ENV RUSTFLAGS="--cfg docker --check-cfg=cfg(docker)"
ENV SQLX_OFFLINE=true

RUN cargo build --profile docker

# Minimalny, bezpieczny kontener
FROM debian:trixie-slim AS runner

# kopiowanie skompilowanych libsów (w tym libheif.so) z etapu buildera
COPY --from=builder /usr/local/lib/ /usr/local/lib/
RUN ldconfig
WORKDIR /app

# instal runtime dla SQLite oraz certyfikaty SSL
RUN apt-get update && apt-get install -y \
    libsqlite3-0 \
    libaom3 \
    libsharpyuv0 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Kopiujemy gotowy plik binarny
COPY --from=builder /app/target/docker/server /app/server

ARG CURRENT_RUST_SERVER_PORT=8080
ENV CURRENT_RUST_SERVER_PORT=${CURRENT_RUST_SERVER_PORT}

EXPOSE ${CURRENT_RUST_SERVER_PORT}

CMD ["./server"]