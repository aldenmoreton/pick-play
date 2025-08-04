ARG APP_NAME=pick-play

# Chef stage - install cargo-chef with nightly support
FROM rust:1.83-alpine AS chef
RUN apk add --no-cache clang lld musl-dev git pkgconfig openssl-dev openssl-libs-static
RUN rustup toolchain install nightly
RUN cargo install cargo-chef --locked
WORKDIR /app

# Planner stage - generate recipe.json
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Builder stage - cache dependencies and build application
FROM chef AS builder
ARG APP_NAME

# Copy recipe and build dependencies (this layer will be cached)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo +nightly chef cook --release --recipe-path recipe.json --no-default-features

# Copy source code and build application
COPY . .
RUN cargo +nightly build --locked --release --no-default-features --bin $APP_NAME && \
    cp ./target/release/$APP_NAME /bin/server

FROM node:18.13.0 AS styles
WORKDIR /app

# Copy package files and install dependencies
COPY package.json .
RUN npm install

# Copy source files needed for Tailwind
COPY tailwind.config.js .
COPY style/ ./style/
COPY pick-play/src/ ./src/

# Generate CSS
RUN npx tailwindcss -i style/input.css -o /bookie.css

FROM alpine:3.18 AS final

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

# Copy the public folder
COPY ./public /public

# Copy the executable from the "builder" stage.
COPY --from=builder /bin/server /bin/server/

# Copy styles
COPY --from=styles /bookie.css /public/styles/bookie.css

EXPOSE 8000

CMD ["/bin/server/server"]
