import React from 'react';
import { AlertSeverity, TimeRange } from './types';
interface AlertHistoryProps {
    service?: string;
    timeRange?: TimeRange;
    severities?: AlertSeverity[];
    className?: string;
}
export declare const AlertHistory: React.FC<AlertHistoryProps>;
export default AlertHistory;
//# sourceMappingURL=AlertHistory.d.ts.map