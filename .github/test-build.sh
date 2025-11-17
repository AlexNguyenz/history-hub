#!/bin/bash
# Script to test GitHub Actions build locally using act
# Install act: brew install act

set -e

echo "🧪 Testing GitHub Actions build locally..."
echo ""

# Check if act is installed
if ! command -v act &> /dev/null; then
    echo "❌ 'act' is not installed. Install it with:"
    echo "   brew install act"
    exit 1
fi

echo "✅ act is installed"
echo ""

# Test build workflow
echo "🚀 Running build workflow locally..."
echo "   Platform: $(uname -s)"
echo ""

# Run the workflow (will build for current platform only)
act push \
    --workflows .github/workflows/build.yml \
    --eventpath <(echo '{"ref":"refs/tags/v1.1.0"}') \
    --verbose

echo ""
echo "✅ Build test completed!"
