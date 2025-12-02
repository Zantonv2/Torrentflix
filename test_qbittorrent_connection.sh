#!/bin/bash

echo "Testing qBittorrent Web UI connection on port 5555..."
echo ""

# Test if port is open
if curl -s http://127.0.0.1:5555/api/v2/app/version > /dev/null 2>&1; then
    echo "✅ qBittorrent Web UI is accessible on port 5555"
    VERSION=$(curl -s http://127.0.0.1:5555/api/v2/app/version)
    echo "   Version: $VERSION"
else
    echo "❌ Cannot connect to qBittorrent Web UI on port 5555"
    echo ""
    echo "To fix this:"
    echo "1. Open qBittorrent"
    echo "2. Go to Tools > Options > Web UI"
    echo "3. Enable 'Web User Interface (Remote control)'"
    echo "4. Set port to 5555"
    echo "5. Set username: admin"
    echo "6. Set password: adminadmin"
    echo "7. Click OK and restart qBittorrent"
fi

echo ""
