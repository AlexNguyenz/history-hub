#!/usr/bin/env node

const { execSync } = require('child_process');
const { platform } = require('os');
const path = require('path');

// Change to native directory
process.chdir(path.join(__dirname, '..', 'native'));

// On Unix-like systems (macOS, Linux), source cargo env if needed
// On Windows, cargo should already be in PATH from dtolnay/rust-toolchain
const isWindows = platform() === 'win32';

let command = 'npm run build';

if (!isWindows) {
  // On Unix, try to source cargo env first
  command = `. $HOME/.cargo/env 2>/dev/null || true && ${command}`;
}

try {
  execSync(command, {
    stdio: 'inherit',
    shell: isWindows ? 'cmd.exe' : '/bin/bash'
  });
} catch (error) {
  process.exit(error.status || 1);
}
