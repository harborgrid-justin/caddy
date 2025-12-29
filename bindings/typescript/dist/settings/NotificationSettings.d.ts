import React from 'react';
import { NotificationSettings as NotificationSettingsType, ToastNotification, ConfirmationDialog, SettingsHistory } from './types';
interface NotificationSettingsProps {
    onSave: (section: string, data: NotificationSettingsType) => Promise<void>;
    onConfirm: (config: Omit<ConfirmationDialog, 'open'>) => void;
    addToast: (toast: Omit<ToastNotification, 'id'>) => void;
    addToHistory: (entry: Omit<SettingsHistory, 'id' | 'timestamp'>) => void;
}
declare const NotificationSettings: React.FC<NotificationSettingsProps>;
export default NotificationSettings;
//# sourceMappingURL=NotificationSettings.d.ts.map