import React from 'react';
import { Permission } from './types';
interface UserPermissionsProps {
    userId: string;
    onPermissionChange?: (permission: Permission, granted: boolean) => void;
    editable?: boolean;
    showInherited?: boolean;
    className?: string;
}
export declare const UserPermissions: React.FC<UserPermissionsProps>;
export default UserPermissions;
//# sourceMappingURL=UserPermissions.d.ts.map