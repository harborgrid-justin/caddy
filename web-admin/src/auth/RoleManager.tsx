/**
 * Role Manager - RBAC Administration UI
 *
 * Manage roles, permissions, and user assignments
 */

import React, { useState, useEffect } from 'react';
import type { Permission, Role, RoleConstraint } from '../../../bindings/typescript/src/auth';

export const RoleManager: React.FC = () => {
  const [roles, setRoles] = useState<Role[]>([]);
  const [selectedRole, setSelectedRole] = useState<Role | null>(null);
  const [isEditing, setIsEditing] = useState(false);
  const [isCreating, setIsCreating] = useState(false);

  useEffect(() => {
    loadRoles();
  }, []);

  const loadRoles = async () => {
    try {
      const response = await fetch('/api/auth/roles');
      const data = await response.json();
      setRoles(data);
    } catch (error) {
      console.error('Failed to load roles:', error);
    }
  };

  const handleCreateRole = () => {
    setSelectedRole({
      id: '',
      name: '',
      description: '',
      permissions: [],
      parents: [],
      constraints: [],
      isSystem: false,
    });
    setIsCreating(true);
    setIsEditing(true);
  };

  const handleSaveRole = async () => {
    if (!selectedRole) return;

    try {
      const url = isCreating ? '/api/auth/roles' : `/api/auth/roles/${selectedRole.id}`;
      const method = isCreating ? 'POST' : 'PUT';

      await fetch(url, {
        method,
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(selectedRole),
      });

      await loadRoles();
      setIsEditing(false);
      setIsCreating(false);
    } catch (error) {
      console.error('Failed to save role:', error);
    }
  };

  const handleDeleteRole = async (roleId: string) => {
    if (!confirm('Are you sure you want to delete this role?')) return;

    try {
      await fetch(`/api/auth/roles/${roleId}`, { method: 'DELETE' });
      await loadRoles();
      setSelectedRole(null);
    } catch (error) {
      console.error('Failed to delete role:', error);
    }
  };

  return (
    <div className="flex h-screen bg-gray-100 dark:bg-gray-900">
      {/* Role List Sidebar */}
      <div className="w-80 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 overflow-y-auto">
        <div className="p-4 border-b border-gray-200 dark:border-gray-700">
          <div className="flex justify-between items-center mb-4">
            <h2 className="text-xl font-bold text-gray-900 dark:text-white">Roles</h2>
            <button
              onClick={handleCreateRole}
              className="px-3 py-1 bg-blue-600 text-white rounded hover:bg-blue-700"
            >
              + New Role
            </button>
          </div>
        </div>

        <div className="divide-y divide-gray-200 dark:divide-gray-700">
          {roles.map((role) => (
            <button
              key={role.id}
              onClick={() => {
                setSelectedRole(role);
                setIsEditing(false);
                setIsCreating(false);
              }}
              className={`w-full p-4 text-left hover:bg-gray-50 dark:hover:bg-gray-700 ${
                selectedRole?.id === role.id ? 'bg-blue-50 dark:bg-blue-900/20' : ''
              }`}
            >
              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium text-gray-900 dark:text-white">{role.name}</div>
                  <div className="text-sm text-gray-500 dark:text-gray-400">
                    {role.permissions.length} permissions
                  </div>
                </div>
                {role.isSystem && (
                  <span className="px-2 py-1 text-xs bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded">
                    System
                  </span>
                )}
              </div>
            </button>
          ))}
        </div>
      </div>

      {/* Role Details */}
      <div className="flex-1 overflow-y-auto">
        {selectedRole ? (
          <div className="max-w-4xl mx-auto p-6">
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
              {/* Header */}
              <div className="flex justify-between items-start mb-6">
                <div>
                  <h3 className="text-2xl font-bold text-gray-900 dark:text-white">
                    {isCreating ? 'New Role' : selectedRole.name}
                  </h3>
                  {!isCreating && (
                    <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                      Role ID: {selectedRole.id}
                    </p>
                  )}
                </div>
                <div className="flex space-x-2">
                  {!isEditing ? (
                    <>
                      <button
                        onClick={() => setIsEditing(true)}
                        disabled={selectedRole.isSystem}
                        className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
                      >
                        Edit
                      </button>
                      {!selectedRole.isSystem && (
                        <button
                          onClick={() => handleDeleteRole(selectedRole.id)}
                          className="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700"
                        >
                          Delete
                        </button>
                      )}
                    </>
                  ) : (
                    <>
                      <button
                        onClick={handleSaveRole}
                        className="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700"
                      >
                        Save
                      </button>
                      <button
                        onClick={() => {
                          setIsEditing(false);
                          if (isCreating) setSelectedRole(null);
                        }}
                        className="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-700"
                      >
                        Cancel
                      </button>
                    </>
                  )}
                </div>
              </div>

              {/* Role Form */}
              <div className="space-y-6">
                {/* Basic Info */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    Role Name
                  </label>
                  <input
                    type="text"
                    value={selectedRole.name}
                    onChange={(e) => setSelectedRole({ ...selectedRole, name: e.target.value })}
                    disabled={!isEditing}
                    className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded dark:bg-gray-700 dark:text-white disabled:opacity-50"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    Description
                  </label>
                  <textarea
                    value={selectedRole.description}
                    onChange={(e) => setSelectedRole({ ...selectedRole, description: e.target.value })}
                    disabled={!isEditing}
                    rows={3}
                    className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded dark:bg-gray-700 dark:text-white disabled:opacity-50"
                  />
                </div>

                {/* Permissions */}
                <div>
                  <h4 className="text-lg font-medium text-gray-900 dark:text-white mb-3">
                    Permissions ({selectedRole.permissions.length})
                  </h4>
                  <div className="border border-gray-200 dark:border-gray-700 rounded-lg divide-y divide-gray-200 dark:divide-gray-700 max-h-96 overflow-y-auto">
                    {selectedRole.permissions.map((perm, index) => (
                      <div key={index} className="p-3 flex justify-between items-center">
                        <div>
                          <span className="font-mono text-sm text-gray-900 dark:text-white">
                            {perm.resource}:{perm.action}
                            {perm.scope && `:${perm.scope}`}
                          </span>
                        </div>
                        {isEditing && (
                          <button
                            onClick={() => {
                              const newPerms = [...selectedRole.permissions];
                              newPerms.splice(index, 1);
                              setSelectedRole({ ...selectedRole, permissions: newPerms });
                            }}
                            className="text-red-600 hover:text-red-700"
                          >
                            Remove
                          </button>
                        )}
                      </div>
                    ))}
                    {selectedRole.permissions.length === 0 && (
                      <div className="p-6 text-center text-gray-500 dark:text-gray-400">
                        No permissions assigned
                      </div>
                    )}
                  </div>
                </div>

                {/* Parent Roles */}
                <div>
                  <h4 className="text-lg font-medium text-gray-900 dark:text-white mb-3">
                    Inherits From
                  </h4>
                  <div className="flex flex-wrap gap-2">
                    {selectedRole.parents.map((parentId) => (
                      <span
                        key={parentId}
                        className="px-3 py-1 bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-300 rounded-full text-sm"
                      >
                        {roles.find(r => r.id === parentId)?.name || parentId}
                      </span>
                    ))}
                    {selectedRole.parents.length === 0 && (
                      <span className="text-gray-500 dark:text-gray-400 text-sm">
                        No parent roles
                      </span>
                    )}
                  </div>
                </div>
              </div>
            </div>
          </div>
        ) : (
          <div className="flex items-center justify-center h-full">
            <div className="text-center text-gray-500 dark:text-gray-400">
              <div className="text-6xl mb-4">👥</div>
              <p>Select a role to view details</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default RoleManager;
