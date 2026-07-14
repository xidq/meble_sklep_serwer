#Budowanie
ARG JWT_SECRET_KEY
ARG DATABASE_URL
ARG FRONTEND_SERVER
ARG PEPPER_KEY
ARG USERS_DB_ENCRYPTION_KEY

# Przypisanie do ENV
ENV JWT_SECRET_KEY=$JWT_SECRET_KEY
ENV DATABASE_URL=$DATABASE_URL
ENV FRONTEND_SERVER=$FRONTEND_SERVER
ENV PEPPER_KEY=$PEPPER_KEY
ENV USERS_DB_ENCRYPTION_KEY=$USERS_DB_ENCRYPTION_KEY
FROM rust:slim AS builder
ENV DEBIAN_FRONTEND=noninteractive
WORKDIR /app

# instal narzędzi systemowych, bibliotek potrzebnych do kompilacji SQLite (np. libsqlite3)
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


#Pobranie i kompilacja libheif 1.21.1 (wgra pliki do /usr/local/lib i /usr/local/include)
RUN git clone --depth 1 --branch v1.21.1 https://github.com/strukturag/libheif.git /tmp/libheif && \
    cd /tmp/libheif && \
    mkdir build && cd build && \
    cmake -DCMAKE_BUILD_TYPE=Release .. && \
    make -j$(nproc) && \
    make install && \
    ldconfig && \
    rm -rf /tmp/libheif


# Kopiujemy całą strukturę projektu (ze względu na workspace)
COPY . .

# Wstrzykujemy flagi kompilacji dla kompilatora
ENV RUSTFLAGS="--cfg docker --check-cfg=cfg(docker)"

# budowanie projektu z flagą profilu 'docker'
RUN cargo build --profile docker

# --- ETAP 2: Uruchomienie (Minimalny, bezpieczny kontener) ---
FROM debian:trixie-slim AS runner
# Kopiujesz skompilowane biblioteki (w tym libheif.so) z etapu buildera
COPY --from=builder /usr/local/lib/ /usr/local/lib/

# Informujesz system, żeby przeładował ścieżki do bibliotek
RUN ldconfig
WORKDIR /app

# Instalujemy tylko runtime dla SQLite oraz certyfikaty SSL (potrzebne np. do reqwesta)
RUN apt-get update && apt-get install -y \
    libsqlite3-0 \
    libaom3 \
    libsharpyuv0 \
#    libavif16 \
#    libheif1 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Kopiujemy gotowy plik binarny z etapu budowania.
# Cargo dla niestandardowych profili umieszcza pliki w target/<nazwa_profilu>/
# (Zakładam, że główny crate w Twoim workspace, który zawiera main.rs, nazywa się "server")
COPY --from=builder /app/target/docker/server /app/server

# Domyślny port (zgodny z fallbackiem w Twoim kodzie)
EXPOSE 8080

# Odpalamy serwer
CMD ["./server"]