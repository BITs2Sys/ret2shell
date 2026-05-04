FROM golang:alpine AS inspector

RUN apk add --no-cache git ca-certificates tzdata


WORKDIR /build

COPY ./cloud/node-inspecter/go.mod ./cloud/node-inspecter/go.sum ./

RUN go mod download

COPY ./cloud/node-inspecter/main.go ./

RUN CGO_ENABLED=0 GOOS=linux go build \
    -a -installsuffix cgo \
    -ldflags='-w -s -extldflags "-static"' \
    -o node-inspecter ./main.go && \
    cp node-inspecter /usr/local/bin/node-inspecter && \
    chmod +x /usr/local/bin/node-inspecter

# --------------------------------------------------------------------------------------------------------

FROM rust:1.94-alpine AS server

# hadolint ignore=DL3018
RUN apk add --update --no-cache musl-dev clang lld ca-certificates

RUN update-ca-certificates

COPY ./.cargo/config.toml /var/lib/ret2shell/.cargo/config.toml
COPY ./Cargo.toml /var/lib/ret2shell/Cargo.toml
COPY ./LICENSE /var/lib/ret2shell/LICENSE
COPY ./crates /var/lib/ret2shell/crates
WORKDIR /var/lib/ret2shell

ARG R2S_GIT_VERSION=DEADBEEF
ENV R2S_GIT_VERSION=${R2S_GIT_VERSION}

RUN --mount=type=cache,target=/var/lib/ret2shell/target cargo update && \
    cargo build --release --bin r2s-server --target x86_64-unknown-linux-musl && \
    cp /var/lib/ret2shell/target/x86_64-unknown-linux-musl/release/r2s-server /usr/local/bin/r2s-server

# FROM debian:bookworm-slim AS gke-bin

# RUN apt-get update && \
#     apt-get install -y apt-transport-https ca-certificates gnupg curl && \
#     curl https://packages.cloud.google.com/apt/doc/apt-key.gpg | gpg --dearmor -o /usr/share/keyrings/cloud.google.gpg && \
#     echo "deb [signed-by=/usr/share/keyrings/cloud.google.gpg] https://packages.cloud.google.com/apt cloud-sdk main" | tee -a /etc/apt/sources.list.d/google-cloud-sdk.list && \
#     apt-get update && \
#     apt-get install -y google-cloud-sdk-gke-gcloud-auth-plugin && \
#     apt-get clean && \
#     rm -rf /var/lib/apt/lists/*

# RUN mkdir -p /gke-auth && \
#     cp "$(realpath $(which gke-gcloud-auth-plugin))" -t /gke-auth/

FROM node:lts-alpine AS frontend

ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable

COPY ./web/package.json ./web/pnpm-lock.yaml /var/lib/ret2shell/web/
WORKDIR /var/lib/ret2shell/web
RUN --mount=type=cache,id=pnpm,target=/pnpm/store \
    pnpm install --frozen-lockfile

COPY ./web /var/lib/ret2shell/web
RUN pnpm build && \
    mkdir -p /var/www/html && \
    cp -r ./dist/* /var/www/html/

FROM alpine:3

# hadolint ignore=DL3018
RUN apk add --update --no-cache curl git skopeo tini && \
    git config --global user.email ctf@cumt.edu.cn && \
    git config --global user.name BXSCTF

COPY --from=server /etc/ssl/certs/ /etc/ssl/certs/

# COPY --from=gke-bin /gke-auth/gke-gcloud-auth-plugin /bin/gke-gcloud-auth-plugin
# COPY ./cloud/gke-gcloud-auth-plugin-proxy /bin/gke-gcloud-auth-plugin-proxy
# RUN chmod +x /bin/gke-gcloud-auth-plugin /bin/gke-gcloud-auth-plugin-proxy

# COPY --from=inspector /usr/local/bin/node-inspecter /bin/node-inspecter

COPY --from=server /usr/local/bin/r2s-server /bin/r2s-server
COPY --from=frontend /var/www/html /var/www/html

# RUN echo '#!/bin/sh' >> /bin/r2s-entrypoint && \
#     echo '/bin/node-inspecter &' >> /bin/r2s-entrypoint && \
#     echo 'exec /bin/r2s-server "$@"' >> /bin/r2s-entrypoint && \
#     chmod +x /bin/r2s-entrypoint

RUN mkdir -p \
    /var/log/ret2shell \
    /var/cache/ret2shell \
    /var/lib/ret2shell

# if you changes the server port in deployment, maybe you should request for a new distribution
HEALTHCHECK --interval=5m --timeout=3s --start-period=10s --retries=1 \
    CMD curl -fsSL http://localhost:8080/api/ping || exit 1

ENTRYPOINT ["/sbin/tini", "--", "/bin/r2s-server"]
