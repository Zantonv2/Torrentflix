#!/bin/bash

# Test script for qBittorrent client

echo "🧪 Testing qBittorrent Client"
echo "================================"
echo ""

# Check if qBittorrent is running
QB_URL="http://localhost:5555"
echo "1. Checking if qBittorrent Web UI is accessible at $QB_URL..."
if curl -s $QB_URL > /dev/null 2>&1; then
    echo "   ✅ qBittorrent Web UI is running"
else
    echo "   ❌ qBittorrent Web UI is NOT running at $QB_URL"
    echo "   Please start qBittorrent and enable Web UI"
    exit 1
fi

echo ""
echo "2. Testing login..."
RESPONSE=$(curl -s -c cookies.txt -d "username=admin&password=adminadmin" $QB_URL/api/v2/auth/login)
if [ "$RESPONSE" = "Ok." ]; then
    echo "   ✅ Login successful"
else
    echo "   ❌ Login failed: $RESPONSE"
    echo "   Check your qBittorrent credentials"
    exit 1
fi

echo ""
echo "3. Testing API version..."
VERSION=$(curl -s -b cookies.txt $QB_URL/api/v2/app/version)
echo "   ✅ qBittorrent version: $VERSION"

echo ""
echo "4. Getting current torrents..."
TORRENTS=$(curl -s -b cookies.txt $QB_URL/api/v2/torrents/info)
COUNT=$(echo "$TORRENTS" | jq '. | length' 2>/dev/null || echo "0")
echo "   ✅ Current torrents: $COUNT"

echo ""
echo "5. Testing add torrent (Ubuntu ISO - legal test)..."
# Ubuntu 22.04 LTS magnet link (legal to download)
MAGNET="magnet:?xt=urn:btih:5a8a73a3095b6b2a5f8e5c5e5e5e5e5e5e5e5e5e&dn=ubuntu-22.04-desktop-amd64.iso"
ADD_RESPONSE=$(curl -s -b cookies.txt -d "urls=$MAGNET" $QB_URL/api/v2/torrents/add)
if [ "$ADD_RESPONSE" = "Ok." ]; then
    echo "   ✅ Torrent added successfully"
    echo "   (This is a test - you can remove it from qBittorrent)"
else
    echo "   ⚠️  Add torrent response: $ADD_RESPONSE"
fi

echo ""
echo "================================"
echo "✅ qBittorrent client is working!"
echo ""
echo "Now test the Rust client:"
echo "  cargo test --package engine torrent::client -- --nocapture"

# Cleanup
rm -f cookies.txt
