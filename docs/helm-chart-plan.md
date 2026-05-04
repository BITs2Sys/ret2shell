# Ret2Shell Helm Chart 编写计划

## 1. 目标

在 `deploy/helm/ret2shell` 下实现一个自维护的 Helm Chart，用于部署以下组件：

- `ret2shell` 平台本体，使用项目自己构建的镜像
- `PostgreSQL`，使用官方 `postgres` 镜像
- `Valkey`，使用官方 `valkey/valkey` 镜像
- `NATS`，使用官方 `nats` 镜像并启用 JetStream
- `Docker Registry`，使用官方 `registry` 镜像
- `VictoriaLogs`，使用官方 `victoriametrics/victoria-logs` 镜像

Chart 需要同时满足以下要求：

- 默认可以一键部署完整内置依赖
- 每个依赖组件都可以切换为外部服务
- `ret2shell` 平台支持 `Ingress` 和 `NodePort` 两种暴露模式
- 平台 namespace 固定为 `ret2shell-platform`
- challenge namespace 固定为 `ret2shell-challenge`
- 保留当前集群调度能力，包括 challenge namespace、service account、cluster role、cluster role binding 等

## 2. 现状与约束

### 2.1 现有部署拓扑

当前推荐运行拓扑已经体现在 `deploy/compose/docker-compose.yml`：

- `ret2shell` 平台依赖 `postgres 18+`、`valkey 8+`、`nats 2+`
- `registry` 和 `VictoriaLogs` 为可选组件
- 平台镜像自身已经包含前端静态资源，不必额外再配一套前端容器或 nginx

对应代码与配置来源：

- `deploy/compose/docker-compose.yml`
- `config/config.sample.toml`
- `Containerfile`
- `crates/server/src/routes/web/mod.rs`

### 2.2 平台本体是单实例有状态服务

虽然 `ret2shell` 平台本身不是数据库，但当前实现并不是无状态 Web 服务：

- `bucket.path` 下保存 game/challenge 的 Git 仓库
- `media.path` 下保存上传后的媒体文件
- Web 端修改题目、附件、checker、env 时会直接写本地文件并提交 Git
- 当前并发保护主要依赖仓库目录下的本地 `.lock` 文件

这意味着：

- 平台在 Helm v1 中必须按 `singleton` 设计
- 不支持 `replicaCount > 1`
- 需要为平台准备持久化卷
- 推荐使用 `StatefulSet(1)`，如后续要兼容旧行为，也可以退回 `Deployment + Recreate`

### 2.3 集群权限不是普通 namespace 级别

`ret2shell` 平台需要在 Kubernetes 中动态管理 challenge 相关资源。当前旧部署通过 `deploy/k8s-deprecated/0-init.yaml` 创建：

- `ret2shell-platform` namespace 下的 `ServiceAccount`
- 一个 cluster-scoped `ClusterRole`
- 一个 `ClusterRoleBinding`
- `ret2shell-challenge` namespace

结合 `crates/cluster/src/lib.rs` 和 `crates/cluster/src/manager.rs`，可以确认当前平台需要：

- 使用 in-cluster service account 访问 Kubernetes API
- 读取节点信息
- 管理 challenge namespace 下的 Pod、Service、ConfigMap、NetworkPolicy 等资源
- 自动检查或创建 `ret2shell-challenge` namespace

因此 Helm Chart 不能只做 namespaced Role/RoleBinding，而应明确支持：

- 创建 `ServiceAccount`
- 创建 `ClusterRole`
- 创建 `ClusterRoleBinding`
- 复用已有的 `ServiceAccount` 和外部预置 RBAC

## 3. 设计原则

- 使用单一 umbrella chart，不依赖远程子 chart
- 所有依赖组件都使用官方镜像，但模板由本仓库维护
- 所有连接信息统一由 Helm 生成到 `config.toml`
- 所有 Secret 都尽量通过 `Secret` 挂载，不再沿用旧方案中把敏感内容放在 `ConfigMap` 的方式
- 默认安装走“内部依赖”模式，但允许逐项切换为外部服务
- 平台、数据库、缓存、队列、日志、镜像仓库都支持单独配置存储类、容量、资源和调度策略

## 4. Chart 目录规划

建议目录结构如下：

```text
deploy/helm/ret2shell/
  Chart.yaml
  values.yaml
  values.schema.json
  README.md
  templates/
    _helpers.tpl
    _config.tpl
    _connections.tpl
    challenge-namespace.yaml
    challenge-networkpolicy.yaml
    platform/
      serviceaccount.yaml
      clusterrole.yaml
      clusterrolebinding.yaml
      secret-config.yaml
      configmap-blocked.yaml
      service-clusterip.yaml
      service-nodeport.yaml
      ingress.yaml
      statefulset.yaml
      pvc.yaml
    postgresql/
      secret.yaml
      service.yaml
      statefulset.yaml
    valkey/
      secret.yaml
      configmap.yaml
      service.yaml
      statefulset.yaml
    nats/
      secret.yaml
      configmap.yaml
      service.yaml
      statefulset.yaml
    registry/
      configmap.yaml
      secret.yaml
      deployment.yaml
      service-internal.yaml
      service-external.yaml
      pvc.yaml
    victoria-logs/
      service.yaml
      statefulset.yaml
    tests/
      smoke-ping.yaml
      smoke-registry.yaml
      smoke-vlogs.yaml
    NOTES.txt
  examples/
    values-ingress-internal.yaml
    values-nodeport-internal.yaml
    values-ingress-external.yaml
    values-nodeport-external.yaml
```

## 5. 工作负载规划

### 5.1 platform

- 工作负载类型：`StatefulSet`
- 副本数：固定 `1`
- 镜像：项目自行构建的 `ret2shell` 镜像
- 数据卷：至少一个平台数据卷，挂载到 `/var/lib/ret2shell`
- 配置挂载：
  - `/etc/ret2shell/config.toml`
  - `/etc/ret2shell/blocked.txt`
- 健康检查：`/api/ping`

说明：

- 平台镜像已经在 `Containerfile` 中内置前端静态资源
- 后端启动时会初始化数据库、缓存、bucket、queue、cluster、media 等模块
- `NodePort` 与 `Ingress` 都只需要指向这一套平台 Service

### 5.2 postgresql

- 工作负载类型：`StatefulSet`
- 副本数：`1`
- 镜像：`postgres:18-alpine`
- 数据目录：`/var/lib/postgresql/data`
- 探针：`pg_isready`
- 默认启用持久化卷

首版不做：

- 主从复制
- Patroni / failover
- 备份 sidecar
- metrics exporter

### 5.3 valkey

- 工作负载类型：`StatefulSet`
- 副本数：`1`
- 镜像：`valkey/valkey:8-alpine`
- 数据目录：`/data`
- 默认启用 AOF
- 默认启用持久化卷

首版不做：

- sentinel
- replication
- cluster mode

### 5.4 nats

- 工作负载类型：`StatefulSet`
- 副本数：`1`
- 镜像：`nats:2-alpine`
- 默认启用 JetStream
- 默认启用 file store 和 PVC
- NATS 服务端口使用 `4222`

首版不做：

- clustering
- leafnodes
- websocket
- mqtt

### 5.5 registry

- 工作负载类型：`Deployment`
- 副本数：`1`
- 更新策略：`Recreate`
- 镜像：`registry:3`
- 默认启用 filesystem 存储和 PVC
- 默认开启 delete capability

说明：

- registry 需要同时考虑平台推送与节点拉取两条访问路径
- 平台推送推荐走内部 `ClusterIP`
- 节点拉取通常需要一个节点可达地址，因此还要支持外部 `NodePort` 或用户手动给出外部地址

### 5.6 victoriaLogs

- 工作负载类型：`StatefulSet`
- 副本数：`1`
- 镜像：`victoriametrics/victoria-logs:v1.43.1`
- 数据目录：`/vlogs`
- 默认启用 PVC

## 6. values 结构规划

建议顶层 values 结构如下：

```yaml
global:
  imagePullSecrets: []
  storageClass: ""
  labels: {}
  annotations: {}

platform:
  image:
    repository: ""
    tag: ""
    digest: ""
    pullPolicy: IfNotPresent
  replicaCount: 1
  exposure:
    mode: ingress # ingress | nodePort
  service:
    port: 8080
    nodePort: 30307
  ingress:
    enabled: true
    className: nginx
    hosts: []
    tls: []
  persistence:
    enabled: true
    size: 420Gi
    storageClass: ""
    existingClaim: ""
  serviceAccount:
    create: true
    name: ""
    annotations: {}
    automountServiceAccountToken: true
  rbac:
    create: true
    clusterRole:
      create: true
      name: ""
    clusterRoleBinding:
      create: true
      name: ""
  config:
    existingSecret: ""
    generate: true
  blocked:
    existingConfigMap: ""

challengeNamespace:
  create: true
  name: ret2shell-challenge

postgresql:
  mode: internal # internal | external

valkey:
  mode: internal # internal | external

nats:
  mode: internal # internal | external

registry:
  mode: internal # disabled | internal | external

victoriaLogs:
  mode: internal # disabled | internal | external
```

每个组件都需要拆出两套参数：

- `internal` 模式参数：镜像、资源、持久化、Service、探针、调度
- `external` 模式参数：地址、端口、用户名、密码、token、TLS、现有 Secret 引用

`values.schema.json` 需要覆盖至少以下规则：

- `platform.replicaCount` 只能为 `1`
- `challengeNamespace.name` 首版应固定为 `ret2shell-challenge`，直到代码支持配置化
- `platform.exposure.mode=ingress` 时必须提供 host
- `*.mode=external` 时必须提供最小连接信息
- `registry.mode=internal` 时必须提供节点可达的 `external` 地址配置
- `platform.serviceAccount.create=false` 时必须给出 `platform.serviceAccount.name`

## 7. ServiceAccount 与 RBAC 方案

这是本 Chart 的关键设计点。

### 7.1 默认行为

默认开启以下资源创建：

- `platform.serviceAccount.create=true`
- `platform.rbac.create=true`
- `platform.rbac.clusterRole.create=true`
- `platform.rbac.clusterRoleBinding.create=true`

也就是说，默认安装会由 Chart 创建：

- 一个平台 `ServiceAccount`
- 一个 cluster-scoped `ClusterRole`
- 一个 `ClusterRoleBinding`

因此默认安装账号必须具备 cluster-scoped 资源创建权限；仅有 namespace admin 权限通常不足以完成安装。

权限边界应先以 `deploy/k8s-deprecated/0-init.yaml` 为兼容基线，保证当前 challenge 调度、service 管理、namespace 检查、network policy 管理等行为不回退。

### 7.2 兼容企业集群的做法

很多集群不允许业务 chart 直接创建 cluster-scoped RBAC，因此必须提供“复用已有账号”的能力：

- `platform.serviceAccount.create=false`
- `platform.serviceAccount.name=<existing-sa>`
- `platform.rbac.create=false`

此时文档需要明确要求运维侧提前准备：

- `ServiceAccount`
- `ClusterRole`
- `ClusterRoleBinding`
- 所需 namespace

### 7.3 namespace 规划

- Helm release 建议安装在 `ret2shell-platform` namespace
- `ret2shell-platform` namespace 建议由 `helm install --create-namespace` 或运维预先创建，不建议依赖 chart 自身去创建 release namespace
- `ret2shell-challenge` namespace 仍建议由 Chart 显式创建
- 虽然程序本身也会尝试自动创建 challenge namespace，但 Chart 侧仍应声明它，便于权限与资源可视化

### 7.4 权限实现原则

v1 先采用“与现状兼容”的权限策略，不在首版尝试激进裁剪权限；等 Helm Chart 落地并验证行为后，再单独做最小权限收敛。

## 8. 配置、Secret 与连接生成策略

### 8.1 config.toml

`config.toml` 由 Chart 生成到 `Secret` 中，除非用户显式指定 `platform.config.existingSecret`。

生成内容包括：

- 数据库连接
- Valkey 连接
- NATS 连接
- logging.victoria
- cluster.registry
- server 配置
- bucket/media/logging/capture 的路径

默认路径建议统一到平台数据卷下：

- `bucket.path = /var/lib/ret2shell/bucket`
- `media.path = /var/lib/ret2shell/media`
- `logging.directory = /var/lib/ret2shell/log`
- `cluster.capture_directory = /var/lib/ret2shell/captures`

### 8.2 blocked.txt

- `blocked.txt` 使用单独 `ConfigMap`

支持两种用法：

- Chart 直接创建
- 引用现有资源

### 8.3 内外部服务切换

在 helper 中统一实现连接生成逻辑：

- `postgresql.mode=internal` 时，生成 chart 内部 service DNS
- `postgresql.mode=external` 时，读取用户给出的外部 host/port/user/password
- 其余组件同理

### 8.4 registry 双地址问题

`cluster.registry` 需要同时处理两个地址：

- `server`：平台向 registry 推送镜像时使用
- `external`：Kubernetes 节点拉取 challenge 镜像时使用

因此内部 registry 模式下需要同时提供：

- 一个内部服务地址给平台容器使用
- 一个节点可达地址给 challenge workload 使用

建议 values 中显式区分：

- `registry.internalService.*`
- `registry.externalAccess.*`

## 9. 网络暴露方案

### 9.1 ret2shell 平台

平台支持两种模式：

- `Ingress` 模式
- `NodePort` 模式

实现建议：

- 永远创建一个内部 `ClusterIP` Service
- `Ingress` 模式下额外创建 `Ingress`
- `NodePort` 模式下额外创建一个 `NodePort` Service

不建议让 `Ingress` 与 `NodePort` 互相复用同一个 Service 类型，避免 values 逻辑过于混乱。

### 9.2 其他组件

- `PostgreSQL`、`Valkey`、`NATS` 默认仅内部暴露
- `VictoriaLogs` 默认仅内部暴露
- `registry` 在 `internal` 模式下需要允许按需暴露外部地址

## 10. 存储策略

### 10.1 默认策略

首版不再把 `deploy/k8s-deprecated/1-volumes.yaml` 那种本地 PV/StorageClass 直接搬进主 Chart，而是采用更通用的 PVC 设计：

- 每个组件自行声明 PVC
- 通过 values 指定 `storageClass`
- 支持 `existingClaim`

### 10.2 需要持久化的组件

- platform
- postgresql
- valkey
- nats JetStream
- registry
- victoriaLogs

### 10.3 升级与回滚

- 配置类资源变更通过 checksum annotation 驱动滚动更新
- 所有凭据默认应通过 `lookup` + 已有 Secret 复用，避免升级时密码漂移
- 卸载时 PVC 默认保留，由文档明确说明数据清理方式

## 11. 实施阶段

### 阶段 0：文档与设计收敛

输出：

- 本文档
- Chart 目标目录和命名约定
- values 草图
- RBAC 与安装权限说明

### 阶段 1：Chart 骨架

任务：

- 创建 `Chart.yaml`
- 创建 `values.yaml`
- 创建 `values.schema.json`
- 创建 `_helpers.tpl`、`_config.tpl`、`_connections.tpl`
- 创建 `README.md` 与 `examples/`

验收：

- `helm lint` 可以通过
- `helm template` 可以输出空实现骨架

### 阶段 2：内部依赖组件模板

优先实现：

- `postgresql`
- `valkey`
- `nats`
- `registry`
- `victoriaLogs`

任务：

- 完成官方镜像模板化
- 完成 Secret/PVC/Service 定义
- 完成最小探针和资源参数

验收：

- 全 internal values 可渲染
- 所有组件都能在本地集群成功启动

### 阶段 3：平台配置生成

任务：

- 渲染 `config.toml`
- 渲染 `blocked.txt` ConfigMap
- 衔接内外部依赖地址
- 加入 checksum 注解

验收：

- `config.toml` 与当前 compose/k8s 语义一致
- internal/external 两种模式切换不会出现缺失字段

### 阶段 4：平台与权限模型

任务：

- 实现平台 `StatefulSet`
- 实现 `ServiceAccount`
- 实现 `ClusterRole`
- 实现 `ClusterRoleBinding`
- 实现 challenge namespace 与 network policy
- 实现平台 `Ingress` 和 `NodePort`

验收：

- 平台可成功读取 in-cluster 凭据
- `/api/ping` 正常
- 可访问前端页面和 API

### 阶段 5：外部服务模式与示例配置

任务：

- 为所有依赖补齐 `external` 模式
- 完成 `registry` / `victoriaLogs` 的 `disabled` 模式
- 输出四套示例 values

建议示例：

- `values-ingress-internal.yaml`
- `values-nodeport-internal.yaml`
- `values-ingress-external.yaml`
- `values-nodeport-external.yaml`

验收：

- 四套示例均可通过 `helm template`

### 阶段 6：测试与验收

任务：

- 增加 `helm test` job
- smoke test `/api/ping`
- 可选验证 registry `/v2/` 接口
- 可选验证 VictoriaLogs HTTP 可达性
- 在 `kind` / `k3d` / `k3s` 上做安装与升级测试

建议验证矩阵：

- internal + ingress
- internal + nodePort
- external + ingress
- external + nodePort
- registry disabled
- victoriaLogs disabled

### 阶段 7：迁移文档

任务：

- 说明如何从 `deploy/k8s-deprecated` 迁移
- 说明如何准备镜像、blocked.txt
- 说明如何预创建 service account / cluster role / cluster role binding
- 说明如何切换到外部数据库、缓存和队列

## 12. 验收标准

完成 Helm Chart v1 时，应满足以下条件：

- 能以单命令安装完整 internal 拓扑
- 能以 mixed external 模式安装平台
- 平台启动后 `/api/ping` 返回成功
- 平台前端可直接从平台镜像提供
- 配置和密钥更新会触发自动滚动
- 在不允许 chart 创建 cluster-scoped RBAC 的集群上，可通过预置 service account 模式运行
- 文档中明确说明平台当前为单实例有状态服务，不支持横向扩容

## 13. 风险与后续问题

### 13.1 当前风险

- 平台仍然依赖本地 Git bucket 和 media 目录，无法安全水平扩容
- registry 的节点可达地址在不同集群形态下差异很大，values 设计必须足够明确
- RBAC 仍以“现状兼容”为主，首版权限会比较宽
- challenge namespace 当前在代码中是硬编码常量，后续如果要做多租户或自定义 namespace，需要先改代码

### 13.2 后续可选优化

- 将平台 bucket/media 解耦到外部存储，再重新评估多副本部署
- 引入分布式锁后，再评估平台的高可用能力
- 对 RBAC 做最小权限收敛
- 为 PostgreSQL / Valkey / NATS 增加更强的生产特性，但不放在 v1

## 14. 建议的实施顺序

实际编码时，建议严格按照下面顺序推进：

1. 先完成 Chart 骨架和 values/schema
2. 先做 internal 依赖模板
3. 再做 config/secret 渲染
4. 再做 platform + service account + cluster role + cluster role binding
5. 最后补 ingress/nodePort、external 模式、测试和迁移文档

这样可以先确保最小可安装路径跑通，再逐步补齐可运维能力。
