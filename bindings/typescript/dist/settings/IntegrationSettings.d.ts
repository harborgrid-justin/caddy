import React from 'react';
import { IntegrationSettings as IntegrationSettingsType, ToastNotification, ConfirmationDialog, SettingsHistory } from './types';
interface IntegrationSettingsProps {
    onSave: (section: string, data: IntegrationSettingsType) => Promise<void>;
    onConfirm: (config: Omit<ConfirmationDialog, 'open'>) => void;
    addToast: (toast: Omit<ToastNotification, 'id'>) => void;
    addToHistory: (entry: Omit<SettingsHistory, 'id' | 'timestamp'>) => void;
}
declare const IntegrationSettings: React.FC<IntegrationSettingsProps>;
export default IntegrationSettings;
//# sourceMappingURL=IntegrationSettings.d.ts.map