# GCC偷走了重要的函数 - Fix 出题模板

这个目录是一套 Fix 题模板，适用于现有的“GCC偷走了重要的函数”题目。Fix 题会复用普通动态环境镜像作为内部靶机，平台把选手上传的修复文件放入靶机，再执行出题人提供的 `fix.sh`，最后启动一个独立 tester 容器访问靶机并判定修复是否通过。

## 文件说明

- `fix.toml`：Fix 题配置示例，可以通过题目仓库或管理员 Fix 面板写入。
- `fix.sh`：平台在内部靶机中执行的替换脚本，负责安装选手上传的修复文件。
- `tester/Dockerfile`：测试容器镜像构建文件。
- `tester/check.sh`：测试逻辑。脚本设置 `R2S_FIX_RESULT`，命令入口会输出 `R2S_FIX_RESULT=success` 或 `R2S_FIX_RESULT=failed` 供平台采集。
- `answer/main.py.patch`：参考修复补丁，供出题人制作标准答案或检查选手思路。

## 出题步骤

1. 继续使用本题原有 Attack 动态环境镜像，服务端口保持 `10001/tcp`，应用协议为 `raw`，`internet=false`。
2. 在题目“评测文件”中上传 `fix.sh`。平台会从 checker 附件区读取 `fix.toml` 里指定的 `fix_script`。
3. 构建 tester 镜像并推送到内置 Registry：

   ```bash
   cd docs/fix-challenge-template/gcc-stolen-important-function/tester
   REGISTRY=registry.example.com/ret2shell
   docker build -t "$REGISTRY/gcc-fix-tester:latest" .
   docker push "$REGISTRY/gcc-fix-tester:latest"
   ```

4. 把 `fix.toml` 中 `[tester].tag` 改成实际 tester 镜像地址，然后在管理员 Fix 面板保存配置，或把 `fix.toml` 放入题目仓库对应挑战目录。
5. 给选手提供原附件 `src.tar.gz`。选手提交时可以上传修复后的 `main.py`，也可以上传包含 `main.py` 的 `.tar`、`.tar.gz` 或 `.tgz`。

## Fix 脚本契约

平台执行 `fix.sh` 时会在靶机中设置这些环境变量：

- `R2S_FIX_UPLOAD`：选手上传文件在靶机内的路径，对应 `fix.toml` 的 `upload_path`。
- `R2S_FIX_ORIGINAL_NAME`：选手上传时的原文件名。
- `R2S_FIX_WORKDIR`：平台预留的工作目录，默认 `/tmp/ret2shell-fix`。

`fix.sh` 退出非 0 会直接判为失败。本模板会把上传的 `main.py` 安装到 `/home/ctf/src/main.py`。本题的服务由 `socat` 每次连接启动 `/run.sh`，替换文件后新连接会自动使用新代码；如果你的题目是常驻服务，需要在 `fix.sh` 中显式重启服务。

## Tester 脚本契约

tester 容器会收到这些环境变量：

- `R2S_FIX_TARGET_HOST`：内部靶机 Service DNS 名。
- `R2S_FIX_TARGET_PORT`：靶机端口，本题为 `10001`。
- `R2S_FIX_TARGET_URL`：`http://host:port` 形式的辅助变量，raw TCP 题可不用。
- `R2S_FIX_RESULT_ENV`：结果变量名，默认 `R2S_FIX_RESULT`。
- `R2S_FIX_RESULT`：默认值为 `failed`。

`check.sh` 应该只设置结果变量，不要在失败时让容器非 0 退出；平台要求 tester Pod 正常结束，并从日志中读取形如 `R2S_FIX_RESULT=success` 的结果行。

## 本题判定思路

原漏洞是只检查了语法树根节点的第一项，选手可以用 `__attribute__((section(".text")))` 把名为 `main` 的字节数组放入可执行代码段，从而绕过函数定义检查。tester 会先确认服务能响应，再发送这个利用载荷；如果输出中出现 `flag{` 或 `Congratulations`，判为失败。

参考修复可以直接拒绝危险 GCC 属性和内联汇编，也可以更彻底地遍历整棵 AST 并拒绝所有会定义可执行入口的结构。比赛中建议接受多种修复方式，只要 tester 证明服务仍可运行且利用失败即可。
