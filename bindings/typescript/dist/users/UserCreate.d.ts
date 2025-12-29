import React from 'react';
import { User } from './types';
interface UserCreateProps {
    onSuccess?: (user: User) => void;
    onCancel?: () => void;
    defaultRoles?: string[];
    defaultTeams?: string[];
    className?: string;
}
export declare const UserCreate: React.FC<UserCreateProps>;
export default UserCreate;
//# sourceMappingURL=UserCreate.d.ts.map