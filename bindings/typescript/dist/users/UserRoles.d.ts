import React from 'react';
interface UserRolesProps {
    userId: string;
    onRoleAssign?: (roleId: string) => void;
    onRoleRemove?: (roleId: string) => void;
    editable?: boolean;
    className?: string;
}
export declare const UserRoles: React.FC<UserRolesProps>;
export default UserRoles;
//# sourceMappingURL=UserRoles.d.ts.map