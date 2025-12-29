/**
 * CADDY v0.3.0 - Background Service Worker
 * Enterprise-grade background processing and message routing
 */

import { api } from '../shared/api';
import type {
  Message,
  ScanResult,
  UserSettings,
  BadgeState,
  ScannerConfig,
} from '../shared/types';

// ============================================================================
// State Management
// ============================================================================

interface BackgroundState {
  activeScans: Map<number, AbortController>;
  scanResults: Map<string, ScanResult>;
  settings: UserSettings | null;
  authenticated: boolean;
}

const state: BackgroundState = {
  activeScans: new Map(),
  scanResults: new Map(),
  settings: null,
  authenticated: false,
};

// ============================================================================
// Initialization
// ============================================================================

chrome.runtime.onInstalled.addListener(async (details) => {
  console.log('[CADDY] Extension installed/updated:', details.reason);

  if (details.reason === 'install') {
    // First-time installation
    await initializeExtension();
    chrome.tabs.create({ url: 'options.html' });
  } else if (details.reason === 'update') {
    // Extension updated
    await migrateSettings(details.previousVersion || '0.0.0');
  }

  // Set up context menu
  await setupContextMenu();
});

chrome.runtime.onStartup.addListener(async () => {
  console.log('[CADDY] Browser started, initializing extension');
  await loadSettings();
  await checkAuthentication();
});

// ============================================================================
// Message Handling
// ============================================================================

chrome.runtime.onMessage.addListener((message: Message, sender, sendResponse) => {
  console.log('[CADDY] Message received:', message.type, sender.tab?.id);

  handleMessage(message, sender)
    .then(sendResponse)
    .catch((error) => {
      console.error('[CADDY] Message handler error:', error);
      sendResponse({ success: false, error: error.message });
    });

  // Return true to indicate async response
  return true;
});

async function handleMessage(
  message: Message,
  sender: chrome.runtime.MessageSender
): Promise<any> {
  switch (message.type) {
    case 'SCAN_PAGE':
      return handleScanPage(message, sender);

    case 'GET_SETTINGS':
      return handleGetSettings();

    case 'UPDATE_SETTINGS':
      return handleUpdateSettings(message.payload);

    case 'GET_RESULTS':
      return handleGetResults(message.payload);

    case 'CLEAR_RESULTS':
      return handleClearResults();

    case 'EXPORT_RESULTS':
      return handleExportResults(message.payload);

    case 'AUTHENTICATE':
      return handleAuthenticate(message.payload);

    case 'SYNC_DATA':
      return handleSyncData();

    case 'TOGGLE_HIGHLIGHTS':
      return handleToggleHighlights(sender.tab?.id);

    default:
      throw new Error(`Unknown message type: ${message.type}`);
  }
}

// ============================================================================
// Message Handlers
// ============================================================================

async function handleScanPage(
  message: Message,
  sender: chrome.runtime.MessageSender
): Promise<any> {
  const tabId = sender.tab?.id || message.tabId;
  if (!tabId) {
    throw new Error('No tab ID provided');
  }

  // Cancel any existing scan for this tab
  if (state.activeScans.has(tabId)) {
    state.activeScans.get(tabId)?.abort();
    state.activeScans.delete(tabId);
  }

  // Create abort controller for this scan
  const controller = new AbortController();
  state.activeScans.set(tabId, controller);

  // Update badge to show scanning
  await updateBadge(tabId, {
    text: '...',
    color: '#FFA500',
    title: 'Scanning for accessibility issues',
  });

  try {
    // Inject content script if not already present
    await ensureContentScript(tabId);

    // Send scan message to content script
    const result = await chrome.tabs.sendMessage(tabId, {
      type: 'START_SCAN',
      payload: message.payload,
    });

    // Store result
    const scanResult = result as ScanResult;
    state.scanResults.set(scanResult.url, scanResult);

    // Update badge with issue count
    await updateBadgeFromResult(tabId, scanResult);

    // Upload to server if authenticated
    if (state.authenticated && state.settings?.syncEnabled) {
      await api.uploadScanResult(scanResult).catch(console.error);
    }

    // Show notification if enabled
    if (state.settings?.notifications.onScanComplete) {
      await showNotification(scanResult);
    }

    // Track analytics
    await api.trackEvent('scan_complete', {
      issueCount: scanResult.summary.total,
      url: scanResult.url,
    });

    return { success: true, result: scanResult };
  } catch (error) {
    console.error('[CADDY] Scan error:', error);

    // Update badge to show error
    await updateBadge(tabId, {
      text: '!',
      color: '#FF0000',
      title: 'Scan error',
    });

    throw error;
  } finally {
    state.activeScans.delete(tabId);
  }
}

async function handleGetSettings(): Promise<UserSettings> {
  if (!state.settings) {
    await loadSettings();
  }
  return state.settings || getDefaultSettings();
}

async function handleUpdateSettings(settings: Partial<UserSettings>): Promise<void> {
  state.settings = { ...getDefaultSettings(), ...state.settings, ...settings };
  await chrome.storage.local.set({ settings: state.settings });

  // Sync to server if enabled
  if (state.authenticated && settings.syncEnabled) {
    await api.updateSettings(state.settings).catch(console.error);
  }
}

async function handleGetResults(filter?: { url?: string }): Promise<ScanResult[]> {
  const results = Array.from(state.scanResults.values());

  if (filter?.url) {
    return results.filter((r) => r.url === filter.url);
  }

  return results;
}

async function handleClearResults(): Promise<void> {
  state.scanResults.clear();
  await chrome.storage.local.remove(['scan_results']);
}

async function handleExportResults(options: {
  resultIds: string[];
  format: string;
}): Promise<Blob> {
  return await api.exportResults(options.resultIds, options.format);
}

async function handleAuthenticate(credentials: {
  email?: string;
  password?: string;
  apiKey?: string;
}): Promise<any> {
  try {
    let authResponse;

    if (credentials.apiKey) {
      authResponse = await api.authenticateWithApiKey(credentials.apiKey);
    } else if (credentials.email && credentials.password) {
      authResponse = await api.authenticate(credentials.email, credentials.password);
    } else {
      throw new Error('Invalid credentials');
    }

    state.authenticated = true;

    await chrome.storage.local.set({
      auth: {
        token: authResponse.token,
        userId: authResponse.userId,
        expiresAt: authResponse.expiresAt,
      },
    });

    return { success: true, data: authResponse };
  } catch (error) {
    state.authenticated = false;
    throw error;
  }
}

async function handleSyncData(): Promise<any> {
  if (!state.authenticated) {
    throw new Error('Not authenticated');
  }

  const results = Array.from(state.scanResults.values());
  const settings = state.settings || getDefaultSettings();

  return await api.syncData({
    results,
    settings,
    timestamp: Date.now(),
  });
}

async function handleToggleHighlights(tabId?: number): Promise<void> {
  if (!tabId) return;

  await chrome.tabs.sendMessage(tabId, {
    type: 'TOGGLE_HIGHLIGHTS',
  });
}

// ============================================================================
// Commands
// ============================================================================

chrome.commands.onCommand.addListener(async (command) => {
  console.log('[CADDY] Command received:', command);

  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  if (!tab?.id) return;

  switch (command) {
    case 'scan-page':
      await handleScanPage(
        { type: 'SCAN_PAGE', timestamp: Date.now() },
        { tab }
      );
      break;

    case 'toggle-highlights':
      await handleToggleHighlights(tab.id);
      break;
  }
});

// ============================================================================
// Tab Events
// ============================================================================

chrome.tabs.onUpdated.addListener(async (tabId, changeInfo, tab) => {
  if (changeInfo.status === 'complete' && tab.url) {
    // Auto-scan if enabled
    if (state.settings?.scanning.scanOnLoad) {
      await handleScanPage(
        { type: 'SCAN_PAGE', timestamp: Date.now() },
        { tab }
      );
    }
  }
});

chrome.tabs.onRemoved.addListener((tabId) => {
  // Clean up any active scans
  if (state.activeScans.has(tabId)) {
    state.activeScans.get(tabId)?.abort();
    state.activeScans.delete(tabId);
  }
});

// ============================================================================
// Context Menu
// ============================================================================

async function setupContextMenu(): Promise<void> {
  await chrome.contextMenus.removeAll();

  chrome.contextMenus.create({
    id: 'scan-page',
    title: 'Scan page for accessibility issues',
    contexts: ['page'],
  });

  chrome.contextMenus.create({
    id: 'scan-element',
    title: 'Check element accessibility',
    contexts: ['all'],
  });

  chrome.contextMenus.create({
    id: 'separator',
    type: 'separator',
    contexts: ['all'],
  });

  chrome.contextMenus.create({
    id: 'open-devtools',
    title: 'Open CADDY DevTools',
    contexts: ['all'],
  });
}

chrome.contextMenus.onClicked.addListener(async (info, tab) => {
  if (!tab?.id) return;

  switch (info.menuItemId) {
    case 'scan-page':
      await handleScanPage(
        { type: 'SCAN_PAGE', timestamp: Date.now() },
        { tab }
      );
      break;

    case 'scan-element':
      await chrome.tabs.sendMessage(tab.id, {
        type: 'SCAN_ELEMENT',
        payload: { x: info.x, y: info.y },
      });
      break;

    case 'open-devtools':
      // DevTools will auto-open when navigating to a special URL
      chrome.tabs.create({ url: `devtools.html?tabId=${tab.id}` });
      break;
  }
});

// ============================================================================
// Badge Management
// ============================================================================

async function updateBadge(tabId: number, badge: BadgeState): Promise<void> {
  await chrome.action.setBadgeText({ text: badge.text, tabId });
  await chrome.action.setBadgeBackgroundColor({ color: badge.color, tabId });
  await chrome.action.setTitle({ title: badge.title, tabId });
}

async function updateBadgeFromResult(
  tabId: number,
  result: ScanResult
): Promise<void> {
  const { summary } = result;
  const total = summary.critical + summary.serious;

  let badge: BadgeState;

  if (total === 0) {
    badge = {
      text: '✓',
      color: '#00AA00',
      title: 'No critical accessibility issues found',
    };
  } else if (total < 10) {
    badge = {
      text: total.toString(),
      color: summary.critical > 0 ? '#FF0000' : '#FFA500',
      title: `${total} accessibility issues found`,
    };
  } else {
    badge = {
      text: '9+',
      color: '#FF0000',
      title: `${total} accessibility issues found`,
    };
  }

  await updateBadge(tabId, badge);
}

// ============================================================================
// Notifications
// ============================================================================

async function showNotification(result: ScanResult): Promise<void> {
  const { summary } = result;

  if (summary.critical + summary.serious === 0) return;

  await chrome.notifications.create({
    type: 'basic',
    iconUrl: 'icons/icon-128.png',
    title: 'CADDY Accessibility Scan Complete',
    message: `Found ${summary.critical} critical and ${summary.serious} serious issues`,
    priority: summary.critical > 0 ? 2 : 1,
  });
}

// ============================================================================
// Utility Functions
// ============================================================================

async function initializeExtension(): Promise<void> {
  const defaultSettings = getDefaultSettings();

  await chrome.storage.local.set({
    settings: defaultSettings,
    scan_results: [],
  });

  state.settings = defaultSettings;
}

async function loadSettings(): Promise<void> {
  const result = await chrome.storage.local.get(['settings']);
  state.settings = result.settings || getDefaultSettings();
}

async function checkAuthentication(): Promise<void> {
  const result = await chrome.storage.local.get(['auth']);

  if (result.auth?.token && result.auth.expiresAt > Date.now()) {
    state.authenticated = true;
  } else {
    state.authenticated = false;
  }
}

async function migrateSettings(previousVersion: string): Promise<void> {
  console.log('[CADDY] Migrating settings from version:', previousVersion);
  // Migration logic here if needed
}

async function ensureContentScript(tabId: number): Promise<void> {
  try {
    await chrome.tabs.sendMessage(tabId, { type: 'PING' });
  } catch {
    // Content script not loaded, inject it
    await chrome.scripting.executeScript({
      target: { tabId },
      files: ['content.js'],
    });
  }
}

function getDefaultSettings(): UserSettings {
  return {
    apiEndpoint: 'https://api.caddy.dev/v1',
    syncEnabled: false,
    theme: 'auto',
    notifications: {
      enabled: true,
      onNewIssues: true,
      onScanComplete: true,
      severity: ['critical', 'serious'],
    },
    scanning: {
      autoScan: false,
      scanOnLoad: false,
      scanFrames: true,
      wcagLevel: 'AA',
      maxIssues: 1000,
    },
    keyboard: {
      scanPage: 'Ctrl+Shift+A',
      toggleHighlights: 'Ctrl+Shift+H',
      nextIssue: 'Ctrl+Shift+N',
      previousIssue: 'Ctrl+Shift+P',
    },
  };
}

// ============================================================================
// Export for testing
// ============================================================================

export { state, handleMessage, updateBadge };
