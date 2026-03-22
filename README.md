# Warden: A Bitwarden-compatible server for Cloudflare Workers

[![Powered by Cloudflare](https://img.shields.io/badge/Powered%20by-Cloudflare-F38020?logo=cloudflare&logoColor=white)](https://www.cloudflare.com/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Deploy to Cloudflare Workers](https://img.shields.io/badge/Deploy%20to-Cloudflare%20Workers-orange?logo=cloudflare&logoColor=white)](https://workers.cloudflare.com/)

This project provides a self-hosted, Bitwarden-compatible server that can be deployed to Cloudflare Workers for free. It's designed to be low-maintenance, allowing you to "deploy and forget" without worrying about server management or recurring costs.

## Why another Bitwarden server?

While projects like [Vaultwarden](https://github.com/dani-garcia/vaultwarden) provide excellent self-hosted solutions, they still require you to manage a server or VPS. This can be a hassle, and if you forget to pay for your server, you could lose access to your passwords.

Warden aims to solve this problem by leveraging the Cloudflare Workers ecosystem. By deploying Warden to a Cloudflare Worker and using Cloudflare D1 for storage, you can have a completely free, serverless, and low-maintenance Bitwarden server.

## Features

* **Core Vault Functionality:** Create, read, update, and delete ciphers and folders.
* **File Attachments:** Optional Cloudflare KV or R2 storage for attachments.
* **TOTP Support:** Store and generate Time-based One-Time Passwords.
* **Bitwarden Compatible:** Works with official Bitwarden clients.
* **Free to Host:** Runs on Cloudflare's free tier.
* **Low Maintenance:** Deploy it once and forget about it.
* **Secure:** Your encrypted data lives in your Cloudflare D1 database.
* **Easy to Deploy:** Get up and running in minutes with the Wrangler CLI.

### Attachments Support

Warden supports file attachments using either **Cloudflare KV** or **Cloudflare R2** as the storage backend:

| Feature | KV | R2 |
|---------|----|----|  
| Max file size | **25 MB** (hard limit) | 100 MB (By request body size limit of Workers) |
| Credit card required | **No** | Yes |
| Streaming I/O | Yes | Yes |

**Backend selection:** R2 takes priority — if R2 is configured, it will be used. Otherwise, KV is used.

See the [deployment guide](docs/deployment.md) for setup details. R2 may incur additional costs; see [Cloudflare R2 pricing](https://developers.cloudflare.com/r2/pricing/).

## Current Status

**This project is not yet feature-complete**, ~~and it may never be~~. It currently supports the core functionality of a personal vault, including TOTP. However, it does **not** support the following features:

* Sharing
* 2FA login (except TOTP)
* File-based Bitwarden Send
* Device and session management
* Emergency access
* Admin operations
* Organizations
* Other Bitwarden advanced features

There are no immediate plans to implement these features. The primary goal of this project is to provide a simple, free, and low-maintenance personal password manager.

## Compatibility

* **Browser Extensions:** Chrome, Firefox, Safari, etc. (Tested 2025.11.1 on Chrome)
* **Android App:** The official Bitwarden Android app. (Tested 2025.11.0)
* **iOS App:** The official Bitwarden iOS app. (Tested 2025.11.0)

## Demo

A demo instance is available at [warden.qqnt.de](https://warden.qqnt.de).

You can register a new account using an email ending with `@warden-worker.demo` (The email does not need verification).

If you decide to stop using the demo instance, please delete your account to make space for others.

It's highly recommended to deploy your own instance since the demo can hit the rate limit and be disabled by Cloudflare.

## Getting Started

- 如果你已经熟悉 Cloudflare，只想看简版部署说明，请直接看 [docs/deployment.md](docs/deployment.md)。
- 如果你是第一次接触 Cloudflare / GitHub Actions，建议直接按照下面的中文教程一步一步做。

## 新手部署教程（推荐）

这一节是给第一次部署的人写的。只要你会使用 GitHub 和 Cloudflare 网页后台，就可以把这个项目跑起来。

最简单的流程是：

1. 注册 Cloudflare 账号
2. 把本仓库 fork 到你自己的 GitHub
3. 在 Cloudflare 创建一个 D1 数据库
4. 创建一个 Cloudflare API Token
5. 把几个必需的值填到 GitHub Secrets
6. 运行仓库自带的 GitHub Actions 工作流
7. 用生成的 `workers.dev` 地址登录 Bitwarden 客户端

你不需要自己准备 VPS、Docker、Linux 服务器，也不需要单独搭数据库服务器。

### 最终你会得到什么

教程完成后，你会得到：

- 一个运行在 Cloudflare Workers 上的 `warden-worker`
- 一个存放加密数据的 Cloudflare D1 数据库
- 一个可访问的 Web Vault 页面和 API
- 一个可以填进官方 Bitwarden 客户端的自建服务地址

### 开始前你需要准备什么

你需要有：

- 一个 [GitHub 账号](https://github.com/)
- 一个 [Cloudflare 账号](https://dash.cloudflare.com/sign-up)

如果你走 GitHub Actions 这条部署路径，理论上不需要先在本机安装任何工具。

### 第 1 步：Fork 本仓库

1. 打开本仓库的 GitHub 页面
2. 点击右上角的 `Fork`
3. Fork 到你自己的 GitHub 账号下

完成后，你应该会得到类似这样的地址：

```text
https://github.com/your-name/warden-worker
```

后面的所有操作，都应该在你自己的 fork 仓库里完成，而不是原仓库。

### 第 2 步：创建 Cloudflare D1 数据库

这个项目会把你的加密数据保存在 Cloudflare D1 里。

操作方法：

1. 登录 [Cloudflare Dashboard](https://dash.cloudflare.com/)
2. 左侧进入 `Workers & Pages`
3. 打开 `D1 SQL Database`
4. 点击 `Create`
5. 输入一个数据库名字，例如：

```text
warden-db
```

6. 点击创建

创建完成后，打开这个数据库页面，找到它的 **Database ID**。

你可以这样找：

- 方法 A：在 D1 数据库概览页直接复制显示出来的 ID
- 方法 B：如果你本机装了 Wrangler，也可以执行：

```bash
wrangler d1 list
```

把这个值保存下来，后面要作为 GitHub Secret 填进去，名字是：

```text
D1_DATABASE_ID
```

### 第 3 步：找到你的 Cloudflare Account ID

GitHub Actions 部署时需要知道要部署到哪个 Cloudflare 账号下。

查找方法：

1. 登录 [Cloudflare Dashboard](https://dash.cloudflare.com/)
2. 打开你的账号主页
3. 找到 `Account ID`
4. 复制它

它通常会出现在这些位置之一：

- 概览页右侧信息栏
- `Workers & Pages` 页面里的账号信息区域
- 有时账号 URL 中也能看到

把这个值保存下来，后面作为 GitHub Secret 使用，名字是：

```text
CLOUDFLARE_ACCOUNT_ID
```

### 第 4 步：创建 Cloudflare API Token

GitHub 工作流会用这个 Token 来做这些事情：

- 部署 Worker
- 执行 D1 初始化和迁移
- 上传 Worker 运行时 secrets

创建方法：

1. 打开 [Cloudflare API Tokens 页面](https://dash.cloudflare.com/profile/api-tokens)
2. 点击 `Create Token`
3. 你可以直接从 `Edit Cloudflare Workers` 模板开始，也可以自己创建自定义 Token
4. 确保这个 Token 拥有项目部署所需权限

如果你选择自定义 Token，建议至少包含这些权限：

- `Account` -> `Workers` -> `Edit`
- `Account` -> `D1` -> `Edit`
- 如果你要开 KV 附件：`Account` -> `Workers KV Storage` -> `Edit`
- 如果你要开 R2 附件：`Account` -> `R2` -> `Edit`

然后继续：

5. 在 `Account Resources` 中选择你的 Cloudflare 账号
6. 点击 `Continue to summary`
7. 点击 `Create Token`
8. 立刻复制生成的 Token

注意：

- Cloudflare 往往只会完整展示一次 Token
- 关闭页面前一定要先保存好

后面它要作为 GitHub Secret 填入，名字是：

```text
CLOUDFLARE_API_TOKEN
```

### 第 5 步：决定哪些邮箱可以注册

这个项目要求你提供一个 `ALLOWED_EMAILS` secret。

它用于控制哪些邮箱地址允许注册账号。

例如：

- 只允许一个邮箱：

```text
you@example.com
```

- 允许多个邮箱：

```text
you@example.com,friend@example.com
```

- 允许整个域名：

```text
*@example.com
```

如果你只是给自己用，最稳妥的做法就是只填你自己的邮箱。

### 第 6 步：生成 JWT Secrets

这个项目还要求两个运行时密钥：

- `JWT_SECRET`
- `JWT_REFRESH_SECRET`

它们用于 Worker 内部的身份认证逻辑。

要求是：

- 足够长
- 随机
- 不要和别的项目复用

如果你本机有 Python，可以直接生成：

```bash
python -c "import secrets; print(secrets.token_urlsafe(48)); print(secrets.token_urlsafe(48))"
```

第一行作为：

```text
JWT_SECRET
```

第二行作为：

```text
JWT_REFRESH_SECRET
```

如果你没有 Python，也可以用可信的密码生成器随机生成两段长字符串。

### 第 7 步：在 GitHub 仓库里添加 Secrets

现在回到你 fork 后的 GitHub 仓库，把前面得到的值填进去。

进入页面的方法：

1. 打开你的 fork 仓库
2. 点击 `Settings`
3. 左侧进入 `Secrets and variables`
4. 点击 `Actions`
5. 点击 `New repository secret`

把下面这些必需的 Secrets 逐个添加进去：

| Secret 名称 | 应填写的内容 |
|-------------|--------------|
| `CLOUDFLARE_API_TOKEN` | 第 4 步创建的 Cloudflare API Token |
| `CLOUDFLARE_ACCOUNT_ID` | 第 3 步找到的 Cloudflare Account ID |
| `D1_DATABASE_ID` | 第 2 步创建的 D1 数据库 ID |
| `ALLOWED_EMAILS` | 第 5 步设置的允许注册邮箱规则 |
| `JWT_SECRET` | 第 6 步生成的第一段随机字符串 |
| `JWT_REFRESH_SECRET` | 第 6 步生成的第二段随机字符串 |

如果你现在只想先把基础密码库跑起来，不开附件，到这里就已经可以部署了。

### 第 8 步：可选 - 开启附件功能

附件功能不是必须的。

如果你什么都不配置，主密码库照样能用，只是不能上传附件。

你有两种存储方案：

- `KV`：更容易开通，不要信用卡，但单文件硬上限是 `25 MB`
- `R2`：更适合大附件，但通常需要在 Cloudflare 开启计费

#### 方案 A：用 KV 开附件

创建 KV Namespace 的方法：

1. 打开 `Workers & Pages`
2. 进入 `KV`
3. 点击 `Create namespace`
4. 给它起个名字，例如：

```text
warden-worker-attachments-kv
```

5. 打开这个 Namespace，复制它的 id

如果你更喜欢用命令行：

```bash
wrangler kv namespace create ATTACHMENTS_KV
```

然后再多加一个 GitHub Secret：

| Secret 名称 | 应填写的内容 |
|-------------|--------------|
| `ATTACHMENTS_KV_ID` | 你的 KV Namespace ID |

#### 方案 B：用 R2 开附件

创建 R2 Bucket 的方法：

1. 打开 Cloudflare 的 `R2 Object Storage`
2. 点击 `Create bucket`
3. 输入一个 bucket 名字，例如：

```text
warden-attachments
```

4. 创建 bucket

如果你更喜欢命令行：

```bash
wrangler r2 bucket create warden-attachments
```

然后再多加一个 GitHub Secret：

| Secret 名称 | 应填写的内容 |
|-------------|--------------|
| `R2_NAME` | 你的 R2 bucket 名字 |

注意：

- 如果同时设置了 `ATTACHMENTS_KV_ID` 和 `R2_NAME`，项目会优先使用 `R2`
- 如果两者都没设置，附件功能保持关闭

### 第 9 步：运行部署工作流

仓库里已经自带了一个 GitHub Actions 工作流，名字叫 `Build`。

它会自动帮你完成这些事情：

- 下载 Web Vault 前端
- 删除过大的 source map 文件
- 把 D1 数据库 ID 写入部署配置
- 把 Worker secrets 上传到 Cloudflare
- 在数据库为空时自动初始化基础表
- 自动执行 migrations
- 如果你配置了附件，会自动绑定 KV 或 R2
- 最后自动部署 Worker

运行方法：

1. 打开你 fork 后的仓库
2. 点击 `Actions`
3. 找到工作流 `Build`
4. 点击 `Run workflow`
5. 确认分支是 `main`
6. 点击绿色的 `Run workflow`

然后等待执行完成。

如果工作流执行成功，你的 Worker 就已经部署好了。

### 第 10 步：找到最终的 Worker 地址

部署完成后，你需要拿到最终的服务地址，用来给 Bitwarden 客户端登录。

查找方法：

- 方法 A：打开 GitHub Actions 的执行日志，里面通常会输出最终部署地址
- 方法 B：进入 Cloudflare Dashboard -> `Workers & Pages` -> 你的 Worker -> `Overview`
- 方法 C：如果你是本地使用 Wrangler 部署，终端也会打印 `workers.dev` 地址

通常它会长这样：

```text
https://your-worker-name.your-subdomain.workers.dev
```

如果 `wrangler.toml` 里启用了 `workers.dev`，你可以先直接用这个地址，不必一开始就上自定义域名。

### 第 11 步：验证服务是否正常

部署完成后，用浏览器打开下面两个地址：

- 首页：

```text
https://your-worker-name.your-subdomain.workers.dev
```

- 配置接口：

```text
https://your-worker-name.your-subdomain.workers.dev/api/config
```

如果部署正常：首页应该能打开，`/api/config` 应该返回 JSON。

### 第 12 步：在 Bitwarden 客户端中登录

现在打开官方 Bitwarden 客户端：

1. 进入登录页面
2. 选择自建服务器 / 自定义服务器
3. 填入你的 Worker 地址
4. 用 `ALLOWED_EMAILS` 允许的邮箱注册
5. 然后登录

如果你只允许了一个特定邮箱，那么注册时必须使用那个精确邮箱地址。

## 新手常见问题

### 我应该选哪种部署方式？

如果你是新手，建议直接用 **GitHub Actions 部署**。

只有在你已经熟悉 Wrangler、想从自己终端手动控制部署时，才建议看 [docs/deployment.md](docs/deployment.md) 里的 CLI 部署方式。

### 我需要自己手动修改 `wrangler.toml` 吗？

对于新手推荐的 GitHub Actions 部署路径，一般 **不需要**。

因为工作流已经会自动处理这些事：

- 替换 `D1_DATABASE_ID`
- 上传运行时 secrets
- 当相关 secret 存在时自动追加 KV / R2 附件绑定

### 如果工作流失败了怎么办？

请优先检查下面几项：

- `CLOUDFLARE_API_TOKEN` 填对了吗？
- `CLOUDFLARE_ACCOUNT_ID` 和你的 D1 数据库是不是同一个 Cloudflare 账号下的？
- `D1_DATABASE_ID` 有没有复制错？
- `ALLOWED_EMAILS`、`JWT_SECRET`、`JWT_REFRESH_SECRET` 有没有漏填？
- 如果你开启了附件，`ATTACHMENTS_KV_ID` 或 `R2_NAME` 是否正确？

### 为什么我不能注册账号？

最常见原因是 `ALLOWED_EMAILS` 和你当前使用的邮箱不匹配。

例如：

- 如果你填的是：

```text
you@example.com
```

那就只有这个邮箱能注册。

- 如果你想允许整个域名：

```text
*@example.com
```

### 为什么附件还是不能用？

因为附件功能必须额外绑定一个存储后端。

你至少要配置其中一个：

- `ATTACHMENTS_KV_ID`
- `R2_NAME`

然后重新运行一次 `Build` 工作流。

### 以后还能加自定义域名吗？

可以。建议先用 `workers.dev` 跑通，再按 [Configure Custom Domain (Optional)](#configure-custom-domain-optional) 里的说明补上自定义域名。

## 进阶部署文档

如果你想要更底层、更细粒度的控制，下面这些文档仍然是正式参考：

- [docs/deployment.md](docs/deployment.md)
- [Configure Custom Domain (Optional)](#configure-custom-domain-optional)
- [Environment Variables](#environment-variables)
- [Database Backup & Restore](docs/db-backup-recovery.md#github-actions-backups)

## 前端（Web Vault）

前端页面会和 Worker 一起打包部署，使用的是 [Cloudflare Workers Static Assets](https://developers.cloudflare.com/workers/static-assets/)。

GitHub Actions 工作流会自动下载一个固定版本的 [bw_web_builds](https://github.com/dani-garcia/bw_web_builds)（也就是 Vaultwarden 的 Web Vault 前端），默认版本是 `v2025.12.0`，并和后端一起部署。

如果你想覆盖默认版本，可以在 GitHub Actions Variables 中设置：

- 生产环境：`BW_WEB_VERSION`
- 开发环境：`BW_WEB_VERSION_DEV`

你也可以把它设成 `latest` 来跟随上游最新版本。

**它的工作方式是：**
- 静态文件（HTML、CSS、JS）由 Cloudflare 边缘网络直接提供
- API 请求（`/api/*`、`/identity/*`）会转发给 Rust Worker
- 不需要再额外部署一个 Cloudflare Pages 前端项目

**可选的 UI 覆盖：**
- 项目自带了一些轻量级的自托管 UI 微调，放在 `public/css/`
- 在 CI/CD 中（本地也可选）会在解压 `bw_web_builds` 后执行：
  - `bash scripts/apply-web-vault-overrides.sh public/web-vault`

> [!NOTE]
> 如果你以前把前端单独部署在 Cloudflare Pages 上，现在可以删掉那个 `warden-frontend` Pages 项目，再把路由改回 Worker。因为现在前端已经直接打包进 Worker，不再需要单独的前端部署。

> [!WARNING]
> Web Vault 前端来自 Vaultwarden，所以界面上会显示很多高级功能入口，但其中相当一部分在当前项目里并没有实现。请结合 [Current Status](#current-status) 一起看。

## 自定义域名（可选）

`wrangler.toml` 当前已经启用了默认的 `*.workers.dev` 域名，所以新手可以先直接使用自动生成的 Worker 地址，等服务跑通后再补自定义域名。

如果你想把默认的 `*.workers.dev` 地址换成自己的域名，可以按下面步骤操作。

### 第 1 步：添加 DNS 记录

1. 登录 [Cloudflare Dashboard](https://dash.cloudflare.com/)
2. 选择你的域名（例如 `example.com`）
3. 打开 **DNS** -> **Records**
4. 点击 **Add record**，填写：
   - **Type:** `A`（如果你用 IPv6 也可以选 `AAAA`）
   - **Name:** 你的子域名，例如 `vault`，这样最后就是 `vault.example.com`
   - **IPv4 address:** `192.0.2.1`（这是一个占位地址，真正路由由 Worker 处理）
   - **Proxy status:** 一定要选 **Proxied**（橙色云朵）
   - **TTL:** Auto
5. 点击保存

> [!IMPORTANT]
> **Proxy status 必须是 "Proxied"（橙云）**。如果是 `DNS only`（灰云），Worker 路由不会生效。

### 第 2 步：为 Worker 添加 Route

1. 进入 **Workers & Pages**，选择你的 `warden-worker`
2. 点击 **Settings** -> **Domains & Routes**
3. 点击 **Add** -> **Route**
4. 填写：
   - **Route:** `vault.example.com/*`（替换成你的域名）
   - **Zone:** 选择你的域名 Zone
   - **Worker:** `warden-worker`
5. 点击添加

## 首次使用建议

如果你刚部署成功，建议按这个顺序做第一次检查：

1. 打开首页，确认 Web Vault 能正常加载
2. 打开 `/api/config`，确认接口返回 JSON
3. 用 `ALLOWED_EMAILS` 允许的邮箱注册一个账号
4. 登录后先新增一个普通密码条目
5. 如果你开了附件，再测试一次上传和下载
6. 如果你开了 `Bitwarden Send`，再测试一次创建和访问

这样可以最快确认：前端、API、数据库、注册、登录和附件功能是否全部正常。

## 内置限流

项目内置了基于 [Cloudflare Rate Limiting API](https://developers.cloudflare.com/workers/runtime-apis/bindings/rate-limit/) 的限流机制，用来保护敏感接口。

| Endpoint | Rate Limit | Key Type | Purpose |
|----------|------------|----------|---------|
| `/identity/connect/token` | 5 req/min | Email address | Prevent password brute force |
| `/api/accounts/register` | 5 req/min | IP address | Prevent mass registration & email enumeration |
| `/api/accounts/prelogin` | 5 req/min | IP address | Prevent email enumeration |

你可以在 `wrangler.toml` 中调整限流配置：

```toml
[[ratelimits]]
name = "LOGIN_RATE_LIMITER"
namespace_id = "1001"
# Adjust limit (requests) and period (10 or 60 seconds)
simple = { limit = 5, period = 60 }
```

> [!NOTE]
> `period` 只能是 `10` 或 `60` 秒。更详细的说明请看 [Cloudflare 官方文档](https://developers.cloudflare.com/workers/runtime-apis/bindings/rate-limit/)。

如果这个 binding 缺失，请求仍然会继续执行，只是不会启用限流。

## 配置说明

### Durable Objects（CPU 卸载）

Cloudflare Workers 免费版对单次请求的 CPU 时间限制比较紧，而下面两类接口特别吃 CPU：

- 导入接口：通常会处理较大的 JSON（大约 500kB 到 1MB），并执行解析和批量写入
- 注册、登录、密码校验接口：服务端需要执行 PBKDF2 密码验证

为了让主 Worker 保持轻量，同时还能支持这些高开销操作，Warden 可以把部分请求转交给 **Durable Objects（DO）**：

- **Heavy DO (`HEAVY_DO`)**：使用 Rust 实现的 `HeavyDo`，复用了现有的 axum router，因此不需要重复写业务逻辑，同时可以获得更高的 CPU 预算

**如何开启 / 关闭**

是否启用 CPU 卸载，取决于 `wrangler.toml` 中是否配置了 `HEAVY_DO` 这个 Durable Object binding。

> [!NOTE]
> Durable Objects 在免费计划下单次请求可以拥有更高的 CPU 时间上限，通常是 30 秒，适合用来承接这些重操作。可参考 [Cloudflare Durable Objects limits](https://developers.cloudflare.com/durable-objects/platform/limits/)。
>
> Durable Objects 可能涉及两类计费：计算与存储。本项目不会使用 DO 存储，而且免费额度通常已经足够个人使用。详细说明见 [Cloudflare Durable Objects pricing](https://developers.cloudflare.com/durable-objects/platform/pricing/)。
>
> 如果你选择关闭 Durable Objects，在某些场景下 Cloudflare 可能更容易对重请求进行限制，这时你可能需要升级付费计划。

### 环境变量

你可以在 `wrangler.toml` 的 `[vars]` 中配置环境变量，也可以通过 Cloudflare Dashboard 进行设置：

* **`PASSWORD_ITERATIONS`**（可选，默认：`600000`）：
  - 服务端密码哈希使用的 PBKDF2 迭代次数
  - 最低不能低于 `600000`
* **`TRASH_AUTO_DELETE_DAYS`**（可选，默认：`30`）：
  - 软删除数据保留的天数
  - 设为 `0` 或负数表示关闭自动清理
* **`IMPORT_BATCH_SIZE`**（可选，默认：`30`）：
  - 导入 / 删除操作的批处理大小
  - 设为 `0` 表示不分批
* **`DISABLE_USER_REGISTRATION`**（可选，默认：`true`）：
  - 控制客户端界面是否显示“注册”按钮
  - 只影响界面显示，不改变服务端逻辑
* **`AUTHENTICATOR_DISABLE_TIME_DRIFT`**（可选，默认：`false`）：
  - 设为 `true` 后，TOTP 校验时不再允许前后一个时间步长的容差
* **`ATTACHMENT_MAX_BYTES`**（可选）：
  - 单个附件允许的最大字节数
  - 示例：`104857600` 代表 100MB
* **`ATTACHMENT_TOTAL_LIMIT_KB`**（可选）：
  - 单个用户允许使用的附件总空间，单位 KB
  - 示例：`1048576` 代表 1GB
* **`ATTACHMENT_TTL_SECS`**（可选，默认：`300`，最小：`60`）：
  - 附件上传 / 下载 URL 的有效期，单位秒

### 定时任务（Cron）

Worker 会通过定时任务自动清理软删除数据。

默认情况下，每天 UTC 时间 `03:00` 执行一次，也就是 `wrangler.toml` 中 `[triggers]` 里的：

```text
0 3 * * *
```

如果你想调整执行时间，可以修改这个 cron 表达式。语法说明见 [Cloudflare Cron Triggers documentation](https://developers.cloudflare.com/workers/configuration/cron-triggers/)。

## 数据库相关操作

- **备份与恢复：** 见 [Database Backup & Restore](docs/db-backup-recovery.md#github-actions-backups)
- **时间点恢复（Time Travel）：** 见 [D1 Time Travel](docs/db-backup-recovery.md#d1-time-travel-point-in-time-recovery)
- **写入全局等价域名（可选）：** 见 [docs/deployment.md](docs/deployment.md)
- **本地 D1 开发：**
  - 快速启动：`wrangler dev --persist`
  - 完整启动（含 Web Vault）：先下载前端资源，再执行 `wrangler dev --persist`
  - 本地导入备份：`wrangler d1 execute vault1 --file=backup.sql`
  - 查看本地数据库：`.wrangler/state/v3/d1/`

## 本地使用 D1 开发

如果你想在本机调试，可以通过 Wrangler 启动带 D1 支持的本地环境。

**快速启动（仅 API）：**

```bash
wrangler dev --persist
```

**完整启动（包含 Web Vault）：**

1. 先下载前端资源（见 [部署文档](docs/deployment.md#5-下载前端web-vault)）
2. 然后启动：

   ```bash
   wrangler dev --persist
   ```

3. 打开：

```text
http://localhost:8787
```

**临时使用生产数据调试：**

1. 先下载并解密备份（见 [备份文档](docs/db-backup-recovery.md#restoring-database-to-cloudflare-d1)）
2. 不带 `--remote` 导入到本地：

   ```bash
   wrangler d1 execute vault1 --file=backup.sql
   ```

3. 启动 `wrangler dev --persist`，然后让客户端连接到：

```text
http://localhost:8787
```

**查看本地 SQLite 文件：**

```bash
ls .wrangler/state/v3/d1/
sqlite3 .wrangler/state/v3/d1/miniflare-D1DatabaseObject/*.sqlite
```

> [!NOTE]
> 本地开发需要安装 Node.js 和 Wrangler。Worker 会通过 [workerd](https://github.com/cloudflare/workerd) 提供的模拟环境运行。

## 贡献

欢迎提交 Issue 和 PR。

在提交代码前，建议至少先运行：

```bash
cargo fmt
cargo clippy --target wasm32-unknown-unknown --no-deps
```

## 许可证

本项目基于 MIT License 发布，详见 `LICENSE` 文件。
