/**
 * CADDY v0.3.0 - Extension Popup
 * Quick access interface for accessibility scanning
 */

import React, { useState, useEffect } from 'react';
import type { ScanResult, UserSettings, SeverityLevel } from '../shared/types';

// ============================================================================
// Popup Component
// ============================================================================

export const Popup: React.FC = () => {
  const [scanning, setScanning] = useState(false);
  const [result, setResult] = useState<ScanResult | null>(null);
  const [settings, setSettings] = useState<UserSettings | null>(null);
  const [authenticated, setAuthenticated] = useState(false);
  const [currentTab, setCurrentTab] = useState<chrome.tabs.Tab | null>(null);

  // --------------------------------------------------------------------------
  // Effects
  // --------------------------------------------------------------------------

  useEffect(() => {
    loadInitialData();
  }, []);

  const loadInitialData = async () => {
    try {
      // Get current tab
      const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
      setCurrentTab(tab);

      // Load settings
      const settingsResponse = await chrome.runtime.sendMessage({
        type: 'GET_SETTINGS',
        timestamp: Date.now(),
      });
      setSettings(settingsResponse);

      // Check authentication
      const authData = await chrome.storage.local.get(['auth']);
      setAuthenticated(!!authData.auth?.token);

      // Load cached result for current URL
      if (tab.url) {
        const resultsResponse = await chrome.runtime.sendMessage({
          type: 'GET_RESULTS',
          payload: { url: tab.url },
          timestamp: Date.now(),
        });

        if (resultsResponse.length > 0) {
          setResult(resultsResponse[0]);
        }
      }
    } catch (error) {
      console.error('[CADDY Popup] Error loading data:', error);
    }
  };

  // --------------------------------------------------------------------------
  // Handlers
  // --------------------------------------------------------------------------

  const handleScan = async () => {
    if (!currentTab?.id) return;

    setScanning(true);
    setResult(null);

    try {
      const response = await chrome.runtime.sendMessage({
        type: 'SCAN_PAGE',
        payload: {
          url: currentTab.url,
          config: settings?.scanning,
        },
        tabId: currentTab.id,
        timestamp: Date.now(),
      });

      if (response.success) {
        setResult(response.result);
      } else {
        console.error('[CADDY Popup] Scan failed:', response.error);
      }
    } catch (error) {
      console.error('[CADDY Popup] Scan error:', error);
    } finally {
      setScanning(false);
    }
  };

  const handleToggleHighlights = async () => {
    if (!currentTab?.id) return;

    await chrome.runtime.sendMessage({
      type: 'TOGGLE_HIGHLIGHTS',
      tabId: currentTab.id,
      timestamp: Date.now(),
    });
  };

  const handleOpenDevTools = () => {
    chrome.tabs.create({
      url: `devtools.html?tabId=${currentTab?.id}`,
    });
  };

  const handleOpenOptions = () => {
    chrome.runtime.openOptionsPage();
  };

  // --------------------------------------------------------------------------
  // Render
  // --------------------------------------------------------------------------

  return (
    <div className="popup-container">
      <Header authenticated={authenticated} />

      <div className="popup-content">
        {/* Scan Button */}
        <ScanButton
          scanning={scanning}
          onScan={handleScan}
          disabled={!currentTab?.url || currentTab.url.startsWith('chrome://')}
        />

        {/* Results Summary */}
        {result && <ResultsSummary result={result} />}

        {/* Quick Actions */}
        <QuickActions
          onToggleHighlights={handleToggleHighlights}
          onOpenDevTools={handleOpenDevTools}
          hasResults={!!result}
        />

        {/* Settings Link */}
        <SettingsLink onOpenOptions={handleOpenOptions} />
      </div>

      <Footer />
    </div>
  );
};

// ============================================================================
// Sub-Components
// ============================================================================

const Header: React.FC<{ authenticated: boolean }> = ({ authenticated }) => (
  <div className="popup-header">
    <div className="header-content">
      <img src="/icons/icon-48.png" alt="CADDY" className="logo" />
      <div>
        <h1 className="title">CADDY</h1>
        <p className="subtitle">Accessibility Scanner</p>
      </div>
    </div>
    {authenticated && (
      <div className="auth-badge">
        <span className="badge badge-success">Pro</span>
      </div>
    )}
  </div>
);

const ScanButton: React.FC<{
  scanning: boolean;
  onScan: () => void;
  disabled: boolean;
}> = ({ scanning, onScan, disabled }) => (
  <button
    className={`scan-button ${scanning ? 'scanning' : ''}`}
    onClick={onScan}
    disabled={disabled || scanning}
  >
    {scanning ? (
      <>
        <Spinner />
        <span>Scanning...</span>
      </>
    ) : (
      <>
        <ScanIcon />
        <span>Scan Page</span>
      </>
    )}
  </button>
);

const ResultsSummary: React.FC<{ result: ScanResult }> = ({ result }) => {
  const { summary } = result;

  return (
    <div className="results-summary">
      <div className="summary-header">
        <h3>Scan Results</h3>
        <span className="timestamp">{formatTimestamp(result.timestamp)}</span>
      </div>

      <div className="score-card">
        <div className="score-circle">
          <svg viewBox="0 0 100 100" className="score-ring">
            <circle
              cx="50"
              cy="50"
              r="45"
              fill="none"
              stroke="#E5E7EB"
              strokeWidth="8"
            />
            <circle
              cx="50"
              cy="50"
              r="45"
              fill="none"
              stroke={getScoreColor(summary.complianceScore)}
              strokeWidth="8"
              strokeDasharray={`${summary.complianceScore * 2.827} 283`}
              strokeLinecap="round"
              transform="rotate(-90 50 50)"
            />
          </svg>
          <div className="score-value">{summary.complianceScore}</div>
        </div>
        <div className="score-label">Compliance Score</div>
      </div>

      <div className="issues-grid">
        <IssueCard
          severity="critical"
          count={summary.critical}
          label="Critical"
        />
        <IssueCard
          severity="serious"
          count={summary.serious}
          label="Serious"
        />
        <IssueCard
          severity="moderate"
          count={summary.moderate}
          label="Moderate"
        />
        <IssueCard severity="minor" count={summary.minor} label="Minor" />
      </div>

      <div className="wcag-info">
        <span className="wcag-badge">WCAG {summary.wcagLevel}</span>
        <span className="total-issues">
          {summary.total} {summary.total === 1 ? 'issue' : 'issues'} found
        </span>
      </div>
    </div>
  );
};

const IssueCard: React.FC<{
  severity: SeverityLevel;
  count: number;
  label: string;
}> = ({ severity, count, label }) => (
  <div className={`issue-card severity-${severity}`}>
    <div className="issue-count">{count}</div>
    <div className="issue-label">{label}</div>
  </div>
);

const QuickActions: React.FC<{
  onToggleHighlights: () => void;
  onOpenDevTools: () => void;
  hasResults: boolean;
}> = ({ onToggleHighlights, onOpenDevTools, hasResults }) => (
  <div className="quick-actions">
    <button
      className="action-button"
      onClick={onToggleHighlights}
      disabled={!hasResults}
      title="Toggle issue highlights (Ctrl+Shift+H)"
    >
      <HighlightIcon />
      <span>Toggle Highlights</span>
    </button>
    <button
      className="action-button"
      onClick={onOpenDevTools}
      title="Open DevTools panel"
    >
      <DevToolsIcon />
      <span>Open DevTools</span>
    </button>
  </div>
);

const SettingsLink: React.FC<{ onOpenOptions: () => void }> = ({
  onOpenOptions,
}) => (
  <div className="settings-link">
    <button className="link-button" onClick={onOpenOptions}>
      <SettingsIcon />
      <span>Settings</span>
    </button>
  </div>
);

const Footer: React.FC = () => (
  <div className="popup-footer">
    <span className="version">v0.3.0</span>
    <a
      href="https://caddy.dev/docs"
      target="_blank"
      rel="noopener noreferrer"
      className="footer-link"
    >
      Documentation
    </a>
  </div>
);

// ============================================================================
// Icons
// ============================================================================

const ScanIcon = () => (
  <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor">
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4"
    />
  </svg>
);

const HighlightIcon = () => (
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor">
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
    />
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
    />
  </svg>
);

const DevToolsIcon = () => (
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor">
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"
    />
  </svg>
);

const SettingsIcon = () => (
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor">
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
    />
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
    />
  </svg>
);

const Spinner = () => (
  <svg
    className="spinner"
    width="20"
    height="20"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
  >
    <circle cx="12" cy="12" r="10" strokeWidth="3" opacity="0.25" />
    <path
      d="M12 2a10 10 0 0110 10"
      strokeWidth="3"
      strokeLinecap="round"
    />
  </svg>
);

// ============================================================================
// Utility Functions
// ============================================================================

function formatTimestamp(timestamp: number): string {
  const date = new Date(timestamp);
  const now = new Date();
  const diff = now.getTime() - date.getTime();

  if (diff < 60000) return 'Just now';
  if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;

  return date.toLocaleDateString();
}

function getScoreColor(score: number): string {
  if (score >= 90) return '#10B981';
  if (score >= 70) return '#3B82F6';
  if (score >= 50) return '#F59E0B';
  return '#DC2626';
}

export default Popup;
