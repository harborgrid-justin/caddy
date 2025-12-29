import React, { ReactNode } from 'react';
import { NotificationContextValue } from './types';
export declare const NotificationContext: React.Context<NotificationContextValue | null>;
interface NotificationProviderProps {
    children: ReactNode;
    apiUrl?: string;
    wsUrl?: string;
    tenantId: string;
    userId: string;
    autoConnect?: boolean;
    pollInterval?: number;
}
export declare const NotificationProvider: React.FC<NotificationProviderProps>;
export {};
//# sourceMappingURL=NotificationProvider.d.ts.map