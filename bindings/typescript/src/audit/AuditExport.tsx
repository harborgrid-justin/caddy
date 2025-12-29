/**
 * Audit Export Component
 * Export audit logs for compliance and archival purposes
 */

import React, { useState } from 'react';
import { AuditFilters } from './AuditFilters';
import type { AuditFilter, ExportOptions } from './types';

interface AuditExportProps {
  organizationId?: string;
}

export const AuditExport: React.FC<AuditExportProps> = ({ organizationId }) => {
  const [filters, setFilters] = useState<AuditFilter>({});
  const [exportFormat, setExportFormat] = useState<'csv' | 'json' | 'pdf' | 'xlsx'>('csv');
  const [includeMetadata, setIncludeMetadata] = useState(true);
  const [includeHashChain, setIncludeHashChain] = useState(true);
  const [encrypt, setEncrypt] = useState(false);
  const [password, setPassword] = useState('');
  const [digitalSignature, setDigitalSignature] = useState(true);
  const [exporting, setExporting] = useState(false);
  const [exportProgress, setExportProgress] = useState(0);
  const [exportHistory, setExportHistory] = useState<ExportHistoryItem[]>([]);

  const handleExport = async () => {
    setExporting(true);
    setExportProgress(0);

    try {
      const options: ExportOptions = {
        format: exportFormat,
        filters,
        include_metadata: includeMetadata,
        include_hash_chain: includeHashChain,
        encrypt,
        password: encrypt ? password : undefined,
        digital_signature: digitalSignature,
      };

      const response = await fetch('/api/audit/export', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          ...options,
          organization_id: organizationId,
        }),
      });

      if (!response.ok) {
        throw new Error('Export failed');
      }

      // Simulate progress
      const progressInterval = setInterval(() => {
        setExportProgress((prev) => Math.min(prev + 10, 90));
      }, 200);

      const blob = await response.blob();
      clearInterval(progressInterval);
      setExportProgress(100);

      // Download file
      const url = URL.createObjectURL(blob);
      const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
      const filename = `audit-export-${timestamp}.${exportFormat}${
        encrypt ? '.encrypted' : ''
      }`;

      const link = document.createElement('a');
      link.href = url;
      link.download = filename;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);

      // Add to export history
      const historyItem: ExportHistoryItem = {
        id: crypto.randomUUID(),
        timestamp: new Date().toISOString(),
        format: exportFormat,
        filename,
        event_count: 0, // Would come from response
        encrypted: encrypt,
        signed: digitalSignature,
      };
      setExportHistory((prev) => [historyItem, ...prev].slice(0, 10));

      // Reset after success
      setTimeout(() => {
        setExportProgress(0);
        setExporting(false);
      }, 1000);
    } catch (error) {
      console.error('Export failed:', error);
      setExporting(false);
      setExportProgress(0);
      alert('Export failed. Please try again.');
    }
  };

  const isValid = () => {
    if (encrypt && !password) return false;
    return true;
  };

  return (
    <div className="audit-export">
      {/* Header */}
      <div className="export-header">
        <div>
          <h2>Export Audit Logs</h2>
          <p className="subtitle">
            Export audit logs for compliance, archival, or analysis purposes
          </p>
        </div>
      </div>

      <div className="export-content">
        {/* Export Configuration */}
        <div className="export-config">
          <section className="config-section">
            <h3>Export Format</h3>
            <div className="format-options">
              <FormatOption
                format="csv"
                label="CSV"
                description="Comma-separated values for spreadsheet analysis"
                icon="📊"
                selected={exportFormat === 'csv'}
                onClick={() => setExportFormat('csv')}
              />
              <FormatOption
                format="json"
                label="JSON"
                description="Structured JSON for programmatic processing"
                icon="{ }"
                selected={exportFormat === 'json'}
                onClick={() => setExportFormat('json')}
              />
              <FormatOption
                format="pdf"
                label="PDF"
                description="Formatted PDF report for documentation"
                icon="📄"
                selected={exportFormat === 'pdf'}
                onClick={() => setExportFormat('pdf')}
              />
              <FormatOption
                format="xlsx"
                label="Excel"
                description="Excel workbook with multiple sheets"
                icon="📈"
                selected={exportFormat === 'xlsx'}
                onClick={() => setExportFormat('xlsx')}
              />
            </div>
          </section>

          <section className="config-section">
            <h3>Export Options</h3>
            <div className="options-list">
              <label className="option-item">
                <input
                  type="checkbox"
                  checked={includeMetadata}
                  onChange={(e) => setIncludeMetadata(e.target.checked)}
                />
                <div className="option-content">
                  <strong>Include Metadata</strong>
                  <span className="option-description">
                    Include custom metadata and additional context
                  </span>
                </div>
              </label>

              <label className="option-item">
                <input
                  type="checkbox"
                  checked={includeHashChain}
                  onChange={(e) => setIncludeHashChain(e.target.checked)}
                />
                <div className="option-content">
                  <strong>Include Hash Chain</strong>
                  <span className="option-description">
                    Include cryptographic hashes for integrity verification
                  </span>
                </div>
              </label>

              <label className="option-item">
                <input
                  type="checkbox"
                  checked={digitalSignature}
                  onChange={(e) => setDigitalSignature(e.target.checked)}
                />
                <div className="option-content">
                  <strong>Digital Signature</strong>
                  <span className="option-description">
                    Sign export with organization's private key
                  </span>
                </div>
              </label>

              <label className="option-item">
                <input
                  type="checkbox"
                  checked={encrypt}
                  onChange={(e) => {
                    setEncrypt(e.target.checked);
                    if (!e.target.checked) setPassword('');
                  }}
                />
                <div className="option-content">
                  <strong>Encrypt Export</strong>
                  <span className="option-description">
                    Password-protect the exported file with AES-256 encryption
                  </span>
                </div>
              </label>

              {encrypt && (
                <div className="password-input">
                  <input
                    type="password"
                    placeholder="Enter encryption password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    className="input"
                  />
                  <small className="input-hint">
                    Minimum 12 characters. Store this password securely.
                  </small>
                </div>
              )}
            </div>
          </section>

          <section className="config-section">
            <h3>Filters</h3>
            <AuditFilters
              filters={filters}
              onChange={setFilters}
              onReset={() => setFilters({})}
            />
          </section>

          {/* Export Button */}
          <div className="export-actions">
            <button
              className="btn btn-primary btn-lg"
              onClick={handleExport}
              disabled={!isValid() || exporting}
            >
              {exporting ? 'Exporting...' : 'Export Audit Logs'}
            </button>
          </div>

          {/* Export Progress */}
          {exporting && (
            <div className="export-progress">
              <div className="progress-bar">
                <div
                  className="progress-fill"
                  style={{ width: `${exportProgress}%` }}
                />
              </div>
              <span className="progress-text">{exportProgress}%</span>
            </div>
          )}
        </div>

        {/* Export History */}
        <div className="export-history">
          <h3>Recent Exports</h3>
          {exportHistory.length === 0 ? (
            <div className="empty-state">
              <p>No recent exports</p>
            </div>
          ) : (
            <div className="history-list">
              {exportHistory.map((item) => (
                <div key={item.id} className="history-item">
                  <div className="history-icon">
                    {item.format === 'csv' && '📊'}
                    {item.format === 'json' && '{ }'}
                    {item.format === 'pdf' && '📄'}
                    {item.format === 'xlsx' && '📈'}
                  </div>
                  <div className="history-content">
                    <div className="history-filename">{item.filename}</div>
                    <div className="history-meta">
                      {formatTimestamp(item.timestamp)}
                      {item.encrypted && (
                        <>
                          {' • '}
                          <span className="badge badge-warning">Encrypted</span>
                        </>
                      )}
                      {item.signed && (
                        <>
                          {' • '}
                          <span className="badge badge-success">Signed</span>
                        </>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Compliance Notice */}
        <div className="compliance-notice">
          <h4>Compliance & Security Notice</h4>
          <ul>
            <li>
              Exported audit logs contain sensitive information. Handle with care
              and follow your organization's data protection policies.
            </li>
            <li>
              Encrypted exports use AES-256 encryption. Store passwords securely
              using a password manager.
            </li>
            <li>
              Digital signatures ensure authenticity and can be verified using your
              organization's public key.
            </li>
            <li>
              Hash chains allow verification of audit log integrity. Tampering with
              any event will break the chain.
            </li>
            <li>
              Retain exports according to your compliance requirements (SOC2, GDPR,
              HIPAA, etc.).
            </li>
          </ul>
        </div>
      </div>
    </div>
  );
};

// Format Option Component
function FormatOption({
  format,
  label,
  description,
  icon,
  selected,
  onClick,
}: {
  format: string;
  label: string;
  description: string;
  icon: string;
  selected: boolean;
  onClick: () => void;
}) {
  return (
    <button
      className={`format-option ${selected ? 'selected' : ''}`}
      onClick={onClick}
    >
      <div className="format-icon">{icon}</div>
      <div className="format-content">
        <div className="format-label">{label}</div>
        <div className="format-description">{description}</div>
      </div>
      {selected && <div className="format-check">✓</div>}
    </button>
  );
}

// Types
interface ExportHistoryItem {
  id: string;
  timestamp: string;
  format: 'csv' | 'json' | 'pdf' | 'xlsx';
  filename: string;
  event_count: number;
  encrypted: boolean;
  signed: boolean;
}

// Utility Functions
function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  return new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date);
}
