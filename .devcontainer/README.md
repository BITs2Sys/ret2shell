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
- Kubectl, Helm, Minikube (for Kubernetes development)

Some devtools are also pre-installed into `/usr/local/bin`, see [devtools](./devtools) for details.

> [!NOTE]
>
> The devcontainer won't start the kubernetes cluster. You need to connect to an existing cluster.
>
> Once you configured your kubectl to connect to a cluster, you can run the following command to generate the kubeconfig file and patch the ret2shell config file:
>
> ```bash
> ./.devcontainer/apply-kubeconfig.sh
> ```

## Fast development

To fast setup the back-end development environment (including database, cache, etc.), you can run the following command in the terminal:

```bash
./.devcontainer/up-dev.sh
```

It will start the [docker-compose.dev.yml](../deploy/compose/docker-compose.dev.yml) file via Docker Compose.
