# SolinBlog Docker 部署指南

本文档面向希望使用 Docker / Docker Compose 部署 SolinBlog 的用户。

相关文件：[`Dockerfile`](Dockerfile:1)、[`docker-compose.yml`](docker-compose.yml:1)、[`.dockerignore`](.dockerignore:1)、环境变量示例 [`.env.example`](.env.example:1)。

---

## 1. 快速开始（Docker Compose 一键部署）

> 前置：已安装 Docker（含 Compose 插件）。

1) 复制环境变量文件：

```bash
cp .env.example .env
```

2) 按需修改 [`.env`](.env.example:1)（至少建议设置 [`SITE_URL`](.env.example:3) 与 [`MCP_TOKEN`](.env.example:4)）。

3) 一键启动：

```bash
docker compose up -d --build
```

4) 查看日志（获取 MCP 地址、排查启动问题）：

```bash
docker compose logs -f solinblog
```

5) 访问服务：

- 本机访问：`http://localhost:3002`
- MCP 入口：启动日志会输出 `MCP endpoint: http://{addr}/{token}/mcp`（由 [`main()`](src/main.rs:422) 打印）。

### 基本配置说明

默认编排见 [`docker-compose.yml`](docker-compose.yml:1)：

- 端口映射：`3002:3002`（可将左侧宿主机端口改为其他值，例如 `8080:3002`）。
- 监听地址：容器内通过 [`WEB_HOST`](docker-compose.yml:13) 设置为 `0.0.0.0`，确保能被容器外访问（对应 [`WEB_HOST`](src/main.rs:450)）。
- 数据与模板挂载：见「数据持久化」与「自定义模板」。

---

## 2. 环境变量配置

项目在启动时会读取环境变量（见 [`main()`](src/main.rs:422) 与 [`render_index_html()`](src/web.rs:35)）。建议通过 Compose 的 `env_file` 或 `docker run --env-file` 统一管理。

### 2.1 全部支持的环境变量

| 变量名 | 是否必需 | 作用 | 默认行为/建议 |
|---|---:|---|---|
| `WEB_HOST` | 否 | Web 服务监听地址 | 若未设置，代码默认回退到 `127.0.0.1`（见 [`WEB_HOST`](src/main.rs:450)）；容器部署务必设为 `0.0.0.0`（Compose 已设置，见 [`WEB_HOST`](docker-compose.yml:13)；镜像也在 [`Dockerfile`](Dockerfile:15) 里设置了默认值）。 |
| `WEB_PORT` | 否 | Web 服务监听端口 | 代码默认 `3000`（见 [`WEB_PORT`](src/main.rs:451)）；Docker 镜像默认 `3002`（见 [`Dockerfile`](Dockerfile:16)）；Compose 映射为 `3002:3002`（见 [`ports`](docker-compose.yml:14)）。 |
| `SITE_URL` | **建议必填** | 站点对外访问的基础 URL（用于生成完整 URL） | 用于在缺少请求头时解析 base url（见 [`resolve_base_url()`](src/main.rs:561)），以及 MCP URL 生成（见 [`resolve_site_url_from_env()`](src/main.rs:587)）。生产环境强烈建议填写，例如 `https://blog.example.com`（不要以 `/` 结尾）。 |
| `MCP_TOKEN` | **建议必填** | MCP 接口路径中的 token（同时起到“路径级鉴权”作用） | 若为空，服务会自动生成并在启动日志打印（见 [`MCP_TOKEN`](src/main.rs:426) 与 `MCP token generated` 输出）。建议显式配置，避免每次重启 token 变化。 |
| `BEIAN_NUMBER` | 否 | 首页底部备案号展示 | 为空则不显示；非空则渲染到首页 footer（见 [`BEIAN_NUMBER`](src/web.rs:65)）。 |

### 2.2 配置示例

参考 [`.env.example`](.env.example:1)：

```dotenv
WEB_HOST=0.0.0.0
WEB_PORT=3002
SITE_URL=https://example.com
MCP_TOKEN=please-change-me
BEIAN_NUMBER=浙ICP备2024056246号
```

> 提示：如果你让系统自动生成 `MCP_TOKEN`，可通过 `docker compose logs -f solinblog` 查看启动时打印的 token。

---

## 3. 数据持久化

SolinBlog 的页面数据默认存储在容器内的 `/app/data` 目录（服务端创建 store：[`PageStore::new("data")`](src/main.rs:425)）。

### 3.1 Compose 挂载方式

默认 Compose 已将宿主机的 `./data` 挂载到容器 `/app/data`（见 [`volumes`](docker-compose.yml:16)）：

- 宿主机：`./data`
- 容器内：`/app/data`

### 3.2 备份建议

推荐将 `data/` 纳入定期备份（例如每日快照）。常见做法：

1) 先停服务，保证备份一致性：

```bash
docker compose stop solinblog
```

2) 打包备份目录：

```bash
tar -czf solinblog-data-$(date +%F).tar.gz data
```

3) 再启动服务：

```bash
docker compose start solinblog
```

### 3.3 权限注意

镜像在运行阶段使用非 root 用户（见 [`USER appuser`](Dockerfile:27)，UID 为 `10001`，见 [`useradd`](Dockerfile:20)）。

如果你在 Linux 上使用 bind mount 且遇到 `Permission denied`，通常需要让宿主机 `data/` 目录对 UID `10001` 可写（例如 `sudo chown -R 10001:10001 ./data`）。

---

## 4. 自定义模板（挂载 `front/`）

SolinBlog 运行时会读取 `front/` 下的静态模板文件：

- 首页模板：[`front/index.html`](front/index.html:1)（见 [`render_index_html()`](src/web.rs:35)）
- Token 生成页：[`front/token-generator.html`](front/token-generator.html:1)（见 [`token_generator_handler()`](src/main.rs:550)）

### 4.1 Compose 挂载方式

默认 Compose 已将宿主机的 `./front` 挂载到容器 `/app/front`（见 [`volumes`](docker-compose.yml:16)）。这意味着：

- 你对宿主机 `front/` 的修改会直接影响容器内的页面渲染（无需重新构建镜像）。
- 若不想覆盖镜像自带模板，可去掉此挂载，但要自行确保容器内 `/app/front` 内容完整。

### 4.2 模板修改注意事项

首页模板使用“字符串占位符替换”，不是完整模板引擎；缺少占位符会导致首页渲染失败（HTTP 500）。详细规则请阅读：[`front/README.md`](front/README.md:1)。

---

## 5. 构建与运行

### 5.1 手动构建镜像

`Dockerfile` 为多阶段构建（见 [`Dockerfile`](Dockerfile:1)）。在项目根目录执行：

```bash
docker build -t solinblog:latest -f Dockerfile .
```

### 5.2 直接使用 `docker run`

示例（使用 `.env`，并挂载 `data/` 与 `front/`）：

```bash
docker run -d \
  --name solinblog \
  -p 3002:3002 \
  --env-file .env \
  -v "$(pwd)/data:/app/data" \
  -v "$(pwd)/front:/app/front" \
  solinblog:latest
```

常用操作：

```bash
docker logs -f solinblog
docker restart solinblog
docker stop solinblog
docker rm -f solinblog
```

### 5.3 常用 `docker compose` 命令

```bash
# 启动（后台）
docker compose up -d

# 重新构建并启动
docker compose up -d --build

# 查看日志
docker compose logs -f solinblog

# 停止/删除容器（保留 data/front 目录）
docker compose down

# 停止但不删除
docker compose stop

# 查看容器状态
docker compose ps
```

---

## 6. 反向代理配置（可选：Nginx）

当你使用 Nginx/Caddy/Traefik 等反向代理提供 HTTPS 时，务必正确传递 `Host` 与 `x-forwarded-proto`：

- SolinBlog 在渲染 `sitemap.xml` / 生成完整 URL 时，会优先使用请求头 `host` 和 `x-forwarded-proto`（见 [`resolve_base_url()`](src/main.rs:561)）。
- 如果缺少 `x-forwarded-proto`，服务会默认当作 `http`，从而导致 sitemap 或页面 URL 使用错误协议（HTTPS 站点被生成成 HTTP）。

### 6.1 Nginx 配置示例

> 假设 SolinBlog 在同机 `127.0.0.1:3002`，对外域名为 `blog.example.com`。

```nginx
server {
  listen 80;
  server_name blog.example.com;

  location / {
    proxy_pass http://127.0.0.1:3002;

    # 关键：正确传递 Host 与协议
    proxy_set_header Host              $host;
    proxy_set_header X-Forwarded-Proto $scheme;

    # 常见转发头（可选）
    proxy_set_header X-Real-IP         $remote_addr;
    proxy_set_header X-Forwarded-For   $proxy_add_x_forwarded_for;

    # 若你的环境需要处理 Upgrade/长连接，可开启（可选）
    proxy_http_version 1.1;
    proxy_set_header Upgrade    $http_upgrade;
    proxy_set_header Connection $connection_upgrade;
  }
}
```

> 注：若你使用 HTTPS（推荐），请在对应的 `server { listen 443 ssl; ... }` 中同样保留 `proxy_set_header X-Forwarded-Proto $scheme;`。

