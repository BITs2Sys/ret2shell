# [Choice] Debian OS version (use bullseye on local arm64/Apple Silicon): buster, bullseye
ARG VARIANT="bookworm"
FROM rust:1-${VARIANT}

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive && \
    apt-get purge -y imagemagick imagemagick-6-common

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive && \
    apt-get -y install --no-install-recommends \
    ca-certificates curl wget \
    musl-dev musl-tools \
    libc6-dev g++ gcc gdb make \
    zip unzip nginx

RUN rustup component add clippy
RUN rustup toolchain install stable nightly
RUN rustup default stable
