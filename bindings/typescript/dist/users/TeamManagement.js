import React, { useState, useCallback, useMemo } from 'react';
import { useTeams, useUsers } from './UserHooks';
export const TeamManagement = ({ onTeamSelect, onTeamCreate, onTeamUpdate, onTeamDelete, className = '', }) => {
    const [showCreateModal, setShowCreateModal] = useState(false);
    const [selectedTeam, setSelectedTeam] = useState(null);
    const [editingTeam, setEditingTeam] = useState({});
    const [searchTerm, setSearchTerm] = useState('');
    const [viewMode, setViewMode] = useState('grid');
    const { teams, loading, createTeam, updateTeam, deleteTeam, addMember, removeMember } = useTeams();
    const { data: usersData } = useUsers({ pageSize: 1000 });
    const filteredTeams = useMemo(() => {
        if (!searchTerm)
            return teams;
        return teams.filter((team) => team.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
            team.displayName.toLowerCase().includes(searchTerm.toLowerCase()) ||
            team.description.toLowerCase().includes(searchTerm.toLowerCase()));
    }, [teams, searchTerm]);
    const teamHierarchy = useMemo(() => {
        const rootTeams = teams.filter((t) => !t.parentTeamId);
        const buildTree = (parentId) => {
            return teams
                .filter((t) => t.parentTeamId === parentId)
                .map((team) => ({
                ...team,
                children: buildTree(team.id),
            }));
        };
        return rootTeams.map((team) => ({
            ...team,
            children: buildTree(team.id),
        }));
    }, [teams]);
    const handleCreateTeam = useCallback(async () => {
        try {
            const team = await createTeam(editingTeam);
            setShowCreateModal(false);
            setEditingTeam({});
            onTeamCreate?.(team);
        }
        catch (err) {
            console.error('Failed to create team:', err);
        }
    }, [editingTeam, createTeam, onTeamCreate]);
    const handleUpdateTeam = useCallback(async (teamId, updates) => {
        try {
            const team = await updateTeam(teamId, updates);
            setSelectedTeam(null);
            onTeamUpdate?.(team);
        }
        catch (err) {
            console.error('Failed to update team:', err);
        }
    }, [updateTeam, onTeamUpdate]);
    const handleDeleteTeam = useCallback(async (teamId) => {
        if (window.confirm('Are you sure you want to delete this team? This action cannot be undone.')) {
            try {
                await deleteTeam(teamId);
                setSelectedTeam(null);
                onTeamDelete?.(teamId);
            }
            catch (err) {
                console.error('Failed to delete team:', err);
            }
        }
    }, [deleteTeam, onTeamDelete]);
    const handleAddMember = useCallback(async (teamId, userId, role) => {
        try {
            await addMember(teamId, userId, role);
        }
        catch (err) {
            console.error('Failed to add member:', err);
        }
    }, [addMember]);
    const handleRemoveMember = useCallback(async (teamId, userId) => {
        if (window.confirm('Are you sure you want to remove this member?')) {
            try {
                await removeMember(teamId, userId);
            }
            catch (err) {
                console.error('Failed to remove member:', err);
            }
        }
    }, [removeMember]);
    const renderTeamTree = (teams, level = 0) => {
        return teams.map((team) => (React.createElement("div", { key: team.id, style: { marginLeft: `${level * 24}px` } },
            React.createElement("div", { className: "border border-gray-200 rounded-md p-4 mb-2 hover:border-indigo-300 cursor-pointer", onClick: () => {
                    setSelectedTeam(team);
                    onTeamSelect?.(team);
                } },
                React.createElement("div", { className: "flex items-center justify-between" },
                    React.createElement("div", { className: "flex-1" },
                        React.createElement("div", { className: "flex items-center space-x-2" },
                            React.createElement("h4", { className: "text-sm font-medium text-gray-900" }, team.displayName),
                            React.createElement("span", { className: "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800" }, team.type)),
                        React.createElement("p", { className: "text-sm text-gray-500 mt-1" }, team.description),
                        React.createElement("div", { className: "flex items-center space-x-4 mt-2 text-xs text-gray-500" },
                            React.createElement("span", null,
                                team.members.length,
                                " members"),
                            React.createElement("span", null,
                                "Level ",
                                team.level),
                            team.children && team.children.length > 0 && (React.createElement("span", null,
                                team.children.length,
                                " sub-teams")))))),
            team.children && team.children.length > 0 && renderTeamTree(team.children, level + 1))));
    };
    if (loading) {
        return (React.createElement("div", { className: "flex justify-center items-center h-64" },
            React.createElement("div", { className: "animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600" })));
    }
    return (React.createElement("div", { className: `bg-white shadow sm:rounded-lg ${className}` },
        React.createElement("div", { className: "px-4 py-5 sm:p-6" },
            React.createElement("div", { className: "flex items-center justify-between mb-6" },
                React.createElement("div", null,
                    React.createElement("h3", { className: "text-lg font-medium text-gray-900" }, "Team Management"),
                    React.createElement("p", { className: "mt-1 text-sm text-gray-500" }, "Create and manage teams, members, and hierarchy")),
                React.createElement("div", { className: "flex items-center space-x-3" },
                    React.createElement("div", { className: "flex rounded-md shadow-sm" },
                        React.createElement("button", { onClick: () => setViewMode('grid'), className: `px-4 py-2 text-sm font-medium rounded-l-md border ${viewMode === 'grid'
                                ? 'bg-indigo-600 text-white border-indigo-600'
                                : 'bg-white text-gray-700 border-gray-300 hover:bg-gray-50'}` }, "Grid"),
                        React.createElement("button", { onClick: () => setViewMode('tree'), className: `px-4 py-2 text-sm font-medium rounded-r-md border-t border-r border-b ${viewMode === 'tree'
                                ? 'bg-indigo-600 text-white border-indigo-600'
                                : 'bg-white text-gray-700 border-gray-300 hover:bg-gray-50'}` }, "Tree")),
                    React.createElement("button", { onClick: () => setShowCreateModal(true), className: "inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700" }, "Create Team"))),
            React.createElement("div", { className: "mb-4" },
                React.createElement("input", { type: "text", value: searchTerm, onChange: (e) => setSearchTerm(e.target.value), placeholder: "Search teams...", className: "shadow-sm focus:ring-indigo-500 focus:border-indigo-500 block w-full sm:text-sm border-gray-300 rounded-md" })),
            viewMode === 'grid' ? (React.createElement("div", { className: "grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3" }, filteredTeams.map((team) => (React.createElement("div", { key: team.id, className: "border border-gray-200 rounded-lg p-4 hover:border-indigo-300 cursor-pointer transition", onClick: () => {
                    setSelectedTeam(team);
                    onTeamSelect?.(team);
                } },
                React.createElement("div", { className: "flex items-start justify-between mb-2" },
                    React.createElement("h4", { className: "text-sm font-medium text-gray-900" }, team.displayName),
                    React.createElement("span", { className: "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800" }, team.type)),
                React.createElement("p", { className: "text-sm text-gray-500 mb-3" }, team.description),
                React.createElement("div", { className: "flex items-center justify-between text-xs text-gray-500" },
                    React.createElement("span", null,
                        team.members.length,
                        " members"),
                    React.createElement("span", null, team.settings.visibility))))))) : (React.createElement("div", null, renderTeamTree(teamHierarchy))),
            filteredTeams.length === 0 && (React.createElement("div", { className: "text-center py-8" },
                React.createElement("svg", { className: "mx-auto h-12 w-12 text-gray-400", fill: "none", stroke: "currentColor", viewBox: "0 0 24 24" },
                    React.createElement("path", { strokeLinecap: "round", strokeLinejoin: "round", strokeWidth: 2, d: "M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" })),
                React.createElement("h3", { className: "mt-2 text-sm font-medium text-gray-900" }, "No teams found"),
                React.createElement("p", { className: "mt-1 text-sm text-gray-500" }, "Get started by creating a new team.")))),
        showCreateModal && (React.createElement("div", { className: "fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center z-50" },
            React.createElement("div", { className: "bg-white rounded-lg shadow-xl max-w-2xl w-full mx-4" },
                React.createElement("div", { className: "px-6 py-4 border-b border-gray-200" },
                    React.createElement("h3", { className: "text-lg font-medium text-gray-900" }, "Create New Team")),
                React.createElement("div", { className: "px-6 py-4 space-y-4" },
                    React.createElement("div", null,
                        React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Team Name"),
                        React.createElement("input", { type: "text", value: editingTeam.name || '', onChange: (e) => setEditingTeam({ ...editingTeam, name: e.target.value }), className: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm" })),
                    React.createElement("div", null,
                        React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Display Name"),
                        React.createElement("input", { type: "text", value: editingTeam.displayName || '', onChange: (e) => setEditingTeam({ ...editingTeam, displayName: e.target.value }), className: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm" })),
                    React.createElement("div", null,
                        React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Description"),
                        React.createElement("textarea", { value: editingTeam.description || '', onChange: (e) => setEditingTeam({ ...editingTeam, description: e.target.value }), rows: 3, className: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm" })),
                    React.createElement("div", null,
                        React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Team Type"),
                        React.createElement("select", { value: editingTeam.type || 'project', onChange: (e) => setEditingTeam({ ...editingTeam, type: e.target.value }), className: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm" },
                            React.createElement("option", { value: "department" }, "Department"),
                            React.createElement("option", { value: "project" }, "Project"),
                            React.createElement("option", { value: "functional" }, "Functional"),
                            React.createElement("option", { value: "virtual" }, "Virtual"),
                            React.createElement("option", { value: "organization" }, "Organization"))),
                    React.createElement("div", null,
                        React.createElement("label", { className: "block text-sm font-medium text-gray-700" }, "Parent Team (optional)"),
                        React.createElement("select", { value: editingTeam.parentTeamId || '', onChange: (e) => setEditingTeam({ ...editingTeam, parentTeamId: e.target.value || undefined }), className: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm" },
                            React.createElement("option", { value: "" }, "None (root level)"),
                            teams.map((team) => (React.createElement("option", { key: team.id, value: team.id }, team.displayName)))))),
                React.createElement("div", { className: "px-6 py-4 border-t border-gray-200 flex justify-end space-x-3" },
                    React.createElement("button", { onClick: () => {
                            setShowCreateModal(false);
                            setEditingTeam({});
                        }, className: "inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50" }, "Cancel"),
                    React.createElement("button", { onClick: handleCreateTeam, className: "inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700" }, "Create Team"))))),
        selectedTeam && (React.createElement("div", { className: "fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center z-50" },
            React.createElement("div", { className: "bg-white rounded-lg shadow-xl max-w-4xl w-full mx-4 max-h-[90vh] overflow-hidden flex flex-col" },
                React.createElement("div", { className: "px-6 py-4 border-b border-gray-200 flex items-center justify-between" },
                    React.createElement("h3", { className: "text-lg font-medium text-gray-900" }, selectedTeam.displayName),
                    React.createElement("div", { className: "flex space-x-2" },
                        React.createElement("button", { onClick: () => handleDeleteTeam(selectedTeam.id), className: "text-sm text-red-600 hover:text-red-900" }, "Delete"),
                        React.createElement("button", { onClick: () => setSelectedTeam(null), className: "text-gray-400 hover:text-gray-600" },
                            React.createElement("svg", { className: "h-6 w-6", fill: "none", stroke: "currentColor", viewBox: "0 0 24 24" },
                                React.createElement("path", { strokeLinecap: "round", strokeLinejoin: "round", strokeWidth: 2, d: "M6 18L18 6M6 6l12 12" }))))),
                React.createElement("div", { className: "flex-1 overflow-y-auto px-6 py-4" },
                    React.createElement("div", { className: "space-y-6" },
                        React.createElement("div", null,
                            React.createElement("h4", { className: "text-sm font-medium text-gray-900 mb-3" }, "Team Details"),
                            React.createElement("dl", { className: "grid grid-cols-2 gap-4" },
                                React.createElement("div", null,
                                    React.createElement("dt", { className: "text-sm text-gray-500" }, "Type"),
                                    React.createElement("dd", { className: "text-sm text-gray-900 mt-1" }, selectedTeam.type)),
                                React.createElement("div", null,
                                    React.createElement("dt", { className: "text-sm text-gray-500" }, "Visibility"),
                                    React.createElement("dd", { className: "text-sm text-gray-900 mt-1" }, selectedTeam.settings.visibility)),
                                React.createElement("div", null,
                                    React.createElement("dt", { className: "text-sm text-gray-500" }, "Members"),
                                    React.createElement("dd", { className: "text-sm text-gray-900 mt-1" }, selectedTeam.members.length)),
                                React.createElement("div", null,
                                    React.createElement("dt", { className: "text-sm text-gray-500" }, "Level"),
                                    React.createElement("dd", { className: "text-sm text-gray-900 mt-1" }, selectedTeam.level)))),
                        React.createElement("div", null,
                            React.createElement("h4", { className: "text-sm font-medium text-gray-900 mb-3" },
                                "Members (",
                                selectedTeam.members.length,
                                ")"),
                            React.createElement("div", { className: "space-y-2" }, selectedTeam.members.map((member) => {
                                const user = usersData?.users.find((u) => u.id === member.userId);
                                return (React.createElement("div", { key: member.userId, className: "flex items-center justify-between p-3 border border-gray-200 rounded-md" },
                                    React.createElement("div", { className: "flex items-center space-x-3" },
                                        React.createElement("div", { className: "h-8 w-8 rounded-full bg-indigo-100 flex items-center justify-center" },
                                            React.createElement("span", { className: "text-indigo-700 font-medium text-xs" },
                                                user?.firstName[0],
                                                user?.lastName[0])),
                                        React.createElement("div", null,
                                            React.createElement("p", { className: "text-sm font-medium text-gray-900" }, user?.displayName || member.userId),
                                            React.createElement("p", { className: "text-xs text-gray-500" }, member.teamRole))),
                                    React.createElement("button", { onClick: () => handleRemoveMember(selectedTeam.id, member.userId), className: "text-sm text-red-600 hover:text-red-900" }, "Remove")));
                            }))))))))));
};
export default TeamManagement;
//# sourceMappingURL=TeamManagement.js.map