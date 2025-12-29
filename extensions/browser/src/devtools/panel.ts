/**
 * CADDY v0.3.0 - DevTools Panel
 * Advanced accessibility inspection and analysis interface
 */

import type {
  AccessibilityIssue,
  AccessibilityNode,
  ScanResult,
  InspectorState,
} from '../shared/types';

// ============================================================================
// DevTools Panel Class
// ============================================================================

class DevToolsPanel {
  private tabId: number;
  private port: chrome.runtime.Port | null = null;
  private state: InspectorState;
  private scanResult: ScanResult | null = null;
  private accessibilityTree: AccessibilityNode | null = null;

  constructor(tabId: number) {
    this.tabId = tabId;
    this.state = {
      highlightEnabled: true,
      treeExpanded: false,
    };

    this.init();
  }

  // --------------------------------------------------------------------------
  // Initialization
  // --------------------------------------------------------------------------

  private async init(): Promise<void> {
    console.log('[CADDY DevTools] Initializing panel for tab:', this.tabId);

    // Connect to background script
    this.port = chrome.runtime.connect({ name: 'devtools-panel' });
    this.port.postMessage({ type: 'INIT', tabId: this.tabId });

    // Listen for messages
    this.port.onMessage.addListener((message) => this.handleMessage(message));

    // Set up UI
    this.setupUI();

    // Load initial data
    await this.loadInitialData();

    // Set up event listeners
    this.setupEventListeners();
  }

  // --------------------------------------------------------------------------
  // UI Setup
  // --------------------------------------------------------------------------

  private setupUI(): void {
    const container = document.getElementById('app');
    if (!container) return;

    container.innerHTML = `
      <div class="devtools-container">
        <!-- Header -->
        <div class="devtools-header">
          <div class="header-left">
            <img src="/icons/icon-48.png" alt="CADDY" class="logo" />
            <h1 class="title">CADDY Accessibility Inspector</h1>
          </div>
          <div class="header-right">
            <button id="scan-button" class="btn btn-primary">
              <svg width="16" height="16" fill="currentColor" viewBox="0 0 16 16">
                <path d="M11.534 7h3.932a.25.25 0 0 1 .192.41l-1.966 2.36a.25.25 0 0 1-.384 0l-1.966-2.36a.25.25 0 0 1 .192-.41zm-11 2h3.932a.25.25 0 0 0 .192-.41L2.692 6.23a.25.25 0 0 0-.384 0L.342 8.59A.25.25 0 0 0 .534 9z"/>
                <path fill-rule="evenodd" d="M8 3c-1.552 0-2.94.707-3.857 1.818a.5.5 0 1 1-.771-.636A6.002 6.002 0 0 1 13.917 7H12.9A5.002 5.002 0 0 0 8 3zM3.1 9a5.002 5.002 0 0 0 8.757 2.182.5.5 0 1 1 .771.636A6.002 6.002 0 0 1 2.083 9H3.1z"/>
              </svg>
              Scan Page
            </button>
            <button id="export-button" class="btn btn-secondary">Export</button>
          </div>
        </div>

        <!-- Main Content -->
        <div class="devtools-main">
          <!-- Sidebar -->
          <div class="sidebar">
            <div class="sidebar-tabs">
              <button class="tab-button active" data-tab="issues">
                Issues <span id="issues-count" class="badge">0</span>
              </button>
              <button class="tab-button" data-tab="tree">
                A11y Tree
              </button>
              <button class="tab-button" data-tab="console">
                Console
              </button>
            </div>

            <!-- Issues Tab -->
            <div id="issues-tab" class="tab-content active">
              <div class="filters">
                <select id="severity-filter" class="filter-select">
                  <option value="all">All Severities</option>
                  <option value="critical">Critical</option>
                  <option value="serious">Serious</option>
                  <option value="moderate">Moderate</option>
                  <option value="minor">Minor</option>
                </select>
                <select id="category-filter" class="filter-select">
                  <option value="all">All Categories</option>
                  <option value="perceivable">Perceivable</option>
                  <option value="operable">Operable</option>
                  <option value="understandable">Understandable</option>
                  <option value="robust">Robust</option>
                </select>
              </div>
              <div id="issues-list" class="issues-list">
                <div class="empty-state">
                  <svg width="64" height="64" fill="currentColor" viewBox="0 0 16 16">
                    <path d="M8 15A7 7 0 1 1 8 1a7 7 0 0 1 0 14zm0 1A8 8 0 1 0 8 0a8 8 0 0 0 0 16z"/>
                    <path d="M7.002 11a1 1 0 1 1 2 0 1 1 0 0 1-2 0zM7.1 4.995a.905.905 0 1 1 1.8 0l-.35 3.507a.552.552 0 0 1-1.1 0L7.1 4.995z"/>
                  </svg>
                  <p>No issues found. Click "Scan Page" to start.</p>
                </div>
              </div>
            </div>

            <!-- Tree Tab -->
            <div id="tree-tab" class="tab-content">
              <div id="tree-view" class="tree-view">
                <div class="empty-state">
                  <p>Accessibility tree will appear after scanning.</p>
                </div>
              </div>
            </div>

            <!-- Console Tab -->
            <div id="console-tab" class="tab-content">
              <div id="console-output" class="console-output"></div>
            </div>
          </div>

          <!-- Inspector -->
          <div class="inspector">
            <div class="inspector-header">
              <h3>Element Inspector</h3>
              <button id="highlight-toggle" class="btn-icon" title="Toggle highlights">
                <svg width="16" height="16" fill="currentColor" viewBox="0 0 16 16">
                  <path d="M16 8s-3-5.5-8-5.5S0 8 0 8s3 5.5 8 5.5S16 8 16 8zM1.173 8a13.133 13.133 0 0 1 1.66-2.043C4.12 4.668 5.88 3.5 8 3.5c2.12 0 3.879 1.168 5.168 2.457A13.133 13.133 0 0 1 14.828 8c-.058.087-.122.183-.195.288-.335.48-.83 1.12-1.465 1.755C11.879 11.332 10.119 12.5 8 12.5c-2.12 0-3.879-1.168-5.168-2.457A13.134 13.134 0 0 1 1.172 8z"/>
                  <path d="M8 5.5a2.5 2.5 0 1 0 0 5 2.5 2.5 0 0 0 0-5zM4.5 8a3.5 3.5 0 1 1 7 0 3.5 3.5 0 0 1-7 0z"/>
                </svg>
              </button>
            </div>
            <div id="inspector-content" class="inspector-content">
              <div class="empty-state">
                <p>Select an issue to inspect the element.</p>
              </div>
            </div>
          </div>
        </div>

        <!-- Status Bar -->
        <div class="status-bar">
          <span id="status-text">Ready</span>
          <span id="scan-duration"></span>
        </div>
      </div>
    `;
  }

  // --------------------------------------------------------------------------
  // Event Listeners
  // --------------------------------------------------------------------------

  private setupEventListeners(): void {
    // Scan button
    document.getElementById('scan-button')?.addEventListener('click', () => {
      this.handleScan();
    });

    // Export button
    document.getElementById('export-button')?.addEventListener('click', () => {
      this.handleExport();
    });

    // Highlight toggle
    document.getElementById('highlight-toggle')?.addEventListener('click', () => {
      this.toggleHighlights();
    });

    // Tab switching
    document.querySelectorAll('.tab-button').forEach((button) => {
      button.addEventListener('click', (e) => {
        const target = e.currentTarget as HTMLElement;
        const tabName = target.dataset.tab;
        if (tabName) this.switchTab(tabName);
      });
    });

    // Filters
    document.getElementById('severity-filter')?.addEventListener('change', () => {
      this.filterIssues();
    });

    document.getElementById('category-filter')?.addEventListener('change', () => {
      this.filterIssues();
    });
  }

  // --------------------------------------------------------------------------
  // Message Handling
  // --------------------------------------------------------------------------

  private handleMessage(message: any): void {
    console.log('[CADDY DevTools] Message received:', message.type);

    switch (message.type) {
      case 'SCAN_COMPLETE':
        this.handleScanComplete(message.payload);
        break;

      case 'ELEMENT_SELECTED':
        this.handleElementSelected(message.payload);
        break;

      case 'CONSOLE_MESSAGE':
        this.addConsoleMessage(message.payload);
        break;
    }
  }

  // --------------------------------------------------------------------------
  // Scan Handling
  // --------------------------------------------------------------------------

  private async handleScan(): Promise<void> {
    this.updateStatus('Scanning...', true);

    try {
      const response = await chrome.runtime.sendMessage({
        type: 'SCAN_PAGE',
        tabId: this.tabId,
        timestamp: Date.now(),
      });

      if (response.success) {
        this.handleScanComplete(response.result);
      } else {
        this.updateStatus('Scan failed: ' + response.error, false);
      }
    } catch (error) {
      console.error('[CADDY DevTools] Scan error:', error);
      this.updateStatus('Scan error', false);
    }
  }

  private handleScanComplete(result: ScanResult): void {
    this.scanResult = result;
    this.renderIssues(result.issues);
    this.buildAccessibilityTree();
    this.updateStatus(
      `Scan complete: ${result.summary.total} issues found`,
      false
    );
    this.updateScanDuration(result.duration);

    // Update badge
    const badge = document.getElementById('issues-count');
    if (badge) badge.textContent = result.summary.total.toString();
  }

  // --------------------------------------------------------------------------
  // Issues Rendering
  // --------------------------------------------------------------------------

  private renderIssues(issues: AccessibilityIssue[]): void {
    const container = document.getElementById('issues-list');
    if (!container) return;

    if (issues.length === 0) {
      container.innerHTML = `
        <div class="empty-state">
          <svg width="64" height="64" fill="currentColor" viewBox="0 0 16 16">
            <path d="M8 15A7 7 0 1 1 8 1a7 7 0 0 1 0 14zm0 1A8 8 0 1 0 8 0a8 8 0 0 0 0 16z"/>
            <path d="M10.97 4.97a.235.235 0 0 0-.02.022L7.477 9.417 5.384 7.323a.75.75 0 0 0-1.06 1.06L6.97 11.03a.75.75 0 0 0 1.079-.02l3.992-4.99a.75.75 0 0 0-1.071-1.05z"/>
          </svg>
          <p>No accessibility issues found!</p>
        </div>
      `;
      return;
    }

    container.innerHTML = issues
      .map(
        (issue) => `
      <div class="issue-item severity-${issue.severity}" data-issue-id="${issue.id}">
        <div class="issue-header">
          <span class="severity-badge severity-${issue.severity}">${issue.severity}</span>
          <span class="category-badge">${issue.category}</span>
        </div>
        <div class="issue-title">${this.escapeHtml(issue.title)}</div>
        <div class="issue-description">${this.escapeHtml(issue.description)}</div>
        <div class="issue-element">
          <code>${this.escapeHtml(issue.snippet)}</code>
        </div>
        <div class="issue-footer">
          <span class="wcag-criteria">WCAG ${issue.wcagLevel}: ${issue.wcagCriteria.join(', ')}</span>
          <button class="btn-link inspect-button" data-issue-id="${issue.id}">Inspect</button>
        </div>
      </div>
    `
      )
      .join('');

    // Add click handlers
    container.querySelectorAll('.inspect-button').forEach((button) => {
      button.addEventListener('click', (e) => {
        const issueId = (e.currentTarget as HTMLElement).dataset.issueId;
        if (issueId) this.inspectIssue(issueId);
      });
    });
  }

  private filterIssues(): void {
    if (!this.scanResult) return;

    const severityFilter = (
      document.getElementById('severity-filter') as HTMLSelectElement
    )?.value;
    const categoryFilter = (
      document.getElementById('category-filter') as HTMLSelectElement
    )?.value;

    let filtered = this.scanResult.issues;

    if (severityFilter !== 'all') {
      filtered = filtered.filter((issue) => issue.severity === severityFilter);
    }

    if (categoryFilter !== 'all') {
      filtered = filtered.filter((issue) => issue.category === categoryFilter);
    }

    this.renderIssues(filtered);
  }

  // --------------------------------------------------------------------------
  // Element Inspector
  // --------------------------------------------------------------------------

  private inspectIssue(issueId: string): void {
    const issue = this.scanResult?.issues.find((i) => i.id === issueId);
    if (!issue) return;

    this.state.selectedIssue = issue;
    this.renderInspector(issue);

    // Highlight element in page
    chrome.tabs.sendMessage(this.tabId, {
      type: 'HIGHLIGHT_ELEMENT',
      payload: { selector: issue.selector },
    });
  }

  private renderInspector(issue: AccessibilityIssue): void {
    const container = document.getElementById('inspector-content');
    if (!container) return;

    container.innerHTML = `
      <div class="inspector-details">
        <div class="detail-section">
          <h4>Issue Details</h4>
          <dl class="detail-list">
            <dt>Severity:</dt>
            <dd><span class="severity-badge severity-${issue.severity}">${issue.severity}</span></dd>
            <dt>Category:</dt>
            <dd>${issue.category}</dd>
            <dt>WCAG Criteria:</dt>
            <dd>${issue.wcagCriteria.join(', ')} (Level ${issue.wcagLevel})</dd>
          </dl>
        </div>

        <div class="detail-section">
          <h4>Element</h4>
          <dl class="detail-list">
            <dt>Tag:</dt>
            <dd><code>${issue.element.tagName}</code></dd>
            ${issue.element.id ? `<dt>ID:</dt><dd><code>${issue.element.id}</code></dd>` : ''}
            ${issue.element.className ? `<dt>Class:</dt><dd><code>${issue.element.className}</code></dd>` : ''}
            ${issue.element.role ? `<dt>Role:</dt><dd><code>${issue.element.role}</code></dd>` : ''}
            <dt>Selector:</dt>
            <dd><code class="code-block">${this.escapeHtml(issue.selector)}</code></dd>
          </dl>
        </div>

        <div class="detail-section">
          <h4>Suggestion</h4>
          <p class="suggestion-text">${this.escapeHtml(issue.suggestion)}</p>
        </div>

        <div class="detail-section">
          <h4>Impact</h4>
          <p class="impact-text">${this.escapeHtml(issue.impact)}</p>
        </div>

        <div class="detail-section">
          <a href="${issue.helpUrl}" target="_blank" class="help-link">
            Learn more about this issue →
          </a>
        </div>
      </div>
    `;
  }

  // --------------------------------------------------------------------------
  // Accessibility Tree
  // --------------------------------------------------------------------------

  private buildAccessibilityTree(): void {
    // This would build a full accessibility tree from the DOM
    // Simplified for now
    const container = document.getElementById('tree-view');
    if (!container) return;

    container.innerHTML = '<div class="tree-node">Accessibility tree construction in progress...</div>';
  }

  // --------------------------------------------------------------------------
  // Utilities
  // --------------------------------------------------------------------------

  private switchTab(tabName: string): void {
    // Update buttons
    document.querySelectorAll('.tab-button').forEach((button) => {
      button.classList.toggle('active', button.getAttribute('data-tab') === tabName);
    });

    // Update content
    document.querySelectorAll('.tab-content').forEach((content) => {
      content.classList.toggle('active', content.id === `${tabName}-tab`);
    });
  }

  private toggleHighlights(): void {
    this.state.highlightEnabled = !this.state.highlightEnabled;

    chrome.tabs.sendMessage(this.tabId, {
      type: 'TOGGLE_HIGHLIGHTS',
    });
  }

  private async handleExport(): Promise<void> {
    if (!this.scanResult) return;

    const blob = new Blob([JSON.stringify(this.scanResult, null, 2)], {
      type: 'application/json',
    });

    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `caddy-scan-${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
  }

  private updateStatus(text: string, loading: boolean): void {
    const status = document.getElementById('status-text');
    if (status) {
      status.textContent = text;
      status.classList.toggle('loading', loading);
    }
  }

  private updateScanDuration(duration: number): void {
    const element = document.getElementById('scan-duration');
    if (element) {
      element.textContent = `Scan took ${duration}ms`;
    }
  }

  private addConsoleMessage(message: any): void {
    const console = document.getElementById('console-output');
    if (!console) return;

    const entry = document.createElement('div');
    entry.className = `console-entry console-${message.level}`;
    entry.textContent = `[${new Date().toLocaleTimeString()}] ${message.text}`;
    console.appendChild(entry);
    console.scrollTop = console.scrollHeight;
  }

  private async loadInitialData(): Promise<void> {
    try {
      const response = await chrome.runtime.sendMessage({
        type: 'GET_RESULTS',
        timestamp: Date.now(),
      });

      if (response.length > 0) {
        this.handleScanComplete(response[0]);
      }
    } catch (error) {
      console.error('[CADDY DevTools] Error loading initial data:', error);
    }
  }

  private handleElementSelected(element: any): void {
    // Handle element selection from page
    console.log('[CADDY DevTools] Element selected:', element);
  }

  private escapeHtml(text: string): string {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }
}

// ============================================================================
// Initialization
// ============================================================================

// Get tab ID from URL params
const params = new URLSearchParams(window.location.search);
const tabId = parseInt(params.get('tabId') || '0');

if (tabId) {
  new DevToolsPanel(tabId);
} else {
  console.error('[CADDY DevTools] No tab ID provided');
}

export { DevToolsPanel };
