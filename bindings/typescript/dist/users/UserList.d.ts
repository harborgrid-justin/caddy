import React from 'react';
import { User } from './types';
interface UserListProps {
    onUserSelect?: (user: User) => void;
    onUserEdit?: (user: User) => void;
    onUserDelete?: (user: User) => void;
    onBulkAction?: (action: string, userIds: string[]) => void;
    compact?: boolean;
    selectable?: boolean;
    className?: string;
}
export declare const UserList: React.FC<UserListProps>;
export default UserList;
//# sourceMappingURL=UserList.d.ts.map