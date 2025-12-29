/**
 * CADDY v0.3.0 - Content Script Entry Point
 * Main entry for content script injection
 */

import { AccessibilityScanner } from './scanner';
import { highlighter } from './highlighter';
import type { Message, ScanResult } from '../shared/types';

// ============================================================================
// Content Script Initialization
// ============================================================================

console.log('[CADDY Content] Initializing content script');

let scanner: AccessibilityScanner | null = null;
let currentScanResult: ScanResult | null = null;
let highlightsEnabled = true;

// ============================================================================
// Message Handler
// ============================================================================

chrome.runtime.onMessage.addListener((message: Message, sender, sendResponse) => {
  console.log('[CADDY Content] Message received:', message.type);

  handleMessage(message)
    .then(sendResponse)
    .catch((error) => {
      console.error('[CADDY Content] Message handler error:', error);
      sendResponse({ success: false, error: error.message });
    });

  // Return true to indicate async response
  return true;
});

async function handleMessage(message: Message): Promise<any> {
  switch (message.type) {
    case 'PING':
      return { success: true, pong: true };

    case 'START_SCAN':
      return await handleStartScan();

    case 'TOGGLE_HIGHLIGHTS':
      return handleToggleHighlights();

    case 'HIGHLIGHT_ELEMENT':
      return handleHighlightElement(message.payload);

    case 'SCAN_ELEMENT':
      return handleScanElement(message.payload);

    default:
      throw new Error(`Unknown message type: ${message.type}`);
  }
}

// ============================================================================
// Scan Handlers
// ============================================================================

async function handleStartScan(): Promise<ScanResult> {
  console.log('[CADDY Content] Starting scan...');

  // Create scanner if not exists
  if (!scanner) {
    scanner = new AccessibilityScanner();
  }

  // Run scan
  const result = await scanner.scan();
  currentScanResult = result;

  // Highlight issues if enabled
  if (highlightsEnabled) {
    highlighter.highlightIssues(result.issues);
  }

  console.log('[CADDY Content] Scan complete:', result.summary);

  return result;
}

function handleToggleHighlights(): any {
  highlightsEnabled = !highlightsEnabled;
  highlighter.toggle();

  return {
    success: true,
    enabled: highlightsEnabled,
  };
}

function handleHighlightElement(payload: { selector: string }): any {
  try {
    const element = document.querySelector(payload.selector);
    if (element) {
      element.scrollIntoView({ behavior: 'smooth', block: 'center' });

      // Flash highlight
      const originalOutline = (element as HTMLElement).style.outline;
      (element as HTMLElement).style.outline = '3px solid #3B82F6';
      setTimeout(() => {
        (element as HTMLElement).style.outline = originalOutline;
      }, 2000);

      return { success: true };
    }

    return { success: false, error: 'Element not found' };
  } catch (error) {
    return { success: false, error: (error as Error).message };
  }
}

function handleScanElement(payload: { x: number; y: number }): any {
  const element = document.elementFromPoint(payload.x, payload.y);

  if (!element || !(element instanceof HTMLElement)) {
    return { success: false, error: 'No element at coordinates' };
  }

  // Find issues for this element
  const issues =
    currentScanResult?.issues.filter(
      (issue) =>
        document.querySelector(issue.selector) === element ||
        element.contains(document.querySelector(issue.selector))
    ) || [];

  return {
    success: true,
    element: {
      tagName: element.tagName,
      id: element.id,
      className: element.className,
    },
    issues,
  };
}

// ============================================================================
// Lifecycle
// ============================================================================

// Auto-scan on load if configured
chrome.storage.local.get(['settings'], (result) => {
  if (result.settings?.scanning?.scanOnLoad) {
    // Wait for page to be fully loaded
    if (document.readyState === 'complete') {
      handleStartScan();
    } else {
      window.addEventListener('load', () => handleStartScan());
    }
  }
});

// Cleanup on unload
window.addEventListener('beforeunload', () => {
  highlighter.destroy();
  scanner = null;
  currentScanResult = null;
});

// Export for testing
export { handleMessage, handleStartScan };
