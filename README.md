# Warden

Bitwarden 兼容的密码管理器后端，运行在 Cloudflare Workers 上。

## 功能

- 🔐 密码库管理
- 📱 多客户端支持
- 🗂️ 文件夹管理
- 📎 文件附件
- 🔢 两步验证 (TOTP)

## 快速部署

```bash
# 安装 Wrangler
npm install -g wrangler
wrangler login

# 创建 D1 数据库
wrangler d1 create warden-db

# 设置环境变量
wrangler secret put ALLOWED_EMAILS
wrangler secret put JWT_SECRET
wrangler secret put JWT_REFRESH_SECRET

# 部署
wrangler deploy
```

## License

MIT
