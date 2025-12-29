/**
 * Role Manager Component
 * Enterprise role and permission management interface
 */

import React, { useState, useEffect } from 'react';
import { Button, Input, Modal, Table, Tooltip } from '../enterprise';
import type {
  Role,
  Permission,
  Action,
  ResourceType,
  BuiltInRole,
  RoleManagerProps,
} from './types';

const ACTIONS: Array<{ value: Action; label: string }> = [
  { value: 'create', label: 'Create' },
  { value: 'read', label: 'Read' },
  { value: 'update', label: 'Update' },
  { value: 'delete', label: 'Delete' },
  { value: 'execute', label: 'Execute' },
  { value: 'share', label: 'Share' },
  { value: 'export', label: 'Export' },
  { value: 'import', label: 'Import' },
  { value: 'approve', label: 'Approve' },
  { value: 'publish', label: 'Publish' },
  { value: 'archive', label: 'Archive' },
  { value: 'restore', label: 'Restore' },
];

const RESOURCES: Array<{ value: ResourceType; label: string }> = [
  { value: 'project', label: 'Projects' },
  { value: 'drawing', label: 'Drawings' },
  { value: 'model', label: '3D Models' },
  { value: 'layer', label: 'Layers' },
  { value: 'template', label: 'Templates' },
  { value: 'user', label: 'Users' },
  { value: 'role', label: 'Roles' },
  { value: 'team', label: 'Teams' },
  { value: 'organization', label: 'Organizations' },
  { value: 'settings', label: 'Settings' },
  { value: 'audit_log', label: 'Audit Logs' },
  { value: 'report', label: 'Reports' },
  { value: 'plugin', label: 'Plugins' },
  { value: 'workflow', label: 'Workflows' },
];

export const RoleManager: React.FC<RoleManagerProps> = ({
  organizationId,
  onRoleCreated,
  onRoleUpdated,
  onRoleDeleted,
}) => {
  const [roles, setRoles] = useState<Role[]>([]);
  const [selectedRole, setSelectedRole] = useState<Role | null>(null);
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [isEditModalOpen, setIsEditModalOpen] = useState(false);
  const [isDeleteModalOpen, setIsDeleteModalOpen] = useState(false);
  const [roleToDelete, setRoleToDelete] = useState<Role | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadRoles();
  }, [organizationId]);

  const loadRoles = async () => {
    setLoading(true);
    try {
      // In production, fetch from API
      const response = await fetch(
        `/api/roles${organizationId ? `?organization_id=${organizationId}` : ''}`
      );
      const data = await response.json();
      setRoles(data.roles || []);
    } catch (error) {
      console.error('Failed to load roles:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCreateRole = () => {
    setIsCreateModalOpen(true);
  };

  const handleEditRole = (role: Role) => {
    setSelectedRole(role);
    setIsEditModalOpen(true);
  };

  const handleDeleteRole = (role: Role) => {
    setRoleToDelete(role);
    setIsDeleteModalOpen(true);
  };

  const confirmDelete = async () => {
    if (!roleToDelete) return;

    try {
      await fetch(`/api/roles/${roleToDelete.id}`, {
        method: 'DELETE',
      });

      setRoles((prev) => prev.filter((r) => r.id !== roleToDelete.id));
      onRoleDeleted?.(roleToDelete.id);
      setIsDeleteModalOpen(false);
      setRoleToDelete(null);
    } catch (error) {
      console.error('Failed to delete role:', error);
    }
  };

  const columns = [
    {
      key: 'name',
      header: 'Role Name',
      render: (role: Role) => (
        <div>
          <div className="font-medium">{role.name}</div>
          {role.built_in && (
            <span className="text-xs bg-blue-100 text-blue-800 px-2 py-1 rounded">
              Built-in
            </span>
          )}
        </div>
      ),
    },
    {
      key: 'description',
      header: 'Description',
      render: (role: Role) => (
        <span className="text-gray-600">{role.description}</span>
      ),
    },
    {
      key: 'permissions',
      header: 'Permissions',
      render: (role: Role) => (
        <span className="text-sm">{role.permissions.length} permissions</span>
      ),
    },
    {
      key: 'status',
      header: 'Status',
      render: (role: Role) => (
        <span
          className={`px-2 py-1 rounded text-xs ${
            role.is_active
              ? 'bg-green-100 text-green-800'
              : 'bg-gray-100 text-gray-800'
          }`}
        >
          {role.is_active ? 'Active' : 'Inactive'}
        </span>
      ),
    },
    {
      key: 'actions',
      header: 'Actions',
      render: (role: Role) => (
        <div className="flex space-x-2">
          <Button
            size="sm"
            variant="ghost"
            onClick={() => handleEditRole(role)}
          >
            Edit
          </Button>
          {!role.built_in && (
            <Button
              size="sm"
              variant="danger"
              onClick={() => handleDeleteRole(role)}
            >
              Delete
            </Button>
          )}
        </div>
      ),
    },
  ];

  return (
    <div className="p-6">
      <div className="flex justify-between items-center mb-6">
        <div>
          <h2 className="text-2xl font-bold">Role Management</h2>
          <p className="text-gray-600 mt-1">
            Manage roles and permissions for your organization
          </p>
        </div>
        <Button onClick={handleCreateRole}>Create Role</Button>
      </div>

      <Table
        columns={columns}
        data={roles}
        loading={loading}
        emptyMessage="No roles found"
      />

      {/* Create Role Modal */}
      <RoleEditor
        isOpen={isCreateModalOpen}
        onClose={() => setIsCreateModalOpen(false)}
        onSave={(role) => {
          setRoles((prev) => [...prev, role]);
          onRoleCreated?.(role);
          setIsCreateModalOpen(false);
        }}
        mode="create"
        organizationId={organizationId}
      />

      {/* Edit Role Modal */}
      {selectedRole && (
        <RoleEditor
          isOpen={isEditModalOpen}
          onClose={() => {
            setIsEditModalOpen(false);
            setSelectedRole(null);
          }}
          onSave={(role) => {
            setRoles((prev) => prev.map((r) => (r.id === role.id ? role : r)));
            onRoleUpdated?.(role);
            setIsEditModalOpen(false);
            setSelectedRole(null);
          }}
          mode="edit"
          role={selectedRole}
          organizationId={organizationId}
        />
      )}

      {/* Delete Confirmation Modal */}
      <Modal
        isOpen={isDeleteModalOpen}
        onClose={() => setIsDeleteModalOpen(false)}
        title="Delete Role"
      >
        <div className="space-y-4">
          <p>
            Are you sure you want to delete the role "{roleToDelete?.name}"?
            This action cannot be undone.
          </p>
          <div className="flex justify-end space-x-2">
            <Button
              variant="secondary"
              onClick={() => setIsDeleteModalOpen(false)}
            >
              Cancel
            </Button>
            <Button variant="danger" onClick={confirmDelete}>
              Delete
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
};

// Role Editor Component
interface RoleEditorProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: (role: Role) => void;
  mode: 'create' | 'edit';
  role?: Role;
  organizationId?: string;
}

const RoleEditor: React.FC<RoleEditorProps> = ({
  isOpen,
  onClose,
  onSave,
  mode,
  role,
  organizationId,
}) => {
  const [name, setName] = useState(role?.name || '');
  const [description, setDescription] = useState(role?.description || '');
  const [permissions, setPermissions] = useState<Permission[]>(
    role?.permissions || []
  );
  const [selectedResource, setSelectedResource] = useState<ResourceType | ''>('');
  const [selectedActions, setSelectedActions] = useState<Set<Action>>(new Set());
  const [errors, setErrors] = useState<Record<string, string>>({});

  const validate = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!name.trim()) {
      newErrors.name = 'Role name is required';
    }

    if (!description.trim()) {
      newErrors.description = 'Description is required';
    }

    if (permissions.length === 0) {
      newErrors.permissions = 'At least one permission is required';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleAddPermissions = () => {
    if (!selectedResource || selectedActions.size === 0) return;

    const newPermissions = Array.from(selectedActions).map((action) => ({
      resource_type: selectedResource,
      action,
    }));

    setPermissions((prev) => [...prev, ...newPermissions]);
    setSelectedResource('');
    setSelectedActions(new Set());
  };

  const handleRemovePermission = (index: number) => {
    setPermissions((prev) => prev.filter((_, i) => i !== index));
  };

  const handleSave = async () => {
    if (!validate()) return;

    const roleData: Role = {
      id: role?.id || crypto.randomUUID(),
      name,
      description,
      permissions,
      parent_roles: role?.parent_roles || [],
      organization_id: organizationId,
      created_by: 'current_user', // Would come from auth context
      created_at: role?.created_at || new Date().toISOString(),
      updated_at: new Date().toISOString(),
      is_active: true,
    };

    try {
      const response = await fetch(
        mode === 'create' ? '/api/roles' : `/api/roles/${role?.id}`,
        {
          method: mode === 'create' ? 'POST' : 'PUT',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(roleData),
        }
      );

      if (response.ok) {
        const savedRole = await response.json();
        onSave(savedRole);
      }
    } catch (error) {
      console.error('Failed to save role:', error);
    }
  };

  const toggleAction = (action: Action) => {
    const newActions = new Set(selectedActions);
    if (newActions.has(action)) {
      newActions.delete(action);
    } else {
      newActions.add(action);
    }
    setSelectedActions(newActions);
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={mode === 'create' ? 'Create Role' : 'Edit Role'}
      size="lg"
    >
      <div className="space-y-6">
        <Input
          label="Role Name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="e.g., Project Manager"
          error={errors.name}
          required
        />

        <div>
          <label className="block text-sm font-medium mb-2">Description</label>
          <textarea
            className="w-full h-20 p-2 border rounded"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="Describe the role and its responsibilities"
          />
          {errors.description && (
            <p className="text-red-500 text-sm mt-1">{errors.description}</p>
          )}
        </div>

        <div className="border-t pt-4">
          <h4 className="font-medium mb-4">Permissions</h4>

          {/* Permission Builder */}
          <div className="bg-gray-50 p-4 rounded mb-4">
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium mb-2">
                  Resource Type
                </label>
                <select
                  className="w-full p-2 border rounded"
                  value={selectedResource}
                  onChange={(e) => setSelectedResource(e.target.value as ResourceType)}
                >
                  <option value="">Select resource...</option>
                  {RESOURCES.map((resource) => (
                    <option key={resource.value} value={resource.value}>
                      {resource.label}
                    </option>
                  ))}
                </select>
              </div>

              {selectedResource && (
                <>
                  <div>
                    <label className="block text-sm font-medium mb-2">
                      Actions
                    </label>
                    <div className="grid grid-cols-4 gap-2">
                      {ACTIONS.map((action) => (
                        <label
                          key={action.value}
                          className="flex items-center space-x-2"
                        >
                          <input
                            type="checkbox"
                            checked={selectedActions.has(action.value)}
                            onChange={() => toggleAction(action.value)}
                            className="rounded"
                          />
                          <span className="text-sm">{action.label}</span>
                        </label>
                      ))}
                    </div>
                  </div>

                  <Button
                    onClick={handleAddPermissions}
                    disabled={selectedActions.size === 0}
                    variant="secondary"
                    fullWidth
                  >
                    Add Permissions
                  </Button>
                </>
              )}
            </div>
          </div>

          {/* Permission List */}
          <div className="space-y-2">
            <h5 className="text-sm font-medium">Assigned Permissions</h5>
            {permissions.length === 0 ? (
              <p className="text-sm text-gray-500">No permissions assigned</p>
            ) : (
              <div className="border rounded divide-y max-h-60 overflow-y-auto">
                {permissions.map((perm, index) => (
                  <div
                    key={index}
                    className="flex justify-between items-center p-2 hover:bg-gray-50"
                  >
                    <div className="text-sm">
                      <span className="font-medium">
                        {RESOURCES.find((r) => r.value === perm.resource_type)
                          ?.label || perm.resource_type}
                      </span>
                      <span className="text-gray-500 mx-2">•</span>
                      <span className="text-blue-600">{perm.action}</span>
                    </div>
                    <button
                      onClick={() => handleRemovePermission(index)}
                      className="text-red-600 hover:text-red-800 text-sm"
                    >
                      Remove
                    </button>
                  </div>
                ))}
              </div>
            )}
            {errors.permissions && (
              <p className="text-red-500 text-sm">{errors.permissions}</p>
            )}
          </div>
        </div>

        <div className="flex justify-end space-x-2 pt-4 border-t">
          <Button variant="secondary" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={handleSave}>
            {mode === 'create' ? 'Create Role' : 'Save Changes'}
          </Button>
        </div>
      </div>
    </Modal>
  );
};
