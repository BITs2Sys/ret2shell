FROM alpine:3

RUN apk add --no-cache \
    bash \
    curl \
    git \
    skopeo

RUN git config --global user.email platform@ret.sh.cn
RUN git config --global user.name Ret2Shell
COPY target/x86_64-unknown-linux-musl/release/r2s-server /bin/r2s-server
RUN chmod +x /bin/r2s-server

ENTRYPOINT ["/bin/r2s-server"]
