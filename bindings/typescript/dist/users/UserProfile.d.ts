import React from 'react';
import { User } from './types';
interface UserProfileProps {
    userId: string;
    onUpdate?: (user: User) => void;
    onClose?: () => void;
    editable?: boolean;
    showSessions?: boolean;
    showActivity?: boolean;
    className?: string;
}
export declare const UserProfile: React.FC<UserProfileProps>;
export default UserProfile;
//# sourceMappingURL=UserProfile.d.ts.map