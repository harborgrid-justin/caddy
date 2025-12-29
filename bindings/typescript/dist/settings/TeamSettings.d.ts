import React from 'react';
import { TeamSettings as TeamSettingsType, ToastNotification, ConfirmationDialog, SettingsHistory } from './types';
interface TeamSettingsProps {
    onSave: (section: string, data: TeamSettingsType) => Promise<void>;
    onConfirm: (config: Omit<ConfirmationDialog, 'open'>) => void;
    addToast: (toast: Omit<ToastNotification, 'id'>) => void;
    addToHistory: (entry: Omit<SettingsHistory, 'id' | 'timestamp'>) => void;
}
declare const TeamSettings: React.FC<TeamSettingsProps>;
export default TeamSettings;
//# sourceMappingURL=TeamSettings.d.ts.map