import React from 'react';
import type { AuditEvent, AuditFilter } from './types';
interface AuditLogProps {
    organizationId?: string;
    defaultFilters?: AuditFilter;
    onEventSelect?: (event: AuditEvent) => void;
}
export declare const AuditLog: React.FC<AuditLogProps>;
export {};
//# sourceMappingURL=AuditLog.d.ts.map