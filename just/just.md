# SolinBlog just 命令

所有可执行脚本都放在 `just/` 目录下；根目录 `justfile` 只负责把命令转发到这些脚本。脚本会根据自身位置计算项目根目录，因此可以从项目内任意子目录运行，也可以在不同机器、不同 checkout 路径下运行。

## 基础命令

```bash
just
```

列出可用命令。

```bash
just check
```

执行 `cargo check`。按项目约定，自动化验证只保证通过 `cargo check`。

```bash
just test-image-api
```

执行图床后端 API 的 sh 集成测试。测试会启动隔离的本地 server，覆盖鉴权、上传、公开读取、列表/详情、元数据更新、替换、删除和错误输入。

```bash
just build
```

执行 `cargo build --release --bin server`。

```bash
just run
```

执行 `cargo run --bin server`，会在项目根目录启动服务。服务本身会读取根目录 `.env`。

## macOS 服务部署

```bash
just deploy
```

将服务部署为当前用户的 macOS LaunchAgent：

- 先构建 `target/release/server`
- 生成 `~/Library/LaunchAgents/com.factrue.solinblog.plist`
- 设置 `WorkingDirectory` 为当前项目根目录
- 设置 `RunAtLoad` 和 `KeepAlive`
- 标准输出写入 `~/Library/Logs/SolinBlog/server.out.log`
- 标准错误写入 `~/Library/Logs/SolinBlog/server.err.log`

默认服务名为 `com.factrue.solinblog`。如需自定义：

```bash
SOLINBLOG_LAUNCHD_LABEL=com.example.solinblog just deploy
```

如需自定义 plist 或日志目录：

```bash
SOLINBLOG_LAUNCHD_DIR="$HOME/Library/LaunchAgents" \
SOLINBLOG_LOG_DIR="$HOME/Library/Logs/SolinBlog" \
just deploy
```

如需只生成并校验 plist，不注册 launchd：

```bash
SOLINBLOG_DEPLOY_DRY_RUN=1 just deploy
```

服务的 `WEB_HOST`、`WEB_PORT`、`TOKEN`、`SITE_URL` 等配置建议维护在项目根目录 `.env` 中，因为部署后的工作目录固定为项目根目录。

## 服务管理

```bash
just status
```

查看当前用户 LaunchAgent 的 launchd 状态。

```bash
just logs
```

持续查看服务 stdout/stderr 日志。

```bash
just undeploy
```

停止并移除当前用户的 LaunchAgent plist。不会删除项目文件、`.env`、`data/` 或已构建产物。

```bash
just redeploy
```

重新部署当前用户的 LaunchAgent，执行顺序固定为：先 `just undeploy` 停止并移除服务，再 `just deploy` 构建 `target/release/server` 并创建服务。
