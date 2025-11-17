#!/usr/bin/env node

const { execSync } = require('child_process');
const { platform } = require('os');
const path = require('path');

const isWindows = platform() === 'win32';
const nativeDir = path.join(__dirname, '..', 'native');

// On Unix-like systems (macOS, Linux), source cargo env if needed
// On Windows, cargo should already be in PATH from dtolnay/rust-toolchain
let command;

if (isWindows) {
  // Windows: just run npm build in native directory
  command = `cd "${nativeDir}" && npm run build`;
} else {
  // Unix: source cargo env in bash, then run npm build
  command = `bash -c "cd '${nativeDir}' && . $HOME/.cargo/env 2>/dev/null && npm run build"`;
}

try {
  execSync(command, {
    stdio: 'inherit',
    shell: isWindows ? true : '/bin/bash'
  });
} catch (error) {
  process.exit(error.status || 1);
}
