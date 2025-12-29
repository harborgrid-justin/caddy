/**
 * Audit Detail Component
 * Detailed view of a single audit event with full metadata and integrity verification
 */

import React, { useState, useEffect } from 'react';
import type { AuditEvent, AnomalyDetectionResult } from './types';

interface AuditDetailProps {
  eventId: string;
  event?: AuditEvent;
  onClose?: () => void;
}

export const AuditDetail: React.FC<AuditDetailProps> = ({
  eventId,
  event: providedEvent,
  onClose,
}) => {
  const [event, setEvent] = useState<AuditEvent | null>(providedEvent || null);
  const [loading, setLoading] = useState(!providedEvent);
  const [verifying, setVerifying] = useState(false);
  const [integrityStatus, setIntegrityStatus] = useState<'verified' | 'failed' | null>(null);
  const [anomalyDetails, setAnomalyDetails] = useState<AnomalyDetectionResult | null>(null);
  const [activeTab, setActiveTab] = useState<'overview' | 'metadata' | 'changes' | 'security'>('overview');

  useEffect(() => {
    if (!providedEvent) {
      loadEventDetails();
    }
  }, [eventId, providedEvent]);

  const loadEventDetails = async () => {
    setLoading(true);
    try {
      const response = await fetch(`/api/audit/events/${eventId}`);
      const data = await response.json();
      setEvent(data);

      // Load anomaly details if detected
      if (data.anomaly_detected) {
        const anomalyResponse = await fetch(`/api/audit/events/${eventId}/anomaly`);
        const anomalyData = await anomalyResponse.json();
        setAnomalyDetails(anomalyData);
      }
    } catch (error) {
      console.error('Failed to load event details:', error);
    } finally {
      setLoading(false);
    }
  };

  const verifyIntegrity = async () => {
    if (!event) return;

    setVerifying(true);
    try {
      const response = await fetch(`/api/audit/events/${event.id}/verify`, {
        method: 'POST',
      });
      const data = await response.json();
      setIntegrityStatus(data.verified ? 'verified' : 'failed');
    } catch (error) {
      console.error('Failed to verify integrity:', error);
      setIntegrityStatus('failed');
    } finally {
      setVerifying(false);
    }
  };

  if (loading) {
    return (
      <div className="audit-detail loading">
        <div className="loading-spinner" />
        <p>Loading event details...</p>
      </div>
    );
  }

  if (!event) {
    return (
      <div className="audit-detail error">
        <h2>Event Not Found</h2>
        <p>The requested audit event could not be found.</p>
        {onClose && (
          <button onClick={onClose} className="btn btn-secondary">
            Close
          </button>
        )}
      </div>
    );
  }

  return (
    <div className="audit-detail">
      {/* Header */}
      <div className="detail-header">
        <div className="header-content">
          <h2>Audit Event Details</h2>
          <div className="event-badges">
            <SeverityBadge severity={event.severity} />
            <StatusBadge status={event.status} />
            {event.anomaly_detected && (
              <span className="badge badge-warning">Anomaly Detected</span>
            )}
            {event.risk_score !== undefined && event.risk_score >= 70 && (
              <span className="badge badge-danger">High Risk</span>
            )}
          </div>
        </div>
        <div className="header-actions">
          <button
            className="btn btn-secondary"
            onClick={verifyIntegrity}
            disabled={verifying}
          >
            {verifying ? 'Verifying...' : 'Verify Integrity'}
          </button>
          {onClose && (
            <button onClick={onClose} className="btn btn-ghost">
              Close
            </button>
          )}
        </div>
      </div>

      {/* Integrity Status */}
      {integrityStatus && (
        <div className={`integrity-status status-${integrityStatus}`}>
          {integrityStatus === 'verified' ? (
            <>
              <span className="status-icon">✓</span>
              <span>Event integrity verified - hash chain intact</span>
            </>
          ) : (
            <>
              <span className="status-icon">✗</span>
              <span>Integrity verification failed - possible tampering detected</span>
            </>
          )}
        </div>
      )}

      {/* Tabs */}
      <div className="detail-tabs">
        <button
          className={`tab ${activeTab === 'overview' ? 'active' : ''}`}
          onClick={() => setActiveTab('overview')}
        >
          Overview
        </button>
        <button
          className={`tab ${activeTab === 'metadata' ? 'active' : ''}`}
          onClick={() => setActiveTab('metadata')}
        >
          Metadata
        </button>
        {event.changes && event.changes.length > 0 && (
          <button
            className={`tab ${activeTab === 'changes' ? 'active' : ''}`}
            onClick={() => setActiveTab('changes')}
          >
            Changes ({event.changes.length})
          </button>
        )}
        <button
          className={`tab ${activeTab === 'security' ? 'active' : ''}`}
          onClick={() => setActiveTab('security')}
        >
          Security
        </button>
      </div>

      {/* Tab Content */}
      <div className="detail-content">
        {activeTab === 'overview' && (
          <OverviewTab event={event} anomalyDetails={anomalyDetails} />
        )}
        {activeTab === 'metadata' && <MetadataTab event={event} />}
        {activeTab === 'changes' && event.changes && (
          <ChangesTab changes={event.changes} />
        )}
        {activeTab === 'security' && (
          <SecurityTab event={event} anomalyDetails={anomalyDetails} />
        )}
      </div>
    </div>
  );
};

// Overview Tab
function OverviewTab({
  event,
  anomalyDetails,
}: {
  event: AuditEvent;
  anomalyDetails: AnomalyDetectionResult | null;
}) {
  return (
    <div className="overview-tab">
      {/* Primary Information */}
      <section className="detail-section">
        <h3>Event Information</h3>
        <div className="detail-grid">
          <DetailField label="Event ID" value={event.id} copyable />
          <DetailField label="Timestamp" value={formatTimestamp(event.timestamp)} />
          <DetailField label="Event Type" value={formatEventType(event.event_type)} />
          <DetailField label="Action" value={event.action} />
          <DetailField label="Description" value={event.description} fullWidth />
        </div>
      </section>

      {/* User Information */}
      <section className="detail-section">
        <h3>Actor Information</h3>
        <div className="detail-grid">
          <DetailField label="User ID" value={event.user_id || 'System'} copyable />
          <DetailField label="Email" value={event.user_email || 'N/A'} />
          <DetailField label="Name" value={event.user_name || 'N/A'} />
          <DetailField label="IP Address" value={event.user_ip_address} copyable />
          <DetailField label="Session ID" value={event.session_id} copyable />
          <DetailField
            label="User Agent"
            value={event.user_agent}
            fullWidth
            truncate
          />
        </div>
        {event.location && (
          <div className="location-info">
            <strong>Location:</strong>
            {event.location.city && ` ${event.location.city},`}
            {event.location.region && ` ${event.location.region},`}
            {event.location.country && ` ${event.location.country}`}
          </div>
        )}
      </section>

      {/* Resource Information */}
      {event.resource_type && (
        <section className="detail-section">
          <h3>Resource Information</h3>
          <div className="detail-grid">
            <DetailField label="Resource Type" value={event.resource_type} />
            <DetailField label="Resource ID" value={event.resource_id || 'N/A'} copyable />
            <DetailField label="Resource Name" value={event.resource_name || 'N/A'} fullWidth />
          </div>
        </section>
      )}

      {/* Anomaly Details */}
      {anomalyDetails && (
        <section className="detail-section anomaly-section">
          <h3>Anomaly Detection</h3>
          <div className="anomaly-info">
            <div className="anomaly-header">
              <span className="anomaly-type">{anomalyDetails.anomaly_type}</span>
              <span className="confidence-score">
                Confidence: {(anomalyDetails.confidence_score * 100).toFixed(1)}%
              </span>
            </div>
            <div className="anomaly-reasons">
              <strong>Reasons:</strong>
              <ul>
                {anomalyDetails.reasons.map((reason, index) => (
                  <li key={index}>{reason}</li>
                ))}
              </ul>
            </div>
            {anomalyDetails.baseline_metrics && (
              <div className="metrics-comparison">
                <div className="metrics-column">
                  <strong>Baseline Metrics:</strong>
                  <pre>{JSON.stringify(anomalyDetails.baseline_metrics, null, 2)}</pre>
                </div>
                <div className="metrics-column">
                  <strong>Current Metrics:</strong>
                  <pre>{JSON.stringify(anomalyDetails.current_metrics, null, 2)}</pre>
                </div>
              </div>
            )}
          </div>
        </section>
      )}
    </div>
  );
}

// Metadata Tab
function MetadataTab({ event }: { event: AuditEvent }) {
  return (
    <div className="metadata-tab">
      <section className="detail-section">
        <h3>Custom Metadata</h3>
        <div className="metadata-content">
          <pre className="json-viewer">
            {JSON.stringify(event.metadata, null, 2)}
          </pre>
        </div>
      </section>

      <section className="detail-section">
        <h3>Compliance & Classification</h3>
        <div className="detail-grid">
          <DetailField
            label="Data Classification"
            value={event.data_classification || 'N/A'}
          />
          <DetailField
            label="Retention Policy"
            value={event.retention_policy}
          />
          <DetailField
            label="Compliance Frameworks"
            value={event.compliance_frameworks?.join(', ') || 'N/A'}
            fullWidth
          />
        </div>
      </section>

      <section className="detail-section">
        <h3>Organization & Tenant</h3>
        <div className="detail-grid">
          <DetailField
            label="Organization ID"
            value={event.organization_id || 'N/A'}
            copyable
          />
          <DetailField
            label="Tenant ID"
            value={event.tenant_id || 'N/A'}
            copyable
          />
        </div>
      </section>
    </div>
  );
}

// Changes Tab
function ChangesTab({
  changes,
}: {
  changes: { field: string; old_value: unknown; new_value: unknown }[];
}) {
  return (
    <div className="changes-tab">
      <section className="detail-section">
        <h3>Data Changes</h3>
        <table className="changes-table">
          <thead>
            <tr>
              <th>Field</th>
              <th>Old Value</th>
              <th>New Value</th>
            </tr>
          </thead>
          <tbody>
            {changes.map((change, index) => (
              <tr key={index}>
                <td className="field-name">{change.field}</td>
                <td className="old-value">
                  <code>{formatValue(change.old_value)}</code>
                </td>
                <td className="new-value">
                  <code>{formatValue(change.new_value)}</code>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>
    </div>
  );
}

// Security Tab
function SecurityTab({
  event,
  anomalyDetails,
}: {
  event: AuditEvent;
  anomalyDetails: AnomalyDetectionResult | null;
}) {
  return (
    <div className="security-tab">
      <section className="detail-section">
        <h3>Security Assessment</h3>
        <div className="detail-grid">
          <DetailField
            label="Risk Score"
            value={event.risk_score?.toFixed(0) || 'N/A'}
          />
          <DetailField
            label="Anomaly Detected"
            value={event.anomaly_detected ? 'Yes' : 'No'}
          />
          {anomalyDetails && (
            <DetailField
              label="Anomaly Type"
              value={anomalyDetails.anomaly_type || 'N/A'}
              fullWidth
            />
          )}
        </div>
      </section>

      <section className="detail-section">
        <h3>Cryptographic Integrity</h3>
        <div className="detail-grid">
          <DetailField label="Event Hash" value={event.hash} copyable fullWidth />
          <DetailField
            label="Digital Signature"
            value={event.signature}
            copyable
            fullWidth
          />
          <DetailField
            label="Previous Event Hash"
            value={event.previous_event_hash || 'N/A (First Event)'}
            copyable
            fullWidth
          />
        </div>
        <div className="integrity-info">
          <p>
            This audit event is part of a tamper-proof hash chain. Each event
            contains the hash of the previous event, ensuring integrity of the
            entire audit trail.
          </p>
        </div>
      </section>

      <section className="detail-section">
        <h3>Session Details</h3>
        <div className="detail-grid">
          <DetailField label="Session ID" value={event.session_id} copyable />
          <DetailField label="IP Address" value={event.user_ip_address} copyable />
          <DetailField label="User Agent" value={event.user_agent} fullWidth truncate />
        </div>
      </section>
    </div>
  );
}

// Detail Field Component
function DetailField({
  label,
  value,
  copyable = false,
  fullWidth = false,
  truncate = false,
}: {
  label: string;
  value: string;
  copyable?: boolean;
  fullWidth?: boolean;
  truncate?: boolean;
}) {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(value);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className={`detail-field ${fullWidth ? 'full-width' : ''}`}>
      <label>{label}</label>
      <div className="field-value">
        <span className={truncate ? 'truncate' : ''}>{value}</span>
        {copyable && (
          <button
            className="copy-button"
            onClick={handleCopy}
            title={copied ? 'Copied!' : 'Copy to clipboard'}
          >
            {copied ? '✓' : '📋'}
          </button>
        )}
      </div>
    </div>
  );
}

// Badge Components
function SeverityBadge({ severity }: { severity: string }) {
  const colors: Record<string, string> = {
    low: 'green',
    medium: 'yellow',
    high: 'orange',
    critical: 'red',
  };

  return (
    <span className={`badge badge-${colors[severity] || 'gray'}`}>
      {severity.toUpperCase()}
    </span>
  );
}

function StatusBadge({ status }: { status: string }) {
  const colors: Record<string, string> = {
    success: 'green',
    failure: 'red',
    pending: 'yellow',
    blocked: 'gray',
  };

  return (
    <span className={`badge badge-${colors[status] || 'gray'}`}>
      {status.toUpperCase()}
    </span>
  );
}

// Utility Functions
function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  return new Intl.DateTimeFormat('en-US', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    timeZoneName: 'short',
  }).format(date);
}

function formatEventType(eventType: string): string {
  return eventType
    .split('.')
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(' ');
}

function formatValue(value: unknown): string {
  if (value === null || value === undefined) return 'null';
  if (typeof value === 'object') return JSON.stringify(value);
  return String(value);
}
