#!/usr/bin/env node

/**
 * CADDY v0.3.0 - Edge Extension Packager
 * Creates a .zip file ready for Edge Add-ons upload
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const DIST_DIR = path.join(__dirname, '../dist');
const OUTPUT_DIR = path.join(__dirname, '../packages');
const VERSION = require('../package.json').version;
const OUTPUT_FILE = path.join(OUTPUT_DIR, `caddy-edge-v${VERSION}.zip`);

console.log('🌊 Packaging CADDY for Edge...\n');

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

  console.log('\n✅ Edge package created successfully!');
  console.log(`📦 Package: ${OUTPUT_FILE}`);
  console.log(`📊 Size: ${fileSizeInMB} MB`);
  console.log('\n📝 Next steps:');
  console.log('1. Go to https://partner.microsoft.com/dashboard/microsoftedge');
  console.log('2. Create new extension or update existing');
  console.log(`3. Upload ${path.basename(OUTPUT_FILE)}`);
} catch (error) {
  console.error('❌ Error creating package:', error.message);
  process.exit(1);
}
