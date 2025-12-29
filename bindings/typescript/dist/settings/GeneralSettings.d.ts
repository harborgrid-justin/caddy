import React from 'react';
import { GeneralSettings as GeneralSettingsType, ToastNotification, ConfirmationDialog, SettingsHistory } from './types';
interface GeneralSettingsProps {
    onSave: (section: string, data: GeneralSettingsType) => Promise<void>;
    onConfirm: (config: Omit<ConfirmationDialog, 'open'>) => void;
    addToast: (toast: Omit<ToastNotification, 'id'>) => void;
    addToHistory: (entry: Omit<SettingsHistory, 'id' | 'timestamp'>) => void;
}
declare const GeneralSettings: React.FC<GeneralSettingsProps>;
export default GeneralSettings;
//# sourceMappingURL=GeneralSettings.d.ts.map