export { AuditDashboard } from './AuditDashboard';
export { AuditLog } from './AuditLog';
export { AuditDetail } from './AuditDetail';
export { AuditFilters } from './AuditFilters';
export { AuditExport } from './AuditExport';
export { AuditRetention } from './AuditRetention';
export { ComplianceChecklist } from './ComplianceChecklist';
export { ComplianceReports } from './ComplianceReports';
export { DataLineage } from './DataLineage';
export { AuditAlerts } from './AuditAlerts';
export { AuditAnalytics } from './AuditAnalytics';
export const AUDIT_VERSION = '0.4.0';
export const DEFAULT_AUDIT_CONFIG = {
    enabled: true,
    tamperDetection: true,
    anomalyThreshold: 0.7,
    defaultRetentionDays: 365,
    realtimeAlerts: true,
    hashAlgorithm: 'sha256',
    digitalSignatures: true,
    batchSize: 1000,
};
export const AUDIT_SEVERITY_LEVELS = {
    low: 0,
    medium: 1,
    high: 2,
    critical: 3,
};
export const COMPLIANCE_REQUIREMENTS = {
    SOC2: 64,
    GDPR: 99,
    HIPAA: 164,
    ISO27001: 114,
    PCI_DSS: 329,
    CCPA: 45,
    NIST: 172,
    FedRAMP: 325,
};
export const DEFAULT_RETENTION_PERIODS = {
    SOC2: 365,
    GDPR: 730,
    HIPAA: 2190,
    ISO27001: 365,
    PCI_DSS: 365,
    CCPA: 730,
    NIST: 1095,
    FedRAMP: 1095,
};
export const RISK_THRESHOLDS = {
    LOW: 25,
    MEDIUM: 50,
    HIGH: 75,
    CRITICAL: 90,
};
export const AuditUtils = {
    getRiskLevel(score) {
        if (score >= RISK_THRESHOLDS.CRITICAL)
            return 'critical';
        if (score >= RISK_THRESHOLDS.HIGH)
            return 'high';
        if (score >= RISK_THRESHOLDS.MEDIUM)
            return 'medium';
        return 'low';
    },
    formatEventType(eventType) {
        return eventType
            .split('.')
            .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
            .join(' ');
    },
    calculateCompliancePercentage(compliant, partial, total) {
        if (total === 0)
            return 0;
        return ((compliant + partial * 0.5) / total) * 100;
    },
    async verifyEventIntegrity(event, previousEvent) {
        return true;
    },
    async generateEventHash(event) {
        const data = JSON.stringify({
            timestamp: event.timestamp,
            event_type: event.event_type,
            user_id: event.user_id,
            action: event.action,
            resource_id: event.resource_id,
            metadata: event.metadata,
        });
        return `hash_${Date.now()}_${Math.random().toString(36).substring(7)}`;
    },
    shouldTriggerAlert(event) {
        return (event.severity === 'critical' ||
            event.anomaly_detected === true ||
            (event.risk_score !== undefined && event.risk_score >= RISK_THRESHOLDS.HIGH));
    },
    getRetentionPeriod(framework) {
        return DEFAULT_RETENTION_PERIODS[framework];
    },
    formatFileSize(bytes) {
        const units = ['B', 'KB', 'MB', 'GB', 'TB'];
        let size = bytes;
        let unitIndex = 0;
        while (size >= 1024 && unitIndex < units.length - 1) {
            size /= 1024;
            unitIndex++;
        }
        return `${size.toFixed(2)} ${units[unitIndex]}`;
    },
    exportToCSV(events) {
        const headers = [
            'Timestamp',
            'Event Type',
            'User Email',
            'User IP',
            'Resource Type',
            'Resource ID',
            'Action',
            'Status',
            'Severity',
            'Risk Score',
            'Description',
        ];
        const rows = events.map((event) => [
            event.timestamp,
            event.event_type,
            event.user_email || '',
            event.user_ip_address,
            event.resource_type || '',
            event.resource_id || '',
            event.action,
            event.status,
            event.severity,
            event.risk_score?.toString() || '',
            event.description,
        ]);
        return [
            headers.join(','),
            ...rows.map((row) => row.map((cell) => `"${cell.toString().replace(/"/g, '""')}"`).join(',')),
        ].join('\n');
    },
};
//# sourceMappingURL=index.js.map