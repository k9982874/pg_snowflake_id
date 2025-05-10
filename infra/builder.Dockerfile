FROM alpine:3.21.3@sha256:a8560b36e8b8210634f77d9f7f9efd7ffa463e380b75e2e74aff4511df3ef88c AS builder

# RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.ustc.edu.cn/g' /etc/apk/repositories

# Allow for overriding rust toolcahin version
ARG RUST_TOOLCHAIN_VERSION=1.85.1
ENV RUST_TOOLCHAIN_VERSION=$RUST_TOOLCHAIN_VERSION

# Allow for overriding of PGRX PG version that is used
ARG PGRX_PG_VERSION=pg17
ENV PGRX_PG_VERSION=$PGRX_PG_VERSION

# Allow overriding features so that this file can be used to build
# different crate features. By default since this is a 17.5 base package
# we expect to build with crate feature 'pg17'
ARG CARGO_FEATURES=pg17
ENV CARGO_FEATURES=$CARGO_FEATURES


ARG CARGO_PGRX_VERSION=0.14.3
ENV CARGO_PGRX_VERSION=$CARGO_PGRX_VERSION

# Install OS deps
RUN apk add --no-cache \
    alpine-sdk \
    bash \
    bison \
    clang \
    clang-dev \
    clang-libs \
    coreutils \
    flex \
    icu-dev \
    linux-headers \
    musl-dev \
    openssl-dev \
    perl \
    readline \
    readline-dev \
    rustup \
    zlib-dev

# Install Rust & related deps
RUN rustup-init -y --profile minimal --default-toolchain $RUST_TOOLCHAIN_VERSION
ENV PATH="/root/.cargo/bin:${PATH}"

# Install pgrx
# (disabling the static C runtime is required since pgrx requires dynamic linking w/ libssl and libcrypto)
RUN RUSTFLAGS="-Ctarget-feature=-crt-static" cargo install --locked cargo-pgrx@${CARGO_PGRX_VERSION}

# Initialize pgrx
ENV PGRX_IGNORE_RUST_VERSIONS=y
RUN cargo pgrx init
