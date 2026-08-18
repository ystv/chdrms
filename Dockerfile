# syntax=docker/dockerfile:1
FROM rust:alpine AS backend-base

ENV RUSTFLAGS="-C link-arg=-fuse-ld=mold"
WORKDIR /app

RUN apk add --no-cache mold curl

RUN rustup show && \
    cargo install cargo-chef --locked

COPY .cargo ./

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
FROM node:24-alpine AS frontend-builder

WORKDIR /app

COPY ui/package.json ui/yarn.lock ui/.yarnrc.yml ./
COPY ui/.yarn ./.yarn
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
