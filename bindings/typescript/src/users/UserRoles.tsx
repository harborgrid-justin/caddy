/**
 * CADDY v0.4.0 - User Roles Management Component
 *
 * Role assignment and management with:
 * - Role hierarchy visualization
 * - Role assignment/removal
 * - Permission inheritance display
 * - Role constraints management
 * - Effective permissions calculation
 * - Temporary role assignments
 */

import React, { useState, useCallback, useMemo } from 'react';
import { Role, RoleAssignment, User } from './types';
import { useRoles, useUser } from './UserHooks';

interface UserRolesProps {
  userId: string;
  onRoleAssign?: (roleId: string) => void;
  onRoleRemove?: (roleId: string) => void;
  editable?: boolean;
  className?: string;
}

export const UserRoles: React.FC<UserRolesProps> = ({
  userId,
  onRoleAssign,
  onRoleRemove,
  editable = true,
  className = '',
}) => {
  const [showAddModal, setShowAddModal] = useState(false);
  const [selectedRole, setSelectedRole] = useState<Role | null>(null);
  const [expandedRoles, setExpandedRoles] = useState<Set<string>>(new Set());

  const { user, loading: userLoading } = useUser(userId);
  const { roles, loading: rolesLoading, assignRole, removeRole } = useRoles();

  const assignedRoles = useMemo(
    () => roles.filter((role) => user?.roles.includes(role.id)),
    [roles, user]
  );

  const availableRoles = useMemo(
    () => roles.filter((role) => !user?.roles.includes(role.id)),
    [roles, user]
  );

  const getRoleHierarchy = useCallback(
    (role: Role): Role[] => {
      const hierarchy: Role[] = [role];
      let current = role;

      while (current.parentRoles.length > 0) {
        const parentId = current.parentRoles[0];
        const parent = roles.find((r) => r.id === parentId);
        if (parent) {
          hierarchy.unshift(parent);
          current = parent;
        } else {
          break;
        }
      }

      return hierarchy;
    },
    [roles]
  );

  const getEffectivePermissions = useCallback(
    (role: Role): Set<string> => {
      const permissions = new Set<string>();
      const hierarchy = getRoleHierarchy(role);

      hierarchy.forEach((r) => {
        r.permissions.forEach((perm) => {
          permissions.add(`${perm.resource}:${perm.action}:${perm.scope}`);
        });
      });

      return permissions;
    },
    [getRoleHierarchy]
  );

  const handleAssignRole = useCallback(
    async (roleId: string) => {
      try {
        await assignRole(userId, roleId);
        setShowAddModal(false);
        onRoleAssign?.(roleId);
      } catch (err) {
        console.error('Failed to assign role:', err);
      }
    },
    [userId, assignRole, onRoleAssign]
  );

  const handleRemoveRole = useCallback(
    async (roleId: string) => {
      if (window.confirm('Are you sure you want to remove this role?')) {
        try {
          await removeRole(userId, roleId);
          onRoleRemove?.(roleId);
        } catch (err) {
          console.error('Failed to remove role:', err);
        }
      }
    },
    [userId, removeRole, onRoleRemove]
  );

  const toggleRoleExpansion = useCallback((roleId: string) => {
    setExpandedRoles((prev) => {
      const next = new Set(prev);
      if (next.has(roleId)) {
        next.delete(roleId);
      } else {
        next.add(roleId);
      }
      return next;
    });
  }, []);

  if (userLoading || rolesLoading) {
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
            <h3 className="text-lg font-medium text-gray-900">User Roles</h3>
            <p className="mt-1 text-sm text-gray-500">
              Manage role assignments and view inherited permissions
            </p>
          </div>
          {editable && (
            <button
              onClick={() => setShowAddModal(true)}
              className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700"
            >
              Assign Role
            </button>
          )}
        </div>

        <div className="space-y-4">
          {assignedRoles.length === 0 ? (
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
                  d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z"
                />
              </svg>
              <h3 className="mt-2 text-sm font-medium text-gray-900">No roles assigned</h3>
              <p className="mt-1 text-sm text-gray-500">
                Get started by assigning a role to this user.
              </p>
            </div>
          ) : (
            assignedRoles.map((role) => {
              const isExpanded = expandedRoles.has(role.id);
              const effectivePerms = getEffectivePermissions(role);
              const hierarchy = getRoleHierarchy(role);

              return (
                <div
                  key={role.id}
                  className="border border-gray-200 rounded-lg overflow-hidden"
                >
                  <div className="bg-gray-50 px-4 py-3 flex items-center justify-between">
                    <div className="flex items-center space-x-3 flex-1">
                      <button
                        onClick={() => toggleRoleExpansion(role.id)}
                        className="text-gray-400 hover:text-gray-600"
                      >
                        <svg
                          className={`h-5 w-5 transform transition-transform ${
                            isExpanded ? 'rotate-90' : ''
                          }`}
                          fill="currentColor"
                          viewBox="0 0 20 20"
                        >
                          <path
                            fillRule="evenodd"
                            d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z"
                            clipRule="evenodd"
                          />
                        </svg>
                      </button>
                      <div className="flex-1">
                        <div className="flex items-center space-x-2">
                          <h4 className="text-sm font-medium text-gray-900">
                            {role.displayName}
                          </h4>
                          {role.isSystem && (
                            <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800">
                              System
                            </span>
                          )}
                        </div>
                        <p className="text-sm text-gray-500">{role.description}</p>
                      </div>
                    </div>
                    <div className="flex items-center space-x-3">
                      <span className="text-xs text-gray-500">
                        {effectivePerms.size} permissions
                      </span>
                      {editable && !role.isSystem && (
                        <button
                          onClick={() => handleRemoveRole(role.id)}
                          className="text-sm text-red-600 hover:text-red-900"
                        >
                          Remove
                        </button>
                      )}
                    </div>
                  </div>

                  {isExpanded && (
                    <div className="px-4 py-4 space-y-4">
                      {hierarchy.length > 1 && (
                        <div>
                          <h5 className="text-xs font-medium text-gray-700 mb-2">
                            Role Hierarchy
                          </h5>
                          <div className="flex items-center space-x-2">
                            {hierarchy.map((h, idx) => (
                              <React.Fragment key={h.id}>
                                {idx > 0 && (
                                  <svg
                                    className="h-4 w-4 text-gray-400"
                                    fill="currentColor"
                                    viewBox="0 0 20 20"
                                  >
                                    <path
                                      fillRule="evenodd"
                                      d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z"
                                      clipRule="evenodd"
                                    />
                                  </svg>
                                )}
                                <span
                                  className={`text-xs px-2 py-1 rounded ${
                                    h.id === role.id
                                      ? 'bg-indigo-100 text-indigo-800 font-medium'
                                      : 'bg-gray-100 text-gray-600'
                                  }`}
                                >
                                  {h.displayName}
                                </span>
                              </React.Fragment>
                            ))}
                          </div>
                        </div>
                      )}

                      <div>
                        <h5 className="text-xs font-medium text-gray-700 mb-2">
                          Permissions ({role.permissions.length} direct)
                        </h5>
                        <div className="grid grid-cols-2 gap-2">
                          {role.permissions.slice(0, 10).map((perm, idx) => (
                            <div
                              key={idx}
                              className="flex items-center justify-between bg-gray-50 px-2 py-1 rounded text-xs"
                            >
                              <span className="text-gray-900">
                                {perm.resource}:{perm.action}
                              </span>
                              <span className="text-gray-500">{perm.scope}</span>
                            </div>
                          ))}
                          {role.permissions.length > 10 && (
                            <div className="col-span-2 text-center text-xs text-gray-500">
                              +{role.permissions.length - 10} more permissions
                            </div>
                          )}
                        </div>
                      </div>

                      {role.constraints.length > 0 && (
                        <div>
                          <h5 className="text-xs font-medium text-gray-700 mb-2">
                            Constraints
                          </h5>
                          <div className="space-y-1">
                            {role.constraints.map((constraint, idx) => (
                              <div
                                key={idx}
                                className="flex items-center space-x-2 text-xs text-gray-600"
                              >
                                <svg
                                  className="h-4 w-4 text-yellow-500"
                                  fill="currentColor"
                                  viewBox="0 0 20 20"
                                >
                                  <path
                                    fillRule="evenodd"
                                    d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
                                    clipRule="evenodd"
                                  />
                                </svg>
                                <span>{constraint.type.replace('_', ' ')}</span>
                              </div>
                            ))}
                          </div>
                        </div>
                      )}
                    </div>
                  )}
                </div>
              );
            })
          )}
        </div>
      </div>

      {showAddModal && (
        <div className="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-[80vh] overflow-hidden flex flex-col">
            <div className="px-6 py-4 border-b border-gray-200">
              <h3 className="text-lg font-medium text-gray-900">Assign Role</h3>
            </div>
            <div className="flex-1 overflow-y-auto px-6 py-4">
              <div className="space-y-2">
                {availableRoles.length === 0 ? (
                  <p className="text-sm text-gray-500 text-center py-8">
                    All available roles have been assigned
                  </p>
                ) : (
                  availableRoles.map((role) => (
                    <div
                      key={role.id}
                      className="border border-gray-200 rounded-md p-4 hover:border-indigo-300 cursor-pointer"
                      onClick={() => setSelectedRole(role)}
                    >
                      <div className="flex items-start justify-between">
                        <div className="flex-1">
                          <div className="flex items-center space-x-2">
                            <h4 className="text-sm font-medium text-gray-900">
                              {role.displayName}
                            </h4>
                            {role.isSystem && (
                              <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800">
                                System
                              </span>
                            )}
                          </div>
                          <p className="text-sm text-gray-500 mt-1">{role.description}</p>
                          <div className="mt-2 flex flex-wrap gap-1">
                            {role.permissions.slice(0, 5).map((perm, idx) => (
                              <span
                                key={idx}
                                className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800"
                              >
                                {perm.resource}:{perm.action}
                              </span>
                            ))}
                            {role.permissions.length > 5 && (
                              <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800">
                                +{role.permissions.length - 5} more
                              </span>
                            )}
                          </div>
                        </div>
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            handleAssignRole(role.id);
                          }}
                          className="ml-4 inline-flex items-center px-3 py-1.5 border border-transparent text-xs font-medium rounded text-white bg-indigo-600 hover:bg-indigo-700"
                        >
                          Assign
                        </button>
                      </div>
                    </div>
                  ))
                )}
              </div>
            </div>
            <div className="px-6 py-4 border-t border-gray-200 flex justify-end">
              <button
                onClick={() => setShowAddModal(false)}
                className="inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default UserRoles;
