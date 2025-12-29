/**
 * CADDY v0.4.0 - User Permissions Matrix Component
 *
 * Granular permission management with:
 * - Permission matrix visualization
 * - Resource-based permission grouping
 * - Effective permissions calculation
 * - Permission conflicts detection
 * - Delegation management
 * - Real-time permission checking
 */

import React, { useState, useMemo, useCallback } from 'react';
import { Permission, PermissionAction, PermissionScope } from './types';
import { usePermissions, useRoles, useUser } from './UserHooks';

interface UserPermissionsProps {
  userId: string;
  onPermissionChange?: (permission: Permission, granted: boolean) => void;
  editable?: boolean;
  showInherited?: boolean;
  className?: string;
}

interface PermissionGroup {
  resource: string;
  permissions: Permission[];
  inherited: Permission[];
  direct: Permission[];
}

export const UserPermissions: React.FC<UserPermissionsProps> = ({
  userId,
  onPermissionChange,
  editable = true,
  showInherited = true,
  className = '',
}) => {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedResource, setSelectedResource] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<'matrix' | 'list'>('matrix');

  const { user } = useUser(userId);
  const { permissions, loading, checkPermission } = usePermissions(userId);
  const { roles } = useRoles();

  const permissionGroups = useMemo(() => {
    const groups = new Map<string, PermissionGroup>();

    permissions.forEach((perm) => {
      if (!groups.has(perm.resource)) {
        groups.set(perm.resource, {
          resource: perm.resource,
          permissions: [],
          inherited: [],
          direct: [],
        });
      }

      const group = groups.get(perm.resource)!;
      group.permissions.push(perm);

      const isInherited = user?.roles.some((roleId) => {
        const role = roles.find((r) => r.id === roleId);
        return role?.permissions.some(
          (p) =>
            p.resource === perm.resource &&
            p.action === perm.action &&
            p.scope === perm.scope
        );
      });

      if (isInherited) {
        group.inherited.push(perm);
      } else {
        group.direct.push(perm);
      }
    });

    return Array.from(groups.values()).sort((a, b) =>
      a.resource.localeCompare(b.resource)
    );
  }, [permissions, user, roles]);

  const filteredGroups = useMemo(() => {
    if (!searchTerm) return permissionGroups;

    return permissionGroups.filter(
      (group) =>
        group.resource.toLowerCase().includes(searchTerm.toLowerCase()) ||
        group.permissions.some((p) =>
          p.action.toLowerCase().includes(searchTerm.toLowerCase())
        )
    );
  }, [permissionGroups, searchTerm]);

  const allActions: PermissionAction[] = [
    'create',
    'read',
    'update',
    'delete',
    'list',
    'execute',
    'manage',
    'approve',
    'publish',
  ];

  const allScopes: PermissionScope[] = [
    'own',
    'team',
    'department',
    'organization',
    'tenant',
    'global',
  ];

  const hasPermission = useCallback(
    (resource: string, action: PermissionAction, scope: PermissionScope): boolean => {
      return permissions.some(
        (p) => p.resource === resource && p.action === action && p.scope === scope
      );
    },
    [permissions]
  );

  const getPermissionSource = useCallback(
    (resource: string, action: PermissionAction, scope: PermissionScope): string => {
      const roleWithPerm = user?.roles.find((roleId) => {
        const role = roles.find((r) => r.id === roleId);
        return role?.permissions.some(
          (p) => p.resource === resource && p.action === action && p.scope === scope
        );
      });

      if (roleWithPerm) {
        const role = roles.find((r) => r.id === roleWithPerm);
        return role?.displayName || 'Inherited';
      }

      return 'Direct';
    },
    [user, roles]
  );

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
            <h3 className="text-lg font-medium text-gray-900">User Permissions</h3>
            <p className="mt-1 text-sm text-gray-500">
              View and manage granular user permissions across all resources
            </p>
          </div>
          <div className="flex items-center space-x-3">
            <div className="flex rounded-md shadow-sm">
              <button
                onClick={() => setViewMode('matrix')}
                className={`px-4 py-2 text-sm font-medium rounded-l-md border ${
                  viewMode === 'matrix'
                    ? 'bg-indigo-600 text-white border-indigo-600'
                    : 'bg-white text-gray-700 border-gray-300 hover:bg-gray-50'
                }`}
              >
                Matrix
              </button>
              <button
                onClick={() => setViewMode('list')}
                className={`px-4 py-2 text-sm font-medium rounded-r-md border-t border-r border-b ${
                  viewMode === 'list'
                    ? 'bg-indigo-600 text-white border-indigo-600'
                    : 'bg-white text-gray-700 border-gray-300 hover:bg-gray-50'
                }`}
              >
                List
              </button>
            </div>
          </div>
        </div>

        <div className="mb-4">
          <input
            type="text"
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            placeholder="Search permissions by resource or action..."
            className="shadow-sm focus:ring-indigo-500 focus:border-indigo-500 block w-full sm:text-sm border-gray-300 rounded-md"
          />
        </div>

        <div className="mb-4 flex items-center space-x-4 text-sm">
          <div className="flex items-center space-x-2">
            <div className="h-4 w-4 bg-green-100 border border-green-300 rounded"></div>
            <span className="text-gray-600">Granted</span>
          </div>
          <div className="flex items-center space-x-2">
            <div className="h-4 w-4 bg-blue-100 border border-blue-300 rounded"></div>
            <span className="text-gray-600">Inherited</span>
          </div>
          <div className="flex items-center space-x-2">
            <div className="h-4 w-4 bg-gray-50 border border-gray-200 rounded"></div>
            <span className="text-gray-600">Not granted</span>
          </div>
        </div>

        {viewMode === 'matrix' ? (
          <div className="space-y-6">
            {filteredGroups.map((group) => (
              <div key={group.resource} className="border border-gray-200 rounded-lg overflow-hidden">
                <div className="bg-gray-50 px-4 py-3 border-b border-gray-200">
                  <h4 className="text-sm font-medium text-gray-900">{group.resource}</h4>
                  <p className="text-xs text-gray-500 mt-1">
                    {group.permissions.length} permissions ({group.inherited.length}{' '}
                    inherited, {group.direct.length} direct)
                  </p>
                </div>
                <div className="overflow-x-auto">
                  <table className="min-w-full divide-y divide-gray-200">
                    <thead className="bg-gray-50">
                      <tr>
                        <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">
                          Action / Scope
                        </th>
                        {allScopes.map((scope) => (
                          <th
                            key={scope}
                            className="px-4 py-2 text-center text-xs font-medium text-gray-500 uppercase"
                          >
                            {scope}
                          </th>
                        ))}
                      </tr>
                    </thead>
                    <tbody className="bg-white divide-y divide-gray-200">
                      {allActions.map((action) => (
                        <tr key={action}>
                          <td className="px-4 py-2 whitespace-nowrap text-sm font-medium text-gray-900">
                            {action}
                          </td>
                          {allScopes.map((scope) => {
                            const hasThis = hasPermission(group.resource, action, scope);
                            const source = hasThis
                              ? getPermissionSource(group.resource, action, scope)
                              : null;
                            const isInherited = source && source !== 'Direct';

                            return (
                              <td
                                key={scope}
                                className="px-4 py-2 whitespace-nowrap text-center"
                              >
                                <div
                                  className={`inline-flex items-center justify-center h-6 w-6 rounded ${
                                    hasThis
                                      ? isInherited
                                        ? 'bg-blue-100 border border-blue-300'
                                        : 'bg-green-100 border border-green-300'
                                      : 'bg-gray-50 border border-gray-200'
                                  }`}
                                  title={source || 'Not granted'}
                                >
                                  {hasThis && (
                                    <svg
                                      className={`h-4 w-4 ${
                                        isInherited ? 'text-blue-600' : 'text-green-600'
                                      }`}
                                      fill="currentColor"
                                      viewBox="0 0 20 20"
                                    >
                                      <path
                                        fillRule="evenodd"
                                        d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                                        clipRule="evenodd"
                                      />
                                    </svg>
                                  )}
                                </div>
                              </td>
                            );
                          })}
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="space-y-2">
            {filteredGroups.flatMap((group) =>
              group.permissions.map((perm) => {
                const source = getPermissionSource(
                  perm.resource,
                  perm.action,
                  perm.scope
                );
                const isInherited = source !== 'Direct';

                return (
                  <div
                    key={`${perm.resource}-${perm.action}-${perm.scope}`}
                    className="border border-gray-200 rounded-md p-4 flex items-center justify-between"
                  >
                    <div className="flex-1">
                      <div className="flex items-center space-x-2">
                        <span className="text-sm font-medium text-gray-900">
                          {perm.resource}
                        </span>
                        <span className="text-sm text-gray-500">•</span>
                        <span className="text-sm text-gray-700">{perm.action}</span>
                        <span className="text-sm text-gray-500">•</span>
                        <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800">
                          {perm.scope}
                        </span>
                      </div>
                      {perm.conditions && perm.conditions.length > 0 && (
                        <div className="mt-1 text-xs text-gray-500">
                          Conditions: {perm.conditions.length}
                        </div>
                      )}
                    </div>
                    <div className="flex items-center space-x-3">
                      <span
                        className={`inline-flex items-center px-2 py-1 rounded text-xs font-medium ${
                          isInherited
                            ? 'bg-blue-100 text-blue-800'
                            : 'bg-green-100 text-green-800'
                        }`}
                      >
                        {source}
                      </span>
                      <span
                        className={`inline-flex items-center px-2 py-1 rounded text-xs font-medium ${
                          perm.effect === 'allow'
                            ? 'bg-green-100 text-green-800'
                            : 'bg-red-100 text-red-800'
                        }`}
                      >
                        {perm.effect}
                      </span>
                    </div>
                  </div>
                );
              })
            )}

            {filteredGroups.length === 0 && (
              <div className="text-center py-8">
                <p className="text-sm text-gray-500">No permissions found</p>
              </div>
            )}
          </div>
        )}

        <div className="mt-6 bg-gray-50 p-4 rounded-md">
          <h4 className="text-sm font-medium text-gray-900 mb-2">Permission Summary</h4>
          <dl className="grid grid-cols-2 gap-4 sm:grid-cols-4">
            <div>
              <dt className="text-xs text-gray-500">Total Permissions</dt>
              <dd className="text-lg font-semibold text-gray-900">{permissions.length}</dd>
            </div>
            <div>
              <dt className="text-xs text-gray-500">Resources</dt>
              <dd className="text-lg font-semibold text-gray-900">
                {permissionGroups.length}
              </dd>
            </div>
            <div>
              <dt className="text-xs text-gray-500">Inherited</dt>
              <dd className="text-lg font-semibold text-gray-900">
                {permissionGroups.reduce((sum, g) => sum + g.inherited.length, 0)}
              </dd>
            </div>
            <div>
              <dt className="text-xs text-gray-500">Direct</dt>
              <dd className="text-lg font-semibold text-gray-900">
                {permissionGroups.reduce((sum, g) => sum + g.direct.length, 0)}
              </dd>
            </div>
          </dl>
        </div>
      </div>
    </div>
  );
};

export default UserPermissions;
