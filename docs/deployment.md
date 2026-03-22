# 部署说明

本文档介绍两种部署方式：

- `CLI` 手动部署
- `GitHub Actions` 自动部署

如果你是第一次接触 Cloudflare，建议优先使用 **GitHub Actions 自动部署**。

## CLI 手动部署

如果你更喜欢自己在终端里一步一步执行命令，可以走这条路径。

### 1. 克隆仓库

```bash
git clone https://github.com/your-username/warden-worker.git
cd warden-worker
```

### 2. 创建 D1 数据库

```bash
wrangler d1 create warden-db
```

执行完成后，Wrangler 会输出数据库信息，其中最重要的是 `database_id`。

### 3. （可选）开启附件存储

默认情况下，附件不是必须的。

如果你想开启附件功能，可以选择：

- `KV`：不需要信用卡，但单文件上限为 `25 MB`
- `R2`：更适合较大文件，但通常需要启用 Cloudflare 计费

如果你想使用 `R2`，先创建 Bucket：

```bash
# 创建生产环境 Bucket
wrangler r2 bucket create warden-attachments
```

然后在 `wrangler.toml` 中启用 `R2` 对应的 binding。

注意：

- 附件功能是可选的
- 如果你移除 `KV` 和 `R2` 两种 binding，附件会关闭，但主密码库仍然能正常工作

### 4. 配置数据库 ID

创建 D1 数据库后，Wrangler 会输出一个 `database_id`。

为了避免把这个值写死在仓库里，项目使用环境变量来提供数据库 ID。

你有两种方式：

#### 方式 A：使用 `.env` 文件（推荐）

在项目根目录创建一个 `.env` 文件，并写入：

```text
D1_DATABASE_ID="your-database-id-goes-here"
```

记得把 `.env` 加进 `.gitignore`，避免误提交到 Git。

#### 方式 B：在当前 shell 中直接设置环境变量

```bash
export D1_DATABASE_ID="your-database-id-goes-here"
wrangler deploy
```

### 5. 下载前端（Web Vault）

```bash
# 默认使用固定版本，可通过 BW_WEB_VERSION 覆盖
BW_WEB_VERSION="${BW_WEB_VERSION:-v2025.12.0}"
if [ "${BW_WEB_VERSION}" = "latest" ]; then
  BW_WEB_VERSION="$(curl -s https://api.github.com/repos/dani-garcia/bw_web_builds/releases/latest | jq -r .tag_name)"
fi

# 下载并解压
wget "https://github.com/dani-garcia/bw_web_builds/releases/download/${BW_WEB_VERSION}/bw_web_${BW_WEB_VERSION}.tar.gz"
tar -xzf "bw_web_${BW_WEB_VERSION}.tar.gz" -C public/
rm "bw_web_${BW_WEB_VERSION}.tar.gz"

# 删除过大的 source map，避免超过 Cloudflare 静态资源单文件限制
find public/web-vault -type f -name '*.map' -delete
```

如果你还想应用项目自带的轻量 UI 覆盖样式：

```bash
bash scripts/apply-web-vault-overrides.sh public/web-vault
```

### 6. 初始化数据库并部署 Worker

```bash
# 第一次部署前执行一次基础建表
wrangler d1 execute vault1 --file sql/schema.sql --remote

# 执行 migrations
wrangler d1 migrations apply vault1 --remote

# （可选）把全局等价域名写入 D1
# 默认会下载 Vaultwarden 的 global_domains.json
bash scripts/seed-global-domains.sh --db vault1 --remote

# 正式部署
wrangler deploy
```

这一步会把 Worker 部署到 Cloudflare，并完成数据库初始化。

### 7. 设置运行时 Secrets

你至少需要设置以下三个运行时 Secret：

- `ALLOWED_EMAILS`
- `JWT_SECRET`
- `JWT_REFRESH_SECRET`

例如：

- `ALLOWED_EMAILS`: `your-email@example.com`
- `JWT_SECRET`: 一段长随机字符串
- `JWT_REFRESH_SECRET`: 另一段长随机字符串

### 8. 在 Bitwarden 客户端中配置服务地址

打开你的 Bitwarden 客户端，选择自建服务器登录，并填入你部署后的 Worker 地址。

如果你希望使用自定义域名，而不是默认的 `workers.dev` 地址，请参考 [README 中的自定义域名说明](../README.md#configure-custom-domain-optional)。

## 使用 GitHub Actions 自动部署

这是最推荐的生产部署方式，因为它能保证每次部署流程一致，而且新手更容易上手。

### 必需 Secrets

在 GitHub 仓库中打开：`Settings > Secrets and variables > Actions`

然后添加以下 Secrets：

| Secret | 是否必需 | 说明 |
|--------|----------|------|
| `CLOUDFLARE_API_TOKEN` | 是 | Cloudflare API Token |
| `CLOUDFLARE_ACCOUNT_ID` | 是 | Cloudflare 账号 ID |
| `D1_DATABASE_ID` | 是 | 生产环境 D1 数据库 ID |
| `ALLOWED_EMAILS` | 是 | 允许注册的邮箱规则，例如 `you@example.com` 或 `*@example.com` |
| `JWT_SECRET` | 是 | Worker 运行时 access token 密钥 |
| `JWT_REFRESH_SECRET` | 是 | Worker 运行时 refresh token 密钥 |
| `ATTACHMENTS_KV_ID` | 否 | 生产环境附件用 KV Namespace ID |
| `R2_NAME` | 否 | 生产环境附件用 R2 Bucket 名字 |
| `D1_DATABASE_ID_DEV` | 否 | `dev` 分支用的 D1 数据库 ID |
| `ALLOWED_EMAILS_DEV` | 否 | 开发环境专用邮箱规则 |
| `JWT_SECRET_DEV` | 否 | 开发环境专用 access token 密钥 |
| `JWT_REFRESH_SECRET_DEV` | 否 | 开发环境专用 refresh token 密钥 |
| `ATTACHMENTS_KV_ID_DEV` | 否 | 开发环境附件用 KV Namespace ID |
| `R2_NAME_DEV` | 否 | 开发环境附件用 R2 Bucket 名字 |

### 如何获取 Cloudflare Account ID

1. 登录 [Cloudflare Dashboard](https://dash.cloudflare.com/)
2. 打开你的 Cloudflare 账号主页
3. 找到 `Account ID`
4. 复制它

你通常可以在以下位置看到它：

- 概览页右侧信息区域
- `Workers & Pages` 页面中的账号详情
- 有时也能在 Dashboard URL 中看到

### 如何获取 Cloudflare API Token

`CLOUDFLARE_API_TOKEN` 至少需要这些权限：

- **Edit Cloudflare Workers**：用于部署 Worker
- **Edit D1**：用于初始化数据库、执行迁移、备份
- **Edit KV**：如果你要使用 KV 附件
- **Edit R2**：如果你要使用 R2 附件

创建方式：

1. 打开 [Cloudflare API Tokens 页面](https://dash.cloudflare.com/profile/api-tokens)
2. 点击 `Create Token`
3. 可以直接使用 `Edit Cloudflare Workers` 模板，或自己创建自定义 Token
4. 如果是自定义 Token，补上 `Account -> D1 -> Edit`
5. 如果你要开 KV 附件，再补 `Workers KV Storage -> Edit`
6. 如果你要开 R2 附件，再补 `R2 -> Edit`
7. 在 `Account Resources` 中选择你的账号
8. 点击 `Continue to Summary`
9. 点击 `Create Token`
10. 立刻复制并保存这个 Token

### 如何获取 D1 数据库 ID

推荐方式：

```bash
wrangler d1 create warden-db
```

Wrangler 输出里会带有 `database_id`。

如果数据库已经存在，也可以执行：

```bash
wrangler d1 list
```

在 Cloudflare Dashboard 中也能查看：

1. 打开 `Workers & Pages`
2. 进入 `D1 SQL Database`
3. 打开你的数据库
4. 在数据库详情页复制 `Database ID`

### 如何开启附件（可选）

如果你不配置附件，主密码库依然可以正常工作。

#### 方案 A：使用 KV

创建 KV Namespace：

```bash
wrangler kv namespace create ATTACHMENTS_KV
```

把输出的 Namespace ID 填到 GitHub Secret：

```text
ATTACHMENTS_KV_ID
```

#### 方案 B：使用 R2

创建 R2 Bucket：

```bash
wrangler r2 bucket create warden-attachments
```

把 Bucket 名字填到 GitHub Secret：

```text
R2_NAME
```

说明：

- 如果 `ATTACHMENTS_KV_ID` 和 `R2_NAME` 同时存在，运行时会优先使用 `R2`
- 这些 binding 会在 CI/CD 部署时自动追加到 `wrangler.toml`
- 你不需要手动去 Cloudflare Dashboard 里编辑 Worker binding

### 可选 Variables

#### Web Vault 前端版本

你可以通过 GitHub Actions Variables 固定或覆盖打包的 Web Vault（`bw_web_builds`）版本：

| Variable | 作用环境 | 默认值 | 示例 | 说明 |
|----------|----------|--------|------|------|
| `BW_WEB_VERSION` | 生产环境（`main/uat/release*`） | `v2025.12.0` | `v2025.12.0` | 可设为 `latest` 跟随上游最新版本 |
| `BW_WEB_VERSION_DEV` | 开发环境（`dev`） | `v2025.12.0` | `v2025.12.0` | 可设为 `latest` |

#### Global Equivalent Domains

Bitwarden 客户端会使用 `globalEquivalentDomains` 做 URI 匹配。

为了避免把一大段 JSON 直接打包进 Worker，项目支持把它存进 D1，并在部署时自动写入。

| Variable | 作用环境 | 默认值 | 示例 | 说明 |
|----------|----------|--------|------|------|
| `SEED_GLOBAL_DOMAINS` | 生产 + 开发 | `true` | `false` | 如果设为 `false`，接口将返回空的 `globalEquivalentDomains` |
| `GLOBAL_DOMAINS_URL` | 生产 | 空 | 原始 GitHub 文件地址 | 可选，用于固定某个 Vaultwarden 版本的数据 |
| `GLOBAL_DOMAINS_URL_DEV` | 开发 | 空 | 原始 GitHub 文件地址 | 开发环境版本 |

如果你跳过这一步，`/api/settings/domains` 和 `/api/sync` 会返回：

```text
globalEquivalentDomains: []
```

### 实际使用步骤

1. Fork 本仓库到你的 GitHub 账号
2. 在仓库里配置好必需的 GitHub Secrets
3. 如果需要附件，额外配置 `ATTACHMENTS_KV_ID` 或 `R2_NAME`
4. 打开 GitHub 仓库的 `Actions`
5. 手动运行 `Build` 工作流
6. 等待工作流完成

工作流会自动完成这些事：

- 下载并解压 Web Vault 前端
- 删除过大的 source map 文件
- 把 `D1_DATABASE_ID` 写入 `wrangler.toml`
- 把 `ALLOWED_EMAILS`、`JWT_SECRET`、`JWT_REFRESH_SECRET` 上传到 Cloudflare Worker Secrets
- 当数据库为空时自动执行 `sql/schema.sql`
- 自动执行 migrations
- 可选写入全局等价域名
- 自动部署 Worker

### 最短部署路径

如果你只想最快把服务跑起来：

1. Fork 仓库
2. 创建一个 Cloudflare D1 数据库
3. 在 GitHub Secrets 中填入：
   - `CLOUDFLARE_API_TOKEN`
   - `CLOUDFLARE_ACCOUNT_ID`
   - `D1_DATABASE_ID`
   - `ALLOWED_EMAILS`
   - `JWT_SECRET`
   - `JWT_REFRESH_SECRET`
4. 如果需要附件，再加：
   - `ATTACHMENTS_KV_ID`
   - 或 `R2_NAME`
5. 运行 `Build` 工作流
6. 打开部署完成后的 Worker URL
7. 在 Bitwarden 客户端中使用这个地址登录

### 关于 `workers.dev` 地址

如果 `wrangler.toml` 中启用了 `workers_dev = true`，部署完成后你通常可以直接使用：

```text
https://your-worker-name.your-subdomain.workers.dev
```

如果你更希望使用自己的域名，请参考 [README 中的自定义域名说明](../README.md#configure-custom-domain-optional)。

如果你希望前端显示“创建账号”按钮，可以把 `DISABLE_USER_REGISTRATION` 配置为 `false`。详细说明见 [README 的环境变量章节](../README.md#environment-variables)。
