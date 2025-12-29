import React from 'react';
import { Incident } from './types';
interface IncidentManagerProps {
    service?: string;
    onIncidentClick?: (incident: Incident) => void;
    className?: string;
}
export declare const IncidentManager: React.FC<IncidentManagerProps>;
export default IncidentManager;
//# sourceMappingURL=IncidentManager.d.ts.map