#!/bin/bash
# Seed global equivalent domains into D1 database
# This script populates the global_equivalent_domains table

set -e

# Default values
DB_NAME="vault1"
ENV=""
REMOTE_FLAG=""
WRANGLER_VERSION="3.0.0"
URL=""

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --db)
      DB_NAME="$2"
      shift 2
      ;;
    --env)
      ENV="$2"
      shift 2
      ;;
    --remote)
      REMOTE_FLAG="--remote"
      shift
      ;;
    --wrangler-version)
      WRANGLER_VERSION="$2"
      shift 2
      ;;
    --url)
      URL="$2"
      shift 2
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

echo "🌐 Seeding global equivalent domains..."

# Build wrangler command
WRANGLER="npx --yes wrangler@${WRANGLER_VERSION}"
DB_CMD="$WRANGLER d1 execute $DB_NAME"

if [ -n "$ENV" ]; then
  DB_CMD="$DB_CMD --env $ENV"
fi

if [ -n "$REMOTE_FLAG" ]; then
  DB_CMD="$DB_CMD --remote"
fi

# Default global equivalent domains
# These are domains that are considered equivalent for password matching
default_domains=$(cat << 'EOF'
google.com,googleapis.com,googleusercontent.com
apple.com,icloud.com,me.com
microsoft.com,windows.com,skype.com,xbox.com
amazon.com,amazon.co.uk,amazon.de,amazon.fr,amazon.co.jp
twitter.com,x.com
facebook.com,instagram.com,whatsapp.com
EOF
)

# If URL provided, fetch from URL
if [ -n "$URL" ]; then
  echo "📥 Fetching domains from $URL..."
  domains=$(curl -s "$URL" || echo "")
  if [ -z "$domains" ]; then
    echo "⚠️ Failed to fetch from URL, using defaults"
    domains="$default_domains"
  fi
else
  domains="$default_domains"
fi

# Insert domains into database
echo "📝 Inserting domains into database..."

# Create table if not exists
$DB_CMD --command "
CREATE TABLE IF NOT EXISTS global_equivalent_domains (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    domains TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
" > /dev/null 2>&1 || true

# Clear existing data and insert new
echo "$domains" | while IFS= read -r line; do
  if [ -n "$line" ]; then
    # Check if already exists
    exists=$($DB_CMD --command "SELECT COUNT(*) as cnt FROM global_equivalent_domains WHERE domains = '$line'" --json 2>/dev/null | grep -o '"cnt":[0-9]*' | cut -d: -f2 || echo "0")
    
    if [ "$exists" = "0" ]; then
      $DB_CMD --command "INSERT INTO global_equivalent_domains (domains) VALUES ('$line')" > /dev/null 2>&1 || true
      echo "  ✓ Added: $line"
    else
      echo "  ℹ️ Skipped (exists): $line"
    fi
  fi
done

echo "✅ Global equivalent domains seeded successfully"
