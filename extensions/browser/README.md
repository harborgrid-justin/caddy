# CADDY Browser Extension v0.3.0

Enterprise-grade accessibility scanner browser extension for Chrome, Firefox, and Edge.

## Features

### Real-Time Accessibility Scanning
- **Comprehensive WCAG Analysis**: Full WCAG 2.1 Level A, AA, and AAA compliance checking
- **30+ Accessibility Rules**: Covering images, forms, headings, links, ARIA, color contrast, keyboard accessibility, and more
- **DOM Inspection**: Deep analysis of HTML structure and accessibility tree
- **Performance Optimized**: Efficient scanning engine with minimal performance impact

### Visual Issue Highlighting
- **Color-Coded Overlays**: Issues highlighted by severity (critical, serious, moderate, minor)
- **Interactive Tooltips**: Detailed issue information on hover
- **Element Focus**: Click to scroll and highlight problematic elements
- **Smart Positioning**: Tooltips automatically adjust to viewport

### Quick Access Popup
- **One-Click Scanning**: Scan any page instantly
- **Compliance Score**: Visual representation of accessibility score
- **Issue Summary**: Quick overview of issues by severity
- **Recent Results**: Access previously scanned pages

### Advanced DevTools Panel
- **Full Inspector Interface**: Comprehensive accessibility debugging
- **Issue Filtering**: Filter by severity and category
- **Element Inspector**: Detailed element properties and attributes
- **Accessibility Tree**: Visual representation of the a11y tree
- **Console Integration**: Debug messages and warnings

### Comprehensive Settings
- **WCAG Level Selection**: Choose between A, AA, or AAA conformance
- **Auto-Scan Options**: Scan on page load or manually
- **Notification Preferences**: Customize alert settings
- **Cloud Sync**: Sync results and settings across devices (Pro)
- **API Integration**: Connect to enterprise accessibility platforms

## Installation

### Development Setup

1. **Clone the repository**:
   ```bash
   cd /home/user/caddy/extensions/browser
   ```

2. **Install dependencies**:
   ```bash
   npm install
   ```

3. **Build the extension**:
   ```bash
   # Development build with watch mode
   npm run dev

   # Production build
   npm run build
   ```

4. **Load in browser**:

   **Chrome/Edge**:
   - Navigate to `chrome://extensions/` (or `edge://extensions/`)
   - Enable "Developer mode"
   - Click "Load unpacked"
   - Select the `dist` folder

   **Firefox**:
   - Navigate to `about:debugging#/runtime/this-firefox`
   - Click "Load Temporary Add-on"
   - Select any file in the `dist` folder

### Production Builds

Build for all browsers:
```bash
npm run build:all
```

Or build for specific browsers:
```bash
npm run build:chrome   # Chrome/Chromium
npm run build:firefox  # Firefox
npm run build:edge     # Microsoft Edge
```

## Usage

### Quick Scan
1. Click the CADDY extension icon in your browser toolbar
2. Click "Scan Page" button
3. Review the compliance score and issue summary
4. Toggle highlights to see issues on the page

### Keyboard Shortcuts
- `Ctrl+Shift+A` (Mac: `Cmd+Shift+A`) - Scan current page
- `Ctrl+Shift+H` (Mac: `Cmd+Shift+H`) - Toggle issue highlights

### DevTools Panel
1. Open browser DevTools (F12)
2. Navigate to the "CADDY A11y" tab
3. Use the full inspection interface to debug accessibility issues

### Settings
1. Right-click the extension icon
2. Select "Options"
3. Configure scanning preferences, notifications, and account settings

## Architecture

### File Structure
```
extensions/browser/
├── manifest.json              # Extension manifest (Manifest V3)
├── src/
│   ├── background/
│   │   └── index.ts          # Background service worker
│   ├── content/
│   │   ├── index.ts          # Content script entry
│   │   ├── scanner.ts        # Accessibility scanning engine
│   │   ├── highlighter.ts    # Issue highlighting system
│   │   └── content.css       # Overlay styles
│   ├── popup/
│   │   ├── Popup.tsx         # React popup component
│   │   ├── index.tsx         # Popup entry point
│   │   ├── popup.html        # Popup HTML template
│   │   └── popup.css         # Popup styles
│   ├── devtools/
│   │   ├── panel.ts          # DevTools panel logic
│   │   ├── index.ts          # DevTools entry point
│   │   ├── devtools.html     # DevTools HTML template
│   │   └── devtools.css      # DevTools styles
│   ├── options/
│   │   ├── Options.tsx       # React options component
│   │   ├── index.tsx         # Options entry point
│   │   ├── options.html      # Options HTML template
│   │   └── options.css       # Options styles
│   └── shared/
│       ├── types.ts          # TypeScript type definitions
│       └── api.ts            # API client
├── package.json              # NPM dependencies
├── tsconfig.json             # TypeScript configuration
└── webpack.config.js         # Webpack build configuration
```

### Technology Stack
- **TypeScript**: Type-safe development
- **React**: UI components (Popup, Options)
- **Webpack**: Module bundling and build pipeline
- **Chrome Extension APIs**: Browser integration

### Accessibility Rules
The scanner implements 30+ rules covering:
- **Images**: Alt text, decorative images, role attributes
- **Forms**: Labels, field types, required fields, fieldset/legend
- **Headings**: Logical order, hierarchy, empty headings
- **Links**: Descriptive text, purpose, skip links
- **ARIA**: Valid attributes, required properties, parent/child relationships
- **Color Contrast**: WCAG AA/AAA ratios for text and UI elements
- **Keyboard**: Tab navigation, focus management, interactive elements
- **Semantic HTML**: Landmarks, lists, tables, document structure
- **Media**: Transcripts, captions, autoplay controls
- **Language**: Lang attributes, language changes

## API Integration

### Authentication
```typescript
// Connect with API key
await chrome.runtime.sendMessage({
  type: 'AUTHENTICATE',
  payload: { apiKey: 'your-api-key' },
});
```

### Upload Scan Results
```typescript
// Results are automatically synced if authenticated and sync is enabled
// Manual upload:
await chrome.runtime.sendMessage({
  type: 'SYNC_DATA',
});
```

### Custom API Endpoint
For enterprise deployments, configure a custom API endpoint in Settings.

## Development

### Type Checking
```bash
npm run type-check
```

### Linting
```bash
npm run lint
npm run lint:fix
```

### Testing
```bash
npm test
npm run test:watch
npm run test:coverage
```

### Clean Build
```bash
npm run clean
npm run build
```

## Browser Compatibility

- ✅ Chrome 109+
- ✅ Edge 109+
- ✅ Firefox 109+
- ✅ Opera 95+
- ✅ Brave (Chromium-based)

## Performance

- **Scan Time**: ~500ms - 2s (depending on page complexity)
- **Memory Usage**: ~20-50MB
- **CPU Impact**: Minimal during scan, zero when idle
- **DOM Nodes**: Can handle 10,000+ elements efficiently

## Privacy & Security

- **No Data Collection**: No telemetry or analytics without explicit opt-in
- **Local Processing**: All scans run locally in the browser
- **Optional Cloud Sync**: Cloud features require explicit authentication
- **Secure Communication**: All API calls use HTTPS with token authentication
- **Content Security Policy**: Strict CSP enforced

## Contributing

We welcome contributions! Please see our [Contributing Guide](../../CONTRIBUTING.md).

## License

MIT License - see [LICENSE](../../LICENSE) for details.

## Support

- **Documentation**: https://caddy.dev/docs
- **Issues**: https://github.com/harborgrid-justin/caddy/issues
- **Email**: support@caddy.dev

## Changelog

### v0.3.0 (2025-12-29)
- Initial release
- Complete accessibility scanning engine
- Visual issue highlighting
- Quick access popup
- DevTools panel
- Settings page
- API integration
- Cloud sync support

---

**Built with ❤️ by the CADDY Team**
