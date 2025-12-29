import React from 'react';
import { AdvancedSettings as AdvancedSettingsType, ToastNotification, ConfirmationDialog, SettingsHistory } from './types';
interface AdvancedSettingsProps {
    onSave: (section: string, data: AdvancedSettingsType) => Promise<void>;
    onConfirm: (config: Omit<ConfirmationDialog, 'open'>) => void;
    addToast: (toast: Omit<ToastNotification, 'id'>) => void;
    addToHistory: (entry: Omit<SettingsHistory, 'id' | 'timestamp'>) => void;
}
declare const AdvancedSettings: React.FC<AdvancedSettingsProps>;
export default AdvancedSettings;
//# sourceMappingURL=AdvancedSettings.d.ts.map