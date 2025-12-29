/**
 * CADDY v0.4.0 - Team Management Component
 *
 * Complete team management with:
 * - Team creation and configuration
 * - Member management
 * - Team hierarchy visualization
 * - Role assignment within teams
 * - Team settings and permissions
 * - Nested team support
 */

import React, { useState, useCallback, useMemo } from 'react';
import { Team, TeamMember, TeamRole, TeamType } from './types';
import { useTeams, useUsers } from './UserHooks';

interface TeamManagementProps {
  onTeamSelect?: (team: Team) => void;
  onTeamCreate?: (team: Team) => void;
  onTeamUpdate?: (team: Team) => void;
  onTeamDelete?: (teamId: string) => void;
  className?: string;
}

export const TeamManagement: React.FC<TeamManagementProps> = ({
  onTeamSelect,
  onTeamCreate,
  onTeamUpdate,
  onTeamDelete,
  className = '',
}) => {
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [selectedTeam, setSelectedTeam] = useState<Team | null>(null);
  const [editingTeam, setEditingTeam] = useState<Partial<Team>>({});
  const [searchTerm, setSearchTerm] = useState('');
  const [viewMode, setViewMode] = useState<'grid' | 'tree'>('grid');

  const { teams, loading, createTeam, updateTeam, deleteTeam, addMember, removeMember } =
    useTeams();
  const { data: usersData } = useUsers({ pageSize: 1000 });

  const filteredTeams = useMemo(() => {
    if (!searchTerm) return teams;
    return teams.filter(
      (team) =>
        team.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
        team.displayName.toLowerCase().includes(searchTerm.toLowerCase()) ||
        team.description.toLowerCase().includes(searchTerm.toLowerCase())
    );
  }, [teams, searchTerm]);

  const teamHierarchy = useMemo(() => {
    const rootTeams = teams.filter((t) => !t.parentTeamId);
    const buildTree = (parentId: string | undefined): Team[] => {
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
    } catch (err) {
      console.error('Failed to create team:', err);
    }
  }, [editingTeam, createTeam, onTeamCreate]);

  const handleUpdateTeam = useCallback(
    async (teamId: string, updates: Partial<Team>) => {
      try {
        const team = await updateTeam(teamId, updates);
        setSelectedTeam(null);
        onTeamUpdate?.(team);
      } catch (err) {
        console.error('Failed to update team:', err);
      }
    },
    [updateTeam, onTeamUpdate]
  );

  const handleDeleteTeam = useCallback(
    async (teamId: string) => {
      if (
        window.confirm(
          'Are you sure you want to delete this team? This action cannot be undone.'
        )
      ) {
        try {
          await deleteTeam(teamId);
          setSelectedTeam(null);
          onTeamDelete?.(teamId);
        } catch (err) {
          console.error('Failed to delete team:', err);
        }
      }
    },
    [deleteTeam, onTeamDelete]
  );

  const handleAddMember = useCallback(
    async (teamId: string, userId: string, role: TeamRole) => {
      try {
        await addMember(teamId, userId, role);
      } catch (err) {
        console.error('Failed to add member:', err);
      }
    },
    [addMember]
  );

  const handleRemoveMember = useCallback(
    async (teamId: string, userId: string) => {
      if (window.confirm('Are you sure you want to remove this member?')) {
        try {
          await removeMember(teamId, userId);
        } catch (err) {
          console.error('Failed to remove member:', err);
        }
      }
    },
    [removeMember]
  );

  const renderTeamTree = (teams: any[], level: number = 0) => {
    return teams.map((team) => (
      <div key={team.id} style={{ marginLeft: `${level * 24}px` }}>
        <div
          className="border border-gray-200 rounded-md p-4 mb-2 hover:border-indigo-300 cursor-pointer"
          onClick={() => {
            setSelectedTeam(team);
            onTeamSelect?.(team);
          }}
        >
          <div className="flex items-center justify-between">
            <div className="flex-1">
              <div className="flex items-center space-x-2">
                <h4 className="text-sm font-medium text-gray-900">{team.displayName}</h4>
                <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800">
                  {team.type}
                </span>
              </div>
              <p className="text-sm text-gray-500 mt-1">{team.description}</p>
              <div className="flex items-center space-x-4 mt-2 text-xs text-gray-500">
                <span>{team.members.length} members</span>
                <span>Level {team.level}</span>
                {team.children && team.children.length > 0 && (
                  <span>{team.children.length} sub-teams</span>
                )}
              </div>
            </div>
          </div>
        </div>
        {team.children && team.children.length > 0 && renderTeamTree(team.children, level + 1)}
      </div>
    ));
  };

  if (loading) {
    return (
      <div className="flex justify-center items-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
      </div>
    );
  }

  return (
    <div className={`bg-white shadow sm:rounded-lg ${className}`}>
      <div className="px-4 py-5 sm:p-6">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h3 className="text-lg font-medium text-gray-900">Team Management</h3>
            <p className="mt-1 text-sm text-gray-500">
              Create and manage teams, members, and hierarchy
            </p>
          </div>
          <div className="flex items-center space-x-3">
            <div className="flex rounded-md shadow-sm">
              <button
                onClick={() => setViewMode('grid')}
                className={`px-4 py-2 text-sm font-medium rounded-l-md border ${
                  viewMode === 'grid'
                    ? 'bg-indigo-600 text-white border-indigo-600'
                    : 'bg-white text-gray-700 border-gray-300 hover:bg-gray-50'
                }`}
              >
                Grid
              </button>
              <button
                onClick={() => setViewMode('tree')}
                className={`px-4 py-2 text-sm font-medium rounded-r-md border-t border-r border-b ${
                  viewMode === 'tree'
                    ? 'bg-indigo-600 text-white border-indigo-600'
                    : 'bg-white text-gray-700 border-gray-300 hover:bg-gray-50'
                }`}
              >
                Tree
              </button>
            </div>
            <button
              onClick={() => setShowCreateModal(true)}
              className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700"
            >
              Create Team
            </button>
          </div>
        </div>

        <div className="mb-4">
          <input
            type="text"
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            placeholder="Search teams..."
            className="shadow-sm focus:ring-indigo-500 focus:border-indigo-500 block w-full sm:text-sm border-gray-300 rounded-md"
          />
        </div>

        {viewMode === 'grid' ? (
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {filteredTeams.map((team) => (
              <div
                key={team.id}
                className="border border-gray-200 rounded-lg p-4 hover:border-indigo-300 cursor-pointer transition"
                onClick={() => {
                  setSelectedTeam(team);
                  onTeamSelect?.(team);
                }}
              >
                <div className="flex items-start justify-between mb-2">
                  <h4 className="text-sm font-medium text-gray-900">{team.displayName}</h4>
                  <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800">
                    {team.type}
                  </span>
                </div>
                <p className="text-sm text-gray-500 mb-3">{team.description}</p>
                <div className="flex items-center justify-between text-xs text-gray-500">
                  <span>{team.members.length} members</span>
                  <span>{team.settings.visibility}</span>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div>{renderTeamTree(teamHierarchy)}</div>
        )}

        {filteredTeams.length === 0 && (
          <div className="text-center py-8">
            <svg
              className="mx-auto h-12 w-12 text-gray-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"
              />
            </svg>
            <h3 className="mt-2 text-sm font-medium text-gray-900">No teams found</h3>
            <p className="mt-1 text-sm text-gray-500">
              Get started by creating a new team.
            </p>
          </div>
        )}
      </div>

      {showCreateModal && (
        <div className="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg shadow-xl max-w-2xl w-full mx-4">
            <div className="px-6 py-4 border-b border-gray-200">
              <h3 className="text-lg font-medium text-gray-900">Create New Team</h3>
            </div>
            <div className="px-6 py-4 space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700">Team Name</label>
                <input
                  type="text"
                  value={editingTeam.name || ''}
                  onChange={(e) => setEditingTeam({ ...editingTeam, name: e.target.value })}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">
                  Display Name
                </label>
                <input
                  type="text"
                  value={editingTeam.displayName || ''}
                  onChange={(e) =>
                    setEditingTeam({ ...editingTeam, displayName: e.target.value })
                  }
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">
                  Description
                </label>
                <textarea
                  value={editingTeam.description || ''}
                  onChange={(e) =>
                    setEditingTeam({ ...editingTeam, description: e.target.value })
                  }
                  rows={3}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">Team Type</label>
                <select
                  value={editingTeam.type || 'project'}
                  onChange={(e) =>
                    setEditingTeam({ ...editingTeam, type: e.target.value as TeamType })
                  }
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                >
                  <option value="department">Department</option>
                  <option value="project">Project</option>
                  <option value="functional">Functional</option>
                  <option value="virtual">Virtual</option>
                  <option value="organization">Organization</option>
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">
                  Parent Team (optional)
                </label>
                <select
                  value={editingTeam.parentTeamId || ''}
                  onChange={(e) =>
                    setEditingTeam({ ...editingTeam, parentTeamId: e.target.value || undefined })
                  }
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                >
                  <option value="">None (root level)</option>
                  {teams.map((team) => (
                    <option key={team.id} value={team.id}>
                      {team.displayName}
                    </option>
                  ))}
                </select>
              </div>
            </div>
            <div className="px-6 py-4 border-t border-gray-200 flex justify-end space-x-3">
              <button
                onClick={() => {
                  setShowCreateModal(false);
                  setEditingTeam({});
                }}
                className="inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
              >
                Cancel
              </button>
              <button
                onClick={handleCreateTeam}
                className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700"
              >
                Create Team
              </button>
            </div>
          </div>
        </div>
      )}

      {selectedTeam && (
        <div className="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg shadow-xl max-w-4xl w-full mx-4 max-h-[90vh] overflow-hidden flex flex-col">
            <div className="px-6 py-4 border-b border-gray-200 flex items-center justify-between">
              <h3 className="text-lg font-medium text-gray-900">
                {selectedTeam.displayName}
              </h3>
              <div className="flex space-x-2">
                <button
                  onClick={() => handleDeleteTeam(selectedTeam.id)}
                  className="text-sm text-red-600 hover:text-red-900"
                >
                  Delete
                </button>
                <button
                  onClick={() => setSelectedTeam(null)}
                  className="text-gray-400 hover:text-gray-600"
                >
                  <svg className="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M6 18L18 6M6 6l12 12"
                    />
                  </svg>
                </button>
              </div>
            </div>
            <div className="flex-1 overflow-y-auto px-6 py-4">
              <div className="space-y-6">
                <div>
                  <h4 className="text-sm font-medium text-gray-900 mb-3">Team Details</h4>
                  <dl className="grid grid-cols-2 gap-4">
                    <div>
                      <dt className="text-sm text-gray-500">Type</dt>
                      <dd className="text-sm text-gray-900 mt-1">{selectedTeam.type}</dd>
                    </div>
                    <div>
                      <dt className="text-sm text-gray-500">Visibility</dt>
                      <dd className="text-sm text-gray-900 mt-1">
                        {selectedTeam.settings.visibility}
                      </dd>
                    </div>
                    <div>
                      <dt className="text-sm text-gray-500">Members</dt>
                      <dd className="text-sm text-gray-900 mt-1">
                        {selectedTeam.members.length}
                      </dd>
                    </div>
                    <div>
                      <dt className="text-sm text-gray-500">Level</dt>
                      <dd className="text-sm text-gray-900 mt-1">{selectedTeam.level}</dd>
                    </div>
                  </dl>
                </div>

                <div>
                  <h4 className="text-sm font-medium text-gray-900 mb-3">
                    Members ({selectedTeam.members.length})
                  </h4>
                  <div className="space-y-2">
                    {selectedTeam.members.map((member) => {
                      const user = usersData?.users.find((u) => u.id === member.userId);
                      return (
                        <div
                          key={member.userId}
                          className="flex items-center justify-between p-3 border border-gray-200 rounded-md"
                        >
                          <div className="flex items-center space-x-3">
                            <div className="h-8 w-8 rounded-full bg-indigo-100 flex items-center justify-center">
                              <span className="text-indigo-700 font-medium text-xs">
                                {user?.firstName[0]}
                                {user?.lastName[0]}
                              </span>
                            </div>
                            <div>
                              <p className="text-sm font-medium text-gray-900">
                                {user?.displayName || member.userId}
                              </p>
                              <p className="text-xs text-gray-500">{member.teamRole}</p>
                            </div>
                          </div>
                          <button
                            onClick={() => handleRemoveMember(selectedTeam.id, member.userId)}
                            className="text-sm text-red-600 hover:text-red-900"
                          >
                            Remove
                          </button>
                        </div>
                      );
                    })}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default TeamManagement;
