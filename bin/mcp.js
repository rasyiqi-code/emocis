#!/usr/bin/env node
const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

const binName = 'emocis';
const possiblePaths = [
  path.join(__dirname, '..', 'target', 'release', binName),
  path.join(__dirname, '..', 'target', 'debug', binName),
];

const binaryPath = possiblePaths.find(p => fs.existsSync(p));

if (!binaryPath) {
  console.error(`Binary '${binName}' tidak ditemukan. Silakan jalankan 'cargo build' atau 'bun run build' terlebih dahulu.`);
  process.exit(1);
}

const child = spawn(binaryPath, ['mcp', ...process.argv.slice(2)], { stdio: 'inherit' });
child.on('exit', (code) => process.exit(code || 0));
