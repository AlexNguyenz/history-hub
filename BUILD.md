# Build Instructions

This project uses Rust native modules, so building for different platforms requires special consideration.

## Prerequisites

- Node.js 20+
- Yarn
- Rust toolchain (rustc, cargo)

## Local Development

```bash
# Install dependencies
yarn install

# Build native module
yarn build:native

# Run in development mode
yarn dev
```

## Building for Production

### Current Platform

Build for your current platform (macOS, Windows, or Linux):

```bash
yarn build
```

This will:
1. Build the Rust native module for your current platform
2. Build the Electron app for your current platform

### Specific Platforms

```bash
# macOS only (DMG for x64 and arm64)
yarn build:mac

# Windows only (NSIS installer for x64 and ia32)
yarn build:win

# Linux only (AppImage and deb)
yarn build:linux
```

**Important**: You can only build native modules for your current platform locally. For example, if you're on macOS, `yarn build:win` will fail because the Rust native module can't be cross-compiled easily.

## Cross-Platform Builds with GitHub Actions

To build for all platforms, use GitHub Actions:

1. Push your code to GitHub
2. Create a tag: `git tag v1.0.0 && git push --tags`
3. GitHub Actions will automatically build for macOS, Windows, and Linux
4. Download the artifacts from the Actions tab or the Release page

Or manually trigger the build:
- Go to the "Actions" tab in your GitHub repository
- Select "Build and Release"
- Click "Run workflow"

## Output

Built applications will be in the `dist/` directory:

- **macOS**: `*.dmg` files
- **Windows**: `*.exe` installer
- **Linux**: `*.AppImage` and `*.deb` files

## Native Module Architecture

The Rust native module (`claude-parser`) is built separately for each platform:

- **macOS**: `claude-parser.darwin-x64.node` (Intel) and `claude-parser.darwin-arm64.node` (Apple Silicon)
- **Windows**: `claude-parser.win32-x64-msvc.node` (64-bit) and `claude-parser.win32-ia32-msvc.node` (32-bit)
- **Linux**: `claude-parser.linux-x64-gnu.node`

The correct binary is automatically loaded at runtime based on the platform.

## Troubleshooting

### "cargo: command not found"

Install Rust: https://rustup.rs/

### Native module build fails

Make sure you have:
- A working Rust installation: `rustc --version`
- The correct build tools for your platform:
  - macOS: Xcode Command Line Tools
  - Windows: Visual Studio Build Tools
  - Linux: build-essential package

### Electron build fails

Try cleaning and rebuilding:

```bash
rm -rf node_modules dist app .next
yarn install
yarn build:native
yarn build
```
