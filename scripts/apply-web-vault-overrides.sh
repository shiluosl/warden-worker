#!/bin/bash
# Apply web vault CSS overrides
# This script applies custom CSS overrides to the web vault

set -e

WEB_VAULT_DIR="${1:-public/web-vault}"

if [ ! -d "$WEB_VAULT_DIR" ]; then
    echo "❌ Web vault directory not found: $WEB_VAULT_DIR"
    exit 1
fi

echo "🎨 Applying web vault overrides..."

# Create custom CSS file
cat > "$WEB_VAULT_DIR/vaultwarden.css" << 'EOF'
/* Custom overrides for vaultwarden web vault */

/* Hide organization-related UI elements (not supported) */
app-organization-layout,
app-org-vault,
app-org-members,
app-org-collections,
app-org-groups,
app-org-policies,
app-org-events,
app-org-export,
app-org-settings,
app-org-tools,
app-org-billing,
[ng-reflect-router-link*="/organizations"],
a[href*="/organizations"],
button[data-toggle="modal"][data-target*="#organization"],
.org-invite-banner,
.org-upgrade-banner {
    display: none !important;
}

/* Hide SSO-related elements (not supported) */
app-sso,
.enterpriseSSO,
[data-route*="sso"],
button[data-toggle="modal"][data-target*="#sso"],
a[href*="/sso"] {
    display: none !important;
}

/* Hide emergency access banner (optional) */
.emergency-access-banner {
    display: none !important;
}

/* Custom branding (optional) */
.logo-brand {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
}
EOF

echo "  ✓ Created vaultwarden.css"

# Inject CSS into index.html
INDEX_FILE="$WEB_VAULT_DIR/index.html"
if [ -f "$INDEX_FILE" ]; then
    # Check if already injected
    if ! grep -q "vaultwarden.css" "$INDEX_FILE"; then
        # Inject CSS link before closing head tag
        sed -i 's|</head>|<link rel="stylesheet" href="vaultwarden.css"></head>|' "$INDEX_FILE"
        echo "  ✓ Injected CSS into index.html"
    else
        echo "  ℹ️ CSS already injected"
    fi
fi

echo "✅ Web vault overrides applied successfully"
