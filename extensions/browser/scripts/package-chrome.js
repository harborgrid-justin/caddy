#!/usr/bin/env node

/**
 * CADDY v0.3.0 - Chrome Extension Packager
 * Creates a .zip file ready for Chrome Web Store upload
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const DIST_DIR = path.join(__dirname, '../dist');
const OUTPUT_DIR = path.join(__dirname, '../packages');
const VERSION = require('../package.json').version;
const OUTPUT_FILE = path.join(OUTPUT_DIR, `caddy-chrome-v${VERSION}.zip`);

console.log('📦 Packaging CADDY for Chrome...\n');

// Ensure dist exists
if (!fs.existsSync(DIST_DIR)) {
  console.error('❌ Error: dist directory not found. Run "npm run build" first.');
  process.exit(1);
}

// Create output directory
if (!fs.existsSync(OUTPUT_DIR)) {
  fs.mkdirSync(OUTPUT_DIR, { recursive: true });
}

// Remove old package
if (fs.existsSync(OUTPUT_FILE)) {
  fs.unlinkSync(OUTPUT_FILE);
}

try {
  // Create zip archive
  console.log('Creating ZIP archive...');
  execSync(`cd ${DIST_DIR} && zip -r "${OUTPUT_FILE}" . -x "*.map" "*.ts"`, {
    stdio: 'inherit',
  });

  const stats = fs.statSync(OUTPUT_FILE);
  const fileSizeInMB = (stats.size / (1024 * 1024)).toFixed(2);

  console.log('\n✅ Chrome package created successfully!');
  console.log(`📦 Package: ${OUTPUT_FILE}`);
  console.log(`📊 Size: ${fileSizeInMB} MB`);
  console.log('\n📝 Next steps:');
  console.log('1. Go to https://chrome.google.com/webstore/devconsole');
  console.log('2. Create new item or update existing');
  console.log(`3. Upload ${path.basename(OUTPUT_FILE)}`);
} catch (error) {
  console.error('❌ Error creating package:', error.message);
  process.exit(1);
}
