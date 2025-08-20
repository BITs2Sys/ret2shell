FROM golang:alpine AS builder

RUN apk add --no-cache git ca-certificates tzdata


WORKDIR /build

COPY go.mod go.sum ./

RUN go mod download

COPY main.go ./

RUN CGO_ENABLED=0 GOOS=linux go build \
    -a -installsuffix cgo \
    -ldflags='-w -s -extldflags "-static"' \
    -o node-inspecter ./main.go && \
    cp node-inspecter /usr/local/bin/node-inspecter && \
    chmod +x /usr/local/bin/node-inspecter

FROM scratch

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /usr/share/zoneinfo /usr/share/zoneinfo
COPY --from=builder /etc/passwd /etc/passwd

COPY --from=builder /usr/local/bin/node-inspecter /bin/node-inspecter

EXPOSE 6464

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/bin/node-inspecter", "-health-check"] || exit 1

ENTRYPOINT ["/bin/node-inspecter"]