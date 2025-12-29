import React from 'react';
import { Notification } from './types';
interface NotificationItemProps {
    notification: Notification;
    compact?: boolean;
    showActions?: boolean;
    onSelect?: (id: string) => void;
    selected?: boolean;
}
export declare const NotificationItem: React.FC<NotificationItemProps>;
export {};
//# sourceMappingURL=NotificationItem.d.ts.map