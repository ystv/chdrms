# syntax=docker/dockerfile:1
FROM rust:alpine AS backend-base

ENV RUSTFLAGS="-C link-arg=-fuse-ld=mold"
WORKDIR /app

RUN apk add --no-cache mold curl

RUN rustup show && \
    cargo install cargo-chef --locked

# ---
FROM backend-base AS backend-planner

COPY Cargo.toml Cargo.lock ./
COPY backend ./backend

RUN cargo chef prepare --recipe-path recipe.json

# ---
FROM backend-base AS backend-builder

COPY --from=backend-planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json --bin chdrms_server

COPY Cargo.toml Cargo.lock ./
COPY backend ./backend
COPY .sqlx ./.sqlx

RUN cargo build --release --bin chdrms_server

# ---
FROM node:alpine AS frontend-base

WORKDIR /app

RUN npm install -g corepack@latest && \
    corepack enable

# ---
FROM frontend-base AS frontend-builder

COPY ui/package.json ui/yarn.lock ./
RUN yarn install --immutable

COPY ui ./
RUN yarn build

# ---
FROM scratch

COPY --from=backend-builder /app/target/release/chdrms_server /backend
COPY --from=frontend-builder /app/dist /dist

ENV ENVIRONMENT=PRODUCTION

EXPOSE 3000

ENTRYPOINT [ "/backend" ]
