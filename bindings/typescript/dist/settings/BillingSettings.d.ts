import React from 'react';
import { BillingSettings as BillingSettingsType, ToastNotification, ConfirmationDialog, SettingsHistory } from './types';
interface BillingSettingsProps {
    onSave: (section: string, data: BillingSettingsType) => Promise<void>;
    onConfirm: (config: Omit<ConfirmationDialog, 'open'>) => void;
    addToast: (toast: Omit<ToastNotification, 'id'>) => void;
    addToHistory: (entry: Omit<SettingsHistory, 'id' | 'timestamp'>) => void;
}
declare const BillingSettings: React.FC<BillingSettingsProps>;
export default BillingSettings;
//# sourceMappingURL=BillingSettings.d.ts.map