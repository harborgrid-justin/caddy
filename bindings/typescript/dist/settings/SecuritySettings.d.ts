import React from 'react';
import { SecuritySettings as SecuritySettingsType, ToastNotification, ConfirmationDialog, SettingsHistory } from './types';
interface SecuritySettingsProps {
    onSave: (section: string, data: SecuritySettingsType) => Promise<void>;
    onConfirm: (config: Omit<ConfirmationDialog, 'open'>) => void;
    addToast: (toast: Omit<ToastNotification, 'id'>) => void;
    addToHistory: (entry: Omit<SettingsHistory, 'id' | 'timestamp'>) => void;
}
declare const SecuritySettings: React.FC<SecuritySettingsProps>;
export default SecuritySettings;
//# sourceMappingURL=SecuritySettings.d.ts.map