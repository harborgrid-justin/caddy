import React, { useState, useCallback, useMemo } from 'react';
import { useUsers, useRoles, useTeams, useDebounce, useLocalStorage } from './UserHooks';
export const UserList = ({ onUserSelect, onUserEdit, onUserDelete, onBulkAction, compact = false, selectable = true, className = '', }) => {
    const [searchTerm, setSearchTerm] = useState('');
    const [page, setPage] = useState(1);
    const [pageSize, setPageSize] = useLocalStorage('userListPageSize', 25);
    const [sortBy, setSortBy] = useState('createdAt');
    const [sortOrder, setSortOrder] = useState('desc');
    const [selectedUsers, setSelectedUsers] = useState(new Set());
    const [statusFilter, setStatusFilter] = useState([]);
    const [roleFilter, setRoleFilter] = useState([]);
    const [teamFilter, setTeamFilter] = useState([]);
    const [showFilters, setShowFilters] = useState(false);
    const debouncedSearch = useDebounce(searchTerm, 300);
    const params = useMemo(() => ({
        page,
        pageSize,
        search: debouncedSearch,
        status: statusFilter.length > 0 ? statusFilter : undefined,
        roles: roleFilter.length > 0 ? roleFilter : undefined,
        teams: teamFilter.length > 0 ? teamFilter : undefined,
        sortBy,
        sortOrder,
    }), [page, pageSize, debouncedSearch, statusFilter, roleFilter, teamFilter, sortBy, sortOrder]);
    const { data, loading, error, refetch } = useUsers(params);
    const { roles } = useRoles();
    const { teams } = useTeams();
    const columns = useMemo(() => [
        {
            id: 'user',
            label: 'User',
            sortable: true,
            width: '30%',
            render: (user) => (React.createElement("div", { className: "flex items-center space-x-3" },
                React.createElement("div", { className: "flex-shrink-0" }, user.avatar ? (React.createElement("img", { className: "h-10 w-10 rounded-full", src: user.avatar, alt: user.displayName })) : (React.createElement("div", { className: "h-10 w-10 rounded-full bg-indigo-100 flex items-center justify-center" },
                    React.createElement("span", { className: "text-indigo-700 font-medium text-sm" },
                        user.firstName[0],
                        user.lastName[0])))),
                React.createElement("div", { className: "min-w-0 flex-1" },
                    React.createElement("p", { className: "text-sm font-medium text-gray-900 truncate" }, user.displayName),
                    React.createElement("p", { className: "text-sm text-gray-500 truncate" }, user.email)))),
        },
        {
            id: 'status',
            label: 'Status',
            sortable: true,
            width: '12%',
            render: (user) => (React.createElement("span", { className: `inline-flex rounded-full px-2 text-xs font-semibold leading-5 ${user.status === 'active'
                    ? 'bg-green-100 text-green-800'
                    : user.status === 'inactive'
                        ? 'bg-gray-100 text-gray-800'
                        : user.status === 'suspended'
                            ? 'bg-yellow-100 text-yellow-800'
                            : user.status === 'locked'
                                ? 'bg-red-100 text-red-800'
                                : 'bg-blue-100 text-blue-800'}` }, user.status)),
        },
        {
            id: 'roles',
            label: 'Roles',
            sortable: false,
            width: '20%',
            render: (user) => (React.createElement("div", { className: "flex flex-wrap gap-1" },
                user.roles.slice(0, 2).map((roleId) => {
                    const role = roles.find((r) => r.id === roleId);
                    return (React.createElement("span", { key: roleId, className: "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800" }, role?.displayName || roleId));
                }),
                user.roles.length > 2 && (React.createElement("span", { className: "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800" },
                    "+",
                    user.roles.length - 2)))),
        },
        {
            id: 'teams',
            label: 'Teams',
            sortable: false,
            width: '15%',
            render: (user) => (React.createElement("span", { className: "text-sm text-gray-900" }, user.teams.length > 0 ? user.teams.length : '-')),
        },
        {
            id: 'lastLogin',
            label: 'Last Login',
            sortable: true,
            width: '15%',
            render: (user) => (React.createElement("span", { className: "text-sm text-gray-500" }, user.lastLoginAt
                ? new Date(user.lastLoginAt).toLocaleDateString()
                : 'Never')),
        },
        {
            id: 'actions',
            label: 'Actions',
            sortable: false,
            width: '8%',
            render: (user) => (React.createElement("div", { className: "flex space-x-2" },
                React.createElement("button", { onClick: () => onUserEdit?.(user), className: "text-indigo-600 hover:text-indigo-900 text-sm font-medium" }, "Edit"),
                React.createElement("button", { onClick: () => onUserDelete?.(user), className: "text-red-600 hover:text-red-900 text-sm font-medium" }, "Delete"))),
        },
    ], [roles, onUserEdit, onUserDelete]);
    const handleSort = useCallback((columnId) => {
        setSortBy(columnId);
        setSortOrder((prev) => (prev === 'asc' ? 'desc' : 'asc'));
    }, []);
    const handleSelectAll = useCallback(() => {
        if (data && data.users.length > 0) {
            if (selectedUsers.size === data.users.length) {
                setSelectedUsers(new Set());
            }
            else {
                setSelectedUsers(new Set(data.users.map((u) => u.id)));
            }
        }
    }, [data, selectedUsers]);
    const handleSelectUser = useCallback((userId) => {
        setSelectedUsers((prev) => {
            const next = new Set(prev);
            if (next.has(userId)) {
                next.delete(userId);
            }
            else {
                next.add(userId);
            }
            return next;
        });
    }, []);
    const handleBulkAction = useCallback((action) => {
        if (selectedUsers.size > 0) {
            onBulkAction?.(action, Array.from(selectedUsers));
            setSelectedUsers(new Set());
        }
    }, [selectedUsers, onBulkAction]);
    const handleExport = useCallback(async () => {
        try {
            const blob = await fetch('/api/users/export', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(params),
            }).then((r) => r.blob());
            const url = window.URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `users-${new Date().toISOString()}.csv`;
            a.click();
            window.URL.revokeObjectURL(url);
        }
        catch (error) {
            console.error('Export failed:', error);
        }
    }, [params]);
    if (error) {
        return (React.createElement("div", { className: "rounded-md bg-red-50 p-4" },
            React.createElement("div", { className: "flex" },
                React.createElement("div", { className: "ml-3" },
                    React.createElement("h3", { className: "text-sm font-medium text-red-800" }, "Error loading users"),
                    React.createElement("div", { className: "mt-2 text-sm text-red-700" }, error.message)))));
    }
    return (React.createElement("div", { className: `bg-white shadow rounded-lg ${className}` },
        React.createElement("div", { className: "px-4 py-5 border-b border-gray-200 sm:px-6" },
            React.createElement("div", { className: "flex items-center justify-between flex-wrap sm:flex-nowrap" },
                React.createElement("div", { className: "flex-1 min-w-0" },
                    React.createElement("h3", { className: "text-lg leading-6 font-medium text-gray-900" }, "Users"),
                    data && (React.createElement("p", { className: "mt-1 text-sm text-gray-500" },
                        data.total,
                        " total users, showing ",
                        data.users.length))),
                React.createElement("div", { className: "flex items-center space-x-3" },
                    React.createElement("button", { onClick: () => setShowFilters(!showFilters), className: "inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500" },
                        React.createElement("svg", { className: "h-4 w-4 mr-2", fill: "none", stroke: "currentColor", viewBox: "0 0 24 24" },
                            React.createElement("path", { strokeLinecap: "round", strokeLinejoin: "round", strokeWidth: 2, d: "M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z" })),
                        "Filters"),
                    React.createElement("button", { onClick: handleExport, className: "inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50" }, "Export"),
                    React.createElement("button", { onClick: refetch, className: "inline-flex items-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700" }, "Refresh"))),
            React.createElement("div", { className: "mt-4" },
                React.createElement("input", { type: "text", value: searchTerm, onChange: (e) => setSearchTerm(e.target.value), placeholder: "Search users by name, email, or username...", className: "shadow-sm focus:ring-indigo-500 focus:border-indigo-500 block w-full sm:text-sm border-gray-300 rounded-md" })),
            showFilters && (React.createElement("div", { className: "mt-4 grid grid-cols-1 gap-4 sm:grid-cols-3" },
                React.createElement("div", null,
                    React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Status"),
                    React.createElement("select", { multiple: true, value: statusFilter, onChange: (e) => setStatusFilter(Array.from(e.target.selectedOptions, (o) => o.value)), className: "mt-1 block w-full pl-3 pr-10 py-2 text-base border-gray-300 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm rounded-md" },
                        React.createElement("option", { value: "active" }, "Active"),
                        React.createElement("option", { value: "inactive" }, "Inactive"),
                        React.createElement("option", { value: "pending" }, "Pending"),
                        React.createElement("option", { value: "suspended" }, "Suspended"),
                        React.createElement("option", { value: "locked" }, "Locked"))),
                React.createElement("div", null,
                    React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Roles"),
                    React.createElement("select", { multiple: true, value: roleFilter, onChange: (e) => setRoleFilter(Array.from(e.target.selectedOptions, (o) => o.value)), className: "mt-1 block w-full pl-3 pr-10 py-2 text-base border-gray-300 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm rounded-md" }, roles.map((role) => (React.createElement("option", { key: role.id, value: role.id }, role.displayName))))),
                React.createElement("div", null,
                    React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Teams"),
                    React.createElement("select", { multiple: true, value: teamFilter, onChange: (e) => setTeamFilter(Array.from(e.target.selectedOptions, (o) => o.value)), className: "mt-1 block w-full pl-3 pr-10 py-2 text-base border-gray-300 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm rounded-md" }, teams.map((team) => (React.createElement("option", { key: team.id, value: team.id }, team.displayName))))))),
            selectedUsers.size > 0 && (React.createElement("div", { className: "mt-4 bg-indigo-50 p-3 rounded-md flex items-center justify-between" },
                React.createElement("span", { className: "text-sm text-indigo-700" },
                    selectedUsers.size,
                    " user",
                    selectedUsers.size > 1 ? 's' : '',
                    " selected"),
                React.createElement("div", { className: "flex space-x-2" },
                    React.createElement("button", { onClick: () => handleBulkAction('assign_role'), className: "text-sm text-indigo-600 hover:text-indigo-900 font-medium" }, "Assign Role"),
                    React.createElement("button", { onClick: () => handleBulkAction('add_to_team'), className: "text-sm text-indigo-600 hover:text-indigo-900 font-medium" }, "Add to Team"),
                    React.createElement("button", { onClick: () => handleBulkAction('deactivate'), className: "text-sm text-red-600 hover:text-red-900 font-medium" }, "Deactivate"))))),
        React.createElement("div", { className: "overflow-x-auto" },
            React.createElement("table", { className: "min-w-full divide-y divide-gray-200" },
                React.createElement("thead", { className: "bg-gray-50" },
                    React.createElement("tr", null,
                        selectable && (React.createElement("th", { scope: "col", className: "px-6 py-3 w-12" },
                            React.createElement("input", { type: "checkbox", checked: data?.users.length > 0 && selectedUsers.size === data?.users.length, onChange: handleSelectAll, className: "h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded" }))),
                        columns.map((column) => (React.createElement("th", { key: column.id, scope: "col", className: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", style: { width: column.width } }, column.sortable ? (React.createElement("button", { onClick: () => handleSort(column.id), className: "group inline-flex items-center space-x-1 hover:text-gray-900" },
                            React.createElement("span", null, column.label),
                            React.createElement("svg", { className: "h-4 w-4", fill: "none", stroke: "currentColor", viewBox: "0 0 24 24" },
                                React.createElement("path", { strokeLinecap: "round", strokeLinejoin: "round", strokeWidth: 2, d: "M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4" })))) : (column.label)))))),
                React.createElement("tbody", { className: "bg-white divide-y divide-gray-200" }, loading ? (React.createElement("tr", null,
                    React.createElement("td", { colSpan: columns.length + (selectable ? 1 : 0), className: "px-6 py-4" },
                        React.createElement("div", { className: "flex justify-center" },
                            React.createElement("div", { className: "animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600" }))))) : data?.users.length === 0 ? (React.createElement("tr", null,
                    React.createElement("td", { colSpan: columns.length + (selectable ? 1 : 0), className: "px-6 py-4 text-center text-sm text-gray-500" }, "No users found"))) : (data?.users.map((user) => (React.createElement("tr", { key: user.id, onClick: () => onUserSelect?.(user), className: "hover:bg-gray-50 cursor-pointer" },
                    selectable && (React.createElement("td", { className: "px-6 py-4" },
                        React.createElement("input", { type: "checkbox", checked: selectedUsers.has(user.id), onChange: (e) => {
                                e.stopPropagation();
                                handleSelectUser(user.id);
                            }, onClick: (e) => e.stopPropagation(), className: "h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded" }))),
                    columns.map((column) => (React.createElement("td", { key: column.id, className: "px-6 py-4 whitespace-nowrap" }, column.render ? column.render(user) : null)))))))))),
        data && data.totalPages > 1 && (React.createElement("div", { className: "bg-white px-4 py-3 flex items-center justify-between border-t border-gray-200 sm:px-6" },
            React.createElement("div", { className: "flex-1 flex justify-between sm:hidden" },
                React.createElement("button", { onClick: () => setPage((p) => Math.max(1, p - 1)), disabled: page === 1, className: "relative inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50" }, "Previous"),
                React.createElement("button", { onClick: () => setPage((p) => Math.min(data.totalPages, p + 1)), disabled: page === data.totalPages, className: "ml-3 relative inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50" }, "Next")),
            React.createElement("div", { className: "hidden sm:flex-1 sm:flex sm:items-center sm:justify-between" },
                React.createElement("div", null,
                    React.createElement("p", { className: "text-sm text-gray-700" },
                        "Showing",
                        ' ',
                        React.createElement("span", { className: "font-medium" }, (page - 1) * pageSize + 1),
                        " to",
                        ' ',
                        React.createElement("span", { className: "font-medium" }, Math.min(page * pageSize, data.total)),
                        ' ',
                        "of ",
                        React.createElement("span", { className: "font-medium" }, data.total),
                        " results")),
                React.createElement("div", { className: "flex items-center space-x-2" },
                    React.createElement("select", { value: pageSize, onChange: (e) => setPageSize(Number(e.target.value)), className: "text-sm border-gray-300 rounded-md" },
                        React.createElement("option", { value: "10" }, "10 per page"),
                        React.createElement("option", { value: "25" }, "25 per page"),
                        React.createElement("option", { value: "50" }, "50 per page"),
                        React.createElement("option", { value: "100" }, "100 per page")),
                    React.createElement("nav", { className: "relative z-0 inline-flex rounded-md shadow-sm -space-x-px" },
                        React.createElement("button", { onClick: () => setPage((p) => Math.max(1, p - 1)), disabled: page === 1, className: "relative inline-flex items-center px-2 py-2 rounded-l-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50 disabled:opacity-50" }, "Previous"),
                        Array.from({ length: Math.min(5, data.totalPages) }, (_, i) => i + 1).map((pageNum) => (React.createElement("button", { key: pageNum, onClick: () => setPage(pageNum), className: `relative inline-flex items-center px-4 py-2 border text-sm font-medium ${page === pageNum
                                ? 'z-10 bg-indigo-50 border-indigo-500 text-indigo-600'
                                : 'bg-white border-gray-300 text-gray-500 hover:bg-gray-50'}` }, pageNum))),
                        React.createElement("button", { onClick: () => setPage((p) => Math.min(data.totalPages, p + 1)), disabled: page === data.totalPages, className: "relative inline-flex items-center px-2 py-2 rounded-r-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50 disabled:opacity-50" }, "Next"))))))));
};
export default UserList;
//# sourceMappingURL=UserList.js.map