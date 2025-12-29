import type { AuditEvent, AuditEventType, AuditSeverity, ComplianceFramework } from './types';
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
export type { AuditEvent, AuditEventType, AuditSeverity, AuditStatus, AuditFilter, AuditMetrics, ResourceType, DataClassification, ComplianceFramework, ComplianceRequirement, ComplianceReport, RetentionPolicy, RetentionRule, AuditAlert, AuditAnalytics as AuditAnalyticsData, DataLineageNode, DataLineageEdge, AnomalyDetectionResult, ExportOptions, GeoLocation, TimeRange, } from './types';
export declare const AUDIT_VERSION = "0.4.0";
export interface AuditModuleConfig {
    enabled: boolean;
    tamperDetection: boolean;
    anomalyThreshold: number;
    defaultRetentionDays: number;
    realtimeAlerts: boolean;
    hashAlgorithm: 'sha256' | 'sha512';
    digitalSignatures: boolean;
    batchSize: number;
}
export declare const DEFAULT_AUDIT_CONFIG: AuditModuleConfig;
export declare const AUDIT_SEVERITY_LEVELS: Record<AuditSeverity, number>;
export declare const COMPLIANCE_REQUIREMENTS: Record<ComplianceFramework, number>;
export declare const DEFAULT_RETENTION_PERIODS: Record<ComplianceFramework, number>;
export declare const RISK_THRESHOLDS: {
    readonly LOW: 25;
    readonly MEDIUM: 50;
    readonly HIGH: 75;
    readonly CRITICAL: 90;
};
export declare const AuditUtils: {
    getRiskLevel(score: number): "low" | "medium" | "high" | "critical";
    formatEventType(eventType: AuditEventType): string;
    calculateCompliancePercentage(compliant: number, partial: number, total: number): number;
    verifyEventIntegrity(event: AuditEvent, previousEvent?: AuditEvent): Promise<boolean>;
    generateEventHash(event: Partial<AuditEvent>): Promise<string>;
    shouldTriggerAlert(event: AuditEvent): boolean;
    getRetentionPeriod(framework: ComplianceFramework): number;
    formatFileSize(bytes: number): string;
    exportToCSV(events: AuditEvent[]): string;
};
//# sourceMappingURL=index.d.ts.map