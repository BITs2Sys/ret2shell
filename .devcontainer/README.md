# DevContainer

This directory contains the configuration for the development container. It allows you to develop in a consistent environment across different machines.

The devcontainer will automatically execute the [00-post-start.sh](./00-post-start.sh) script after the container is started.

## Environment

The devcontainer has the following applications pre-installed:

- Rust
- Node.js
- Git
- Docker
- LLDB
- K3s

Some devtools are also pre-installed into `/usr/local/bin`, see [devtools](./devtools) for details.

## Fast development

To fast setup the back-end development environment (including database, cache, etc.), you can run the following command in the terminal:

```bash
./.devcontainer/up-dev.sh
```

It will start the [docker-compose.dev.yml](../deploy/compose/docker-compose.dev.yml) file.
