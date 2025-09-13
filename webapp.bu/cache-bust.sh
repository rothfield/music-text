#!/bin/bash
# Update cache-busting versions for web assets

cd "$(dirname "$0")"
node update-cache-bust.js
echo "Cache-busting versions updated. Browsers will reload updated assets."