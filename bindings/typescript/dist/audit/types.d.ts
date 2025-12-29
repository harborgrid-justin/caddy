export type AuditEventType = 'user.login' | 'user.logout' | 'user.created' | 'user.updated' | 'user.deleted' | 'user.password_changed' | 'user.mfa_enabled' | 'user.mfa_disabled' | 'role.created' | 'role.updated' | 'role.deleted' | 'role.assigned' | 'role.revoked' | 'permission.granted' | 'permission.revoked' | 'resource.created' | 'resource.read' | 'resource.updated' | 'resource.deleted' | 'resource.shared' | 'resource.exported' | 'resource.imported' | 'data.accessed' | 'data.modified' | 'data.deleted' | 'data.exported' | 'config.changed' | 'security.breach_attempt' | 'security.unauthorized_access' | 'security.suspicious_activity' | 'system.started' | 'system.stopped' | 'system.error' | 'compliance.violation' | 'audit.tamper_attempt';
export type AuditSeverity = 'low' | 'medium' | 'high' | 'critical';
export type AuditStatus = 'success' | 'failure' | 'pending' | 'blocked';
export type ResourceType = 'project' | 'drawing' | 'model' | 'layer' | 'template' | 'user' | 'role' | 'team' | 'organization' | 'settings' | 'audit_log' | 'report' | 'plugin' | 'workflow';
export type ComplianceFramework = 'SOC2' | 'GDPR' | 'HIPAA' | 'ISO27001' | 'PCI_DSS' | 'CCPA' | 'NIST' | 'FedRAMP';
export type DataClassification = 'public' | 'internal' | 'confidential' | 'restricted';
export type RetentionPolicy = 'short' | 'medium' | 'long' | 'permanent';
export interface AuditEvent {
    id: string;
    timestamp: string;
    event_type: AuditEventType;
    severity: AuditSeverity;
    status: AuditStatus;
    user_id?: string;
    user_email?: string;
    user_name?: string;
    user_ip_address: string;
    user_agent: string;
    session_id: string;
    resource_type?: ResourceType;
    resource_id?: string;
    resource_name?: string;
    action: string;
    description: string;
    metadata: Record<string, unknown>;
    organization_id?: string;
    tenant_id?: string;
    location?: GeoLocation;
    changes?: {
        field: string;
        old_value: unknown;
        new_value: unknown;
    }[];
    risk_score?: number;
    anomaly_detected?: boolean;
    hash: string;
    signature: string;
    previous_event_hash?: string;
    data_classification?: DataClassification;
    compliance_frameworks?: ComplianceFramework[];
    retention_policy: RetentionPolicy;
    created_at: string;
}
export interface AuditFilter {
    event_types?: AuditEventType[];
    severities?: AuditSeverity[];
    statuses?: AuditStatus[];
    user_ids?: string[];
    resource_types?: ResourceType[];
    resource_ids?: string[];
    organization_ids?: string[];
    start_date?: Date;
    end_date?: Date;
    search_query?: string;
    anomaly_only?: boolean;
    min_risk_score?: number;
    ip_address?: string;
    session_id?: string;
}
export interface AuditMetrics {
    total_events: number;
    events_by_type: Record<AuditEventType, number>;
    events_by_severity: Record<AuditSeverity, number>;
    events_by_status: Record<AuditStatus, number>;
    unique_users: number;
    unique_resources: number;
    anomalies_detected: number;
    high_risk_events: number;
    failed_events: number;
    timeline: {
        timestamp: string;
        count: number;
        anomalies: number;
        high_risk: number;
    }[];
    top_users: {
        user_id: string;
        user_email: string;
        event_count: number;
        risk_score: number;
    }[];
    top_resources: {
        resource_type: ResourceType;
        resource_id: string;
        resource_name: string;
        access_count: number;
    }[];
}
export interface GeoLocation {
    country?: string;
    region?: string;
    city?: string;
    latitude?: number;
    longitude?: number;
}
export interface AuditAlert {
    id: string;
    name: string;
    description: string;
    enabled: boolean;
    conditions: {
        event_types?: AuditEventType[];
        severities?: AuditSeverity[];
        min_risk_score?: number;
        user_pattern?: string;
        resource_pattern?: string;
        threshold?: {
            count: number;
            window_seconds: number;
        };
    };
    notification_channels: ('email' | 'slack' | 'webhook' | 'sms')[];
    notification_recipients: string[];
    cooldown_minutes: number;
    created_by: string;
    created_at: string;
    updated_at: string;
    last_triggered?: string;
    trigger_count: number;
}
export interface RetentionRule {
    id: string;
    name: string;
    description: string;
    enabled: boolean;
    event_types?: AuditEventType[];
    severities?: AuditSeverity[];
    data_classifications?: DataClassification[];
    compliance_frameworks?: ComplianceFramework[];
    retention_days: number;
    archive_after_days?: number;
    delete_after_days?: number;
    legal_hold?: boolean;
    legal_hold_reason?: string;
    created_by: string;
    created_at: string;
    updated_at: string;
}
export interface ComplianceRequirement {
    id: string;
    framework: ComplianceFramework;
    requirement_id: string;
    title: string;
    description: string;
    category: string;
    status: 'compliant' | 'non_compliant' | 'partial' | 'not_applicable';
    compliance_percentage: number;
    evidence_required: string[];
    evidence_collected: {
        type: string;
        description: string;
        collected_at: string;
        collected_by: string;
    }[];
    last_assessed: string;
    assessed_by: string;
    next_assessment: string;
    remediation_required: boolean;
    remediation_tasks?: {
        id: string;
        description: string;
        assigned_to?: string;
        due_date?: string;
        status: 'pending' | 'in_progress' | 'completed';
    }[];
}
export interface ComplianceReport {
    id: string;
    framework: ComplianceFramework;
    report_type: 'audit' | 'assessment' | 'certification' | 'gap_analysis';
    start_date: string;
    end_date: string;
    total_requirements: number;
    compliant_count: number;
    non_compliant_count: number;
    partial_count: number;
    overall_compliance_percentage: number;
    requirements: ComplianceRequirement[];
    findings: {
        severity: 'low' | 'medium' | 'high' | 'critical';
        title: string;
        description: string;
        requirement_id: string;
        evidence: string[];
        recommendation: string;
    }[];
    generated_by: string;
    generated_at: string;
    approved_by?: string;
    approved_at?: string;
    status: 'draft' | 'final' | 'approved';
}
export interface DataLineageNode {
    id: string;
    type: 'source' | 'transformation' | 'destination' | 'process';
    name: string;
    description: string;
    data_type?: string;
    schema?: Record<string, unknown>;
    classification: DataClassification;
    compliance_frameworks: ComplianceFramework[];
    retention_policy: RetentionPolicy;
    parent_ids: string[];
    child_ids: string[];
    owner: string;
    created_at: string;
    updated_at: string;
    last_accessed?: string;
}
export interface DataLineageEdge {
    id: string;
    source_id: string;
    target_id: string;
    transformation_type?: string;
    transformation_logic?: string;
    access_control?: {
        requires_approval: boolean;
        approved_by?: string[];
        encryption_required: boolean;
    };
    created_at: string;
}
export interface AuditAnalytics {
    period: {
        start: string;
        end: string;
    };
    event_trends: {
        date: string;
        total: number;
        by_severity: Record<AuditSeverity, number>;
        by_status: Record<AuditStatus, number>;
    }[];
    user_analytics: {
        user_id: string;
        user_email: string;
        total_events: number;
        login_count: number;
        failed_login_count: number;
        data_accessed_count: number;
        data_modified_count: number;
        anomaly_count: number;
        risk_score: number;
        last_activity: string;
    }[];
    resource_analytics: {
        resource_type: ResourceType;
        resource_id: string;
        total_accesses: number;
        unique_users: number;
        modifications: number;
        last_accessed: string;
    }[];
    security_insights: {
        total_anomalies: number;
        breach_attempts: number;
        unauthorized_access_attempts: number;
        suspicious_activities: number;
        high_risk_events: number;
        patterns: {
            type: string;
            description: string;
            occurrence_count: number;
            risk_level: 'low' | 'medium' | 'high' | 'critical';
            affected_users: string[];
            recommendations: string[];
        }[];
    };
    compliance_insights: {
        framework: ComplianceFramework;
        compliant_percentage: number;
        violations: number;
        at_risk_requirements: string[];
    }[];
}
export interface ExportOptions {
    format: 'csv' | 'json' | 'pdf' | 'xlsx';
    filters: AuditFilter;
    include_metadata: boolean;
    include_hash_chain: boolean;
    encrypt: boolean;
    password?: string;
    digital_signature: boolean;
}
export interface AuditDashboardProps {
    organizationId?: string;
    timeRange?: TimeRange;
}
export interface TimeRange {
    start: Date;
    end: Date;
}
export interface AnomalyDetectionResult {
    is_anomaly: boolean;
    anomaly_type?: 'unusual_time' | 'unusual_location' | 'unusual_pattern' | 'privilege_escalation' | 'data_exfiltration';
    confidence_score: number;
    reasons: string[];
    baseline_metrics?: Record<string, number>;
    current_metrics?: Record<string, number>;
}
//# sourceMappingURL=types.d.ts.map