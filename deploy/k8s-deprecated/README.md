# Kubernetes deployment for small clusters

This directory contains the Kubernetes deployment files for the application.

We use nodePort with external nginx to proxy streams.

## Install Nginx

**ON YOUR SERVER**

use your package manager to install nginx and setup it on your server.

## Install k3s

**ON YOUR SERVER**

If you want to host ret2shell in a single-node cluster, it's recommended to enlarge the max-pod-limit which is 110 by default.

create a file on `/etc/rancher/k3s/kubelet.config`:

```yaml
apiVersion: kubelet.config.k8s.io/v1beta1
kind: KubeletConfiguration
maxPods: 32600
```

and then install k3s using the following command:

```
curl -sfL https://get.k3s.io | INSTALL_K3S_EXEC="--kube-controller-manager-arg=node-cidr-mask-size=16 --kubelet-arg=config=/etc/rancher/k3s/kubelet.config" sh -
```

### For users in China Mainland

```
curl -sfL https://rancher-mirror.rancher.cn/k3s/k3s-install.sh | INSTALL_K3S_MIRROR=cn INSTALL_K3S_EXEC="--kube-controller-manager-arg=node-cidr-mask-size=16 --kubelet-arg=config=/etc/rancher/k3s/kubelet.config" sh -
```

You can setup more nodes following documents here: https://docs.k3s.io/quick-start

## Prepare config files

**UPLOAD TO YOUR SERVER**

You need prepare 3 files to launch ret2shell, templates could be found at ![here](../../config/):

```bash
config.toml # the config file.
blocked.txt # the sensitive word list, leave it empty if you do not need it
license     # the license
```

## Setup license and compile your image

**IN PROJECT ROOT DIRECTORY**

The license currently not restrict anything, you can generate it by yourself:

```bash
cargo run --bin r2s-license -- init -p ./config/
cargo run --bin r2s-license -- new --ca ./config/priv.bin --path ./config/ --issuer <Your Org> --website <Domain> --date 2077-01-01 --level enterprise
```

The ret2shell image needs `priv.bin` to compile, after setup these things, run:

```
./release-image.sh
# or you want to use fish
./release-image.fish
```

to build your OCI image.

## Apply deployments

**ON YOUR SERVER**

Open each deployment file in this folder and change values, here are some important value you should be care with:

1. location of the persist volume / storage class: open `1-volumes.yaml`, you can find the size and position for each storage class,
   you need to change node position from `ret2shell-node-master` to your master node hostname, you can see it with `kubectl get nodes`.
2. storage size in each components: search `storageClass` or `storage-class` in each components' `values.yaml`, and change the size to
   fixup `1-volumes.yaml`.
3. change auth methods / passwords: setup passwords for `2-cache`, `3-database`, `4-queue`, `6-logs`, it's highly recommended to choose
   a password from `uuidgen` or other random password generators.
4. ensure the server configuration `config.toml` is ok with your previous setup.

Deploy these files with number order:

```bash
kubectl apply -f 0-init.yaml
kubectl apply -f 1-volumes.yaml
helm install ret2shell-cache ./2-cache -n ret2shell-platform
helm install ret2shell-database ./3-database -n ret2shell-platform
helm install ret2shell-queue ./4-queue -n ret2shell-platform
helm install ret2shell-registry ./5-registry -n ret2shell-platform # Optional, ret2shell could use external registries
helm install ret2shell-logs ./6-logs -n ret2shell-platform         # Optional, this enables log metrics for ret2shell
```

before deploying `7-platform.yaml`, we should mount configmaps first:

```
kubectl -n ret2shell-platform create configmap ret2shell-license --from-file /path/to/license
kubectl -n ret2shell-platform create configmap ret2shell-blocked --from-file /path/to/blocked.txt
kubectl -n ret2shell-platform create configmap ret2shell-config --from-file /path/to/config.toml
```

then, upload your ret2shell OCI image to private registry and deploy it.

if you use internal registry deployed by `5-registry`, change the `image` field in `7-platform.yaml` to `localhost:30310/ret2shell:latest`,
then use the following command to push your OCI image:

```bash
# on your build machine
docker save ret2shell:latest -o ret2shell.tar
rsync -avzhP ret2shell.tar YourServer:
# on server
skopeo copy docker-archive:ret2shell.tar docker://localhost:30310/ret2shell:latest --dest-tls-verify=false
```

deploy!

```bash
kubectl apply -f 7-platform.yaml
```

## Setup nginx and frontend

we have already deployed ret2shell's backend in k8s cluster, now build the frontend:

```bash
cd web && pnpm build
```

upload `web/dist` folder to your server, use [this template file](../nginx/ret2shell.conf) to setup it, the default nodePort for backend is `30307`,
you can change it with your favor.

## Other instr

### Update the server & change configuration

rebuild the OCI image and push it to registry, change config file by:

```bash
kubectl -n ret2shell-platform edit configmaps ret2shell-config
```

then perform a rollout deployment:

```bash
kubectl -n ret2shell-platform rollout restart deployment ret2shell-platform
```

### Update components like cache or database

update or change something in `*-component` folder, then:

(use database for example)

```bash
helm upgrade ret2shell-database ./3-database -n ret2shell-platform
# DO NOT FORGET RE_DEPLOY PLATFORM AFTER ANY DEPENDENCY UPDATES
kubectl -n ret2shell-platform rollout restart deployment ret2shell-platform
```

### Manage nodes

you can add nodes to k8s cluster for higher capacities, use `ret.sh.cn/workload` to label nodes:

```bash
kubectl label nodes ret2shell-worker-1 ret.sh.cn/workload=challenge
```

the `challenge` could be changed with your needs. after label the nodes, you can login to the platform,
and find workload label configurations in platform config / game config page, just change it,
and all the dispatch features will work as expect.
