#!/bin/bash
# Test build locally before pushing to GitHub

set -e

echo "🧪 Testing local build (simulating CI/CD)..."
echo ""

# Clean
echo "1️⃣ Cleaning..."
rm -rf dist app .next node_modules/.cache

# Install with frozen lockfile (like CI)
echo ""
echo "2️⃣ Installing dependencies (frozen lockfile)..."
yarn install --frozen-lockfile

# Build native module
echo ""
echo "3️⃣ Building native module..."
yarn build:native

# Build for current platform
echo ""
echo "4️⃣ Building Electron app for $(uname -s)..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    yarn build:mac
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    yarn build:linux
else
    echo "Windows build - run: yarn build:win"
fi

echo ""
echo "✅ Build test completed successfully!"
echo "📦 Check dist/ folder for build artifacts"
