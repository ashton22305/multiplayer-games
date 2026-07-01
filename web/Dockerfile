# syntax=docker/dockerfile:1
FROM node:22-slim AS build

RUN apt-get update \
    && apt-get install -y --no-install-recommends curl ca-certificates gcc libc-dev \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --no-modify-path --default-toolchain stable
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /repo

# Cargo workspace metadata — pre-download registry before web source changes.
# rust-toolchain.toml triggers rustup to add wasm32-unknown-unknown on first run.
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY .cargo/ .cargo/
COPY crates/ crates/
RUN --mount=type=cache,target=/root/.cargo/registry \
    cargo fetch --locked

# Node dependencies
COPY web/package.json web/package-lock.json web/
RUN cd web && npm ci

# Web source (most likely to change)
COPY web/ web/

# Compile wasm game bundles then build the Vue SPA
RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/repo/target \
    cd web && npm run build:games && npm run build

FROM nginx:alpine AS runtime
COPY web/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /repo/web/dist /usr/share/nginx/html
EXPOSE 80
