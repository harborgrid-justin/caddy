/**
 * CADDY v0.3.0 - DevTools Entry Point
 */

import './devtools.css';
import './panel';

// Register DevTools panel
chrome.devtools.panels.create(
  'CADDY A11y',
  '/icons/icon-48.png',
  '/devtools.html',
  (panel) => {
    console.log('[CADDY DevTools] Panel created');
  }
);
