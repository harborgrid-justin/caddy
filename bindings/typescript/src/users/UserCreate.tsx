/**
 * CADDY v0.4.0 - User Creation Wizard
 *
 * Multi-step user creation wizard with:
 * - Basic information collection
 * - Role and team assignment
 * - Security settings configuration
 * - Invitation email sending
 * - Bulk user import
 * - Validation and error handling
 */

import React, { useState, useCallback } from 'react';
import { CreateUserRequest, User } from './types';
import { useCreateUser, useRoles, useTeams, useInvitations } from './UserHooks';

interface UserCreateProps {
  onSuccess?: (user: User) => void;
  onCancel?: () => void;
  defaultRoles?: string[];
  defaultTeams?: string[];
  className?: string;
}

type StepType = 'basic' | 'roles' | 'teams' | 'security' | 'review';

const STEPS: Array<{ id: StepType; label: string; description: string }> = [
  {
    id: 'basic',
    label: 'Basic Information',
    description: 'Enter user details',
  },
  {
    id: 'roles',
    label: 'Roles',
    description: 'Assign user roles',
  },
  {
    id: 'teams',
    label: 'Teams',
    description: 'Add to teams',
  },
  {
    id: 'security',
    label: 'Security',
    description: 'Configure security settings',
  },
  {
    id: 'review',
    label: 'Review',
    description: 'Review and create',
  },
];

export const UserCreate: React.FC<UserCreateProps> = ({
  onSuccess,
  onCancel,
  defaultRoles = [],
  defaultTeams = [],
  className = '',
}) => {
  const [currentStep, setCurrentStep] = useState<StepType>('basic');
  const [formData, setFormData] = useState<CreateUserRequest>({
    username: '',
    email: '',
    firstName: '',
    lastName: '',
    roles: defaultRoles,
    teams: defaultTeams,
    sendInvitation: true,
    skipEmailVerification: false,
  });
  const [errors, setErrors] = useState<Record<string, string>>({});

  const { createUser, loading, error } = useCreateUser();
  const { roles } = useRoles();
  const { teams } = useTeams();
  const { sendInvitation } = useInvitations();

  const currentStepIndex = STEPS.findIndex((s) => s.id === currentStep);

  const validateStep = useCallback(
    (step: StepType): boolean => {
      const newErrors: Record<string, string> = {};

      if (step === 'basic') {
        if (!formData.username) {
          newErrors.username = 'Username is required';
        } else if (!/^[a-zA-Z0-9_-]{3,20}$/.test(formData.username)) {
          newErrors.username = 'Username must be 3-20 characters (alphanumeric, -, _)';
        }

        if (!formData.email) {
          newErrors.email = 'Email is required';
        } else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(formData.email)) {
          newErrors.email = 'Invalid email address';
        }

        if (!formData.firstName) {
          newErrors.firstName = 'First name is required';
        }

        if (!formData.lastName) {
          newErrors.lastName = 'Last name is required';
        }

        if (formData.password && formData.password.length < 8) {
          newErrors.password = 'Password must be at least 8 characters';
        }
      }

      if (step === 'roles' && formData.roles && formData.roles.length === 0) {
        newErrors.roles = 'At least one role must be assigned';
      }

      setErrors(newErrors);
      return Object.keys(newErrors).length === 0;
    },
    [formData]
  );

  const handleNext = useCallback(() => {
    if (validateStep(currentStep)) {
      const nextIndex = currentStepIndex + 1;
      if (nextIndex < STEPS.length) {
        setCurrentStep(STEPS[nextIndex].id);
      }
    }
  }, [currentStep, currentStepIndex, validateStep]);

  const handleBack = useCallback(() => {
    const prevIndex = currentStepIndex - 1;
    if (prevIndex >= 0) {
      setCurrentStep(STEPS[prevIndex].id);
    }
  }, [currentStepIndex]);

  const handleSubmit = useCallback(async () => {
    if (!validateStep('basic')) return;

    try {
      const user = await createUser(formData);

      if (formData.sendInvitation) {
        await sendInvitation({
          email: formData.email,
          firstName: formData.firstName,
          lastName: formData.lastName,
          roles: formData.roles || [],
          teams: formData.teams || [],
        });
      }

      onSuccess?.(user);
    } catch (err) {
      console.error('Failed to create user:', err);
    }
  }, [formData, createUser, sendInvitation, validateStep, onSuccess]);

  const handleRoleToggle = useCallback((roleId: string) => {
    setFormData((prev) => {
      const roles = prev.roles || [];
      const newRoles = roles.includes(roleId)
        ? roles.filter((r) => r !== roleId)
        : [...roles, roleId];
      return { ...prev, roles: newRoles };
    });
  }, []);

  const handleTeamToggle = useCallback((teamId: string) => {
    setFormData((prev) => {
      const teams = prev.teams || [];
      const newTeams = teams.includes(teamId)
        ? teams.filter((t) => t !== teamId)
        : [...teams, teamId];
      return { ...prev, teams: newTeams };
    });
  }, []);

  return (
    <div className={`bg-white shadow sm:rounded-lg ${className}`}>
      <div className="px-4 py-5 sm:p-6">
        <h3 className="text-lg leading-6 font-medium text-gray-900 mb-6">
          Create New User
        </h3>

        <nav aria-label="Progress" className="mb-8">
          <ol className="flex items-center">
            {STEPS.map((step, idx) => (
              <li
                key={step.id}
                className={`relative ${idx !== STEPS.length - 1 ? 'pr-8 sm:pr-20 flex-1' : ''}`}
              >
                {idx !== STEPS.length - 1 && (
                  <div
                    className="absolute inset-0 flex items-center"
                    aria-hidden="true"
                  >
                    <div
                      className={`h-0.5 w-full ${
                        idx < currentStepIndex ? 'bg-indigo-600' : 'bg-gray-200'
                      }`}
                    />
                  </div>
                )}
                <div className="relative flex items-center justify-center">
                  <span
                    className={`h-8 w-8 rounded-full flex items-center justify-center ${
                      idx < currentStepIndex
                        ? 'bg-indigo-600'
                        : idx === currentStepIndex
                        ? 'border-2 border-indigo-600 bg-white'
                        : 'border-2 border-gray-300 bg-white'
                    }`}
                  >
                    {idx < currentStepIndex ? (
                      <svg
                        className="h-5 w-5 text-white"
                        fill="currentColor"
                        viewBox="0 0 20 20"
                      >
                        <path
                          fillRule="evenodd"
                          d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                          clipRule="evenodd"
                        />
                      </svg>
                    ) : (
                      <span
                        className={`text-sm font-medium ${
                          idx === currentStepIndex ? 'text-indigo-600' : 'text-gray-500'
                        }`}
                      >
                        {idx + 1}
                      </span>
                    )}
                  </span>
                </div>
                <div className="absolute top-10 left-1/2 transform -translate-x-1/2 w-32 text-center">
                  <p
                    className={`text-xs font-medium ${
                      idx <= currentStepIndex ? 'text-indigo-600' : 'text-gray-500'
                    }`}
                  >
                    {step.label}
                  </p>
                </div>
              </li>
            ))}
          </ol>
        </nav>

        <div className="mt-16">
          {currentStep === 'basic' && (
            <div className="space-y-6">
              <div className="grid grid-cols-1 gap-6 sm:grid-cols-2">
                <div>
                  <label className="block text-sm font-medium text-gray-700">
                    Username *
                  </label>
                  <input
                    type="text"
                    value={formData.username}
                    onChange={(e) =>
                      setFormData({ ...formData, username: e.target.value })
                    }
                    className={`mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm ${
                      errors.username ? 'border-red-300' : ''
                    }`}
                  />
                  {errors.username && (
                    <p className="mt-1 text-sm text-red-600">{errors.username}</p>
                  )}
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700">
                    Email *
                  </label>
                  <input
                    type="email"
                    value={formData.email}
                    onChange={(e) => setFormData({ ...formData, email: e.target.value })}
                    className={`mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm ${
                      errors.email ? 'border-red-300' : ''
                    }`}
                  />
                  {errors.email && (
                    <p className="mt-1 text-sm text-red-600">{errors.email}</p>
                  )}
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700">
                    First Name *
                  </label>
                  <input
                    type="text"
                    value={formData.firstName}
                    onChange={(e) =>
                      setFormData({ ...formData, firstName: e.target.value })
                    }
                    className={`mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm ${
                      errors.firstName ? 'border-red-300' : ''
                    }`}
                  />
                  {errors.firstName && (
                    <p className="mt-1 text-sm text-red-600">{errors.firstName}</p>
                  )}
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700">
                    Last Name *
                  </label>
                  <input
                    type="text"
                    value={formData.lastName}
                    onChange={(e) =>
                      setFormData({ ...formData, lastName: e.target.value })
                    }
                    className={`mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm ${
                      errors.lastName ? 'border-red-300' : ''
                    }`}
                  />
                  {errors.lastName && (
                    <p className="mt-1 text-sm text-red-600">{errors.lastName}</p>
                  )}
                </div>
                <div className="sm:col-span-2">
                  <label className="block text-sm font-medium text-gray-700">
                    Password (optional - will be generated if not provided)
                  </label>
                  <input
                    type="password"
                    value={formData.password || ''}
                    onChange={(e) =>
                      setFormData({ ...formData, password: e.target.value })
                    }
                    className={`mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm ${
                      errors.password ? 'border-red-300' : ''
                    }`}
                  />
                  {errors.password && (
                    <p className="mt-1 text-sm text-red-600">{errors.password}</p>
                  )}
                </div>
              </div>

              <div className="flex items-start">
                <div className="flex items-center h-5">
                  <input
                    type="checkbox"
                    checked={formData.sendInvitation}
                    onChange={(e) =>
                      setFormData({ ...formData, sendInvitation: e.target.checked })
                    }
                    className="h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded"
                  />
                </div>
                <div className="ml-3 text-sm">
                  <label className="font-medium text-gray-700">
                    Send invitation email
                  </label>
                  <p className="text-gray-500">
                    User will receive an email to set up their account
                  </p>
                </div>
              </div>
            </div>
          )}

          {currentStep === 'roles' && (
            <div>
              <h4 className="text-sm font-medium text-gray-700 mb-4">
                Select Roles (at least one required)
              </h4>
              {errors.roles && (
                <p className="mb-4 text-sm text-red-600">{errors.roles}</p>
              )}
              <div className="space-y-2">
                {roles.map((role) => (
                  <div
                    key={role.id}
                    className={`border rounded-md p-4 cursor-pointer transition ${
                      formData.roles?.includes(role.id)
                        ? 'border-indigo-500 bg-indigo-50'
                        : 'border-gray-200 hover:border-gray-300'
                    }`}
                    onClick={() => handleRoleToggle(role.id)}
                  >
                    <div className="flex items-start">
                      <div className="flex items-center h-5">
                        <input
                          type="checkbox"
                          checked={formData.roles?.includes(role.id)}
                          onChange={() => handleRoleToggle(role.id)}
                          className="h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded"
                        />
                      </div>
                      <div className="ml-3 flex-1">
                        <label className="font-medium text-gray-900">
                          {role.displayName}
                        </label>
                        <p className="text-sm text-gray-500">{role.description}</p>
                        <div className="mt-2 flex flex-wrap gap-1">
                          {role.permissions.slice(0, 3).map((perm, idx) => (
                            <span
                              key={idx}
                              className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800"
                            >
                              {perm.resource}:{perm.action}
                            </span>
                          ))}
                          {role.permissions.length > 3 && (
                            <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800">
                              +{role.permissions.length - 3} more
                            </span>
                          )}
                        </div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {currentStep === 'teams' && (
            <div>
              <h4 className="text-sm font-medium text-gray-700 mb-4">
                Add to Teams (optional)
              </h4>
              <div className="space-y-2">
                {teams.map((team) => (
                  <div
                    key={team.id}
                    className={`border rounded-md p-4 cursor-pointer transition ${
                      formData.teams?.includes(team.id)
                        ? 'border-indigo-500 bg-indigo-50'
                        : 'border-gray-200 hover:border-gray-300'
                    }`}
                    onClick={() => handleTeamToggle(team.id)}
                  >
                    <div className="flex items-start">
                      <div className="flex items-center h-5">
                        <input
                          type="checkbox"
                          checked={formData.teams?.includes(team.id)}
                          onChange={() => handleTeamToggle(team.id)}
                          className="h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded"
                        />
                      </div>
                      <div className="ml-3 flex-1">
                        <label className="font-medium text-gray-900">
                          {team.displayName}
                        </label>
                        <p className="text-sm text-gray-500">{team.description}</p>
                        <p className="text-xs text-gray-400 mt-1">
                          {team.members.length} members • {team.type}
                        </p>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {currentStep === 'security' && (
            <div className="space-y-6">
              <div className="flex items-start">
                <div className="flex items-center h-5">
                  <input
                    type="checkbox"
                    checked={formData.skipEmailVerification}
                    onChange={(e) =>
                      setFormData({
                        ...formData,
                        skipEmailVerification: e.target.checked,
                      })
                    }
                    className="h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded"
                  />
                </div>
                <div className="ml-3 text-sm">
                  <label className="font-medium text-gray-700">
                    Skip email verification
                  </label>
                  <p className="text-gray-500">
                    User account will be active immediately without email verification
                  </p>
                </div>
              </div>

              <div className="bg-yellow-50 border-l-4 border-yellow-400 p-4">
                <div className="flex">
                  <div className="flex-shrink-0">
                    <svg
                      className="h-5 w-5 text-yellow-400"
                      fill="currentColor"
                      viewBox="0 0 20 20"
                    >
                      <path
                        fillRule="evenodd"
                        d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
                        clipRule="evenodd"
                      />
                    </svg>
                  </div>
                  <div className="ml-3">
                    <p className="text-sm text-yellow-700">
                      Security recommendations will be applied automatically including MFA
                      prompts and password requirements.
                    </p>
                  </div>
                </div>
              </div>
            </div>
          )}

          {currentStep === 'review' && (
            <div className="space-y-6">
              <div>
                <h4 className="text-sm font-medium text-gray-900 mb-4">
                  Review User Details
                </h4>
                <dl className="grid grid-cols-1 gap-4 sm:grid-cols-2">
                  <div className="bg-gray-50 p-4 rounded-md">
                    <dt className="text-sm font-medium text-gray-500">Username</dt>
                    <dd className="mt-1 text-sm text-gray-900">{formData.username}</dd>
                  </div>
                  <div className="bg-gray-50 p-4 rounded-md">
                    <dt className="text-sm font-medium text-gray-500">Email</dt>
                    <dd className="mt-1 text-sm text-gray-900">{formData.email}</dd>
                  </div>
                  <div className="bg-gray-50 p-4 rounded-md">
                    <dt className="text-sm font-medium text-gray-500">Name</dt>
                    <dd className="mt-1 text-sm text-gray-900">
                      {formData.firstName} {formData.lastName}
                    </dd>
                  </div>
                  <div className="bg-gray-50 p-4 rounded-md">
                    <dt className="text-sm font-medium text-gray-500">Roles</dt>
                    <dd className="mt-1 text-sm text-gray-900">
                      {formData.roles?.length || 0} assigned
                    </dd>
                  </div>
                  <div className="bg-gray-50 p-4 rounded-md">
                    <dt className="text-sm font-medium text-gray-500">Teams</dt>
                    <dd className="mt-1 text-sm text-gray-900">
                      {formData.teams?.length || 0} assigned
                    </dd>
                  </div>
                  <div className="bg-gray-50 p-4 rounded-md">
                    <dt className="text-sm font-medium text-gray-500">Invitation</dt>
                    <dd className="mt-1 text-sm text-gray-900">
                      {formData.sendInvitation ? 'Will be sent' : 'Will not be sent'}
                    </dd>
                  </div>
                </dl>
              </div>

              {error && (
                <div className="rounded-md bg-red-50 p-4">
                  <div className="flex">
                    <div className="ml-3">
                      <h3 className="text-sm font-medium text-red-800">
                        Error creating user
                      </h3>
                      <div className="mt-2 text-sm text-red-700">{error.message}</div>
                    </div>
                  </div>
                </div>
              )}
            </div>
          )}
        </div>

        <div className="mt-8 flex justify-between">
          <button
            type="button"
            onClick={currentStepIndex === 0 ? onCancel : handleBack}
            className="inline-flex items-center px-4 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
          >
            {currentStepIndex === 0 ? 'Cancel' : 'Back'}
          </button>
          <button
            type="button"
            onClick={currentStep === 'review' ? handleSubmit : handleNext}
            disabled={loading}
            className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50"
          >
            {loading
              ? 'Creating...'
              : currentStep === 'review'
              ? 'Create User'
              : 'Next'}
          </button>
        </div>
      </div>
    </div>
  );
};

export default UserCreate;
