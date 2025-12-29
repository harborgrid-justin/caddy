/**
 * CADDY v0.4.0 - User Bulk Actions Component
 *
 * Bulk operations management with:
 * - CSV import/export
 * - Bulk role assignments
 * - Bulk team assignments
 * - Bulk status updates
 * - Progress tracking
 * - Error reporting
 * - Rollback capabilities
 */

import React, { useState, useCallback, useRef } from 'react';
import {
  BulkOperation,
  BulkOperationType,
  ImportUserData,
} from './types';
import { useBulkOperations } from './UserHooks';

interface UserBulkActionsProps {
  onOperationComplete?: (operation: BulkOperation) => void;
  className?: string;
}

export const UserBulkActions: React.FC<UserBulkActionsProps> = ({
  onOperationComplete,
  className = '',
}) => {
  const [selectedAction, setSelectedAction] = useState<BulkOperationType | null>(null);
  const [uploading, setUploading] = useState(false);
  const [importData, setImportData] = useState<ImportUserData[]>([]);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const { operations, loading, createOperation, getOperation } = useBulkOperations();

  const handleFileSelect = useCallback(
    async (event: React.ChangeEvent<HTMLInputElement>) => {
      const file = event.target.files?.[0];
      if (!file) return;

      setSelectedFile(file);
      setUploading(true);

      try {
        const text = await file.text();
        const lines = text.split('\n').filter((line) => line.trim());
        const headers = lines[0].split(',').map((h) => h.trim());

        const data: ImportUserData[] = lines.slice(1).map((line) => {
          const values = line.split(',').map((v) => v.trim());
          const user: any = {};

          headers.forEach((header, index) => {
            const value = values[index];
            if (header === 'roles' || header === 'teams') {
              user[header] = value ? value.split('|') : [];
            } else if (header === 'attributes') {
              try {
                user[header] = value ? JSON.parse(value) : {};
              } catch {
                user[header] = {};
              }
            } else {
              user[header] = value;
            }
          });

          return user as ImportUserData;
        });

        setImportData(data);
      } catch (err) {
        console.error('Failed to parse CSV:', err);
        alert('Failed to parse CSV file. Please check the format.');
      } finally {
        setUploading(false);
      }
    },
    []
  );

  const handleImportUsers = useCallback(async () => {
    if (importData.length === 0) return;

    try {
      const operation = await createOperation('import_users', {
        users: importData,
      });
      onOperationComplete?.(operation);
      setImportData([]);
      setSelectedFile(null);
      setSelectedAction(null);
    } catch (err) {
      console.error('Failed to import users:', err);
      alert('Failed to start import operation.');
    }
  }, [importData, createOperation, onOperationComplete]);

  const handleExportUsers = useCallback(async () => {
    try {
      const operation = await createOperation('export_users', {});
      onOperationComplete?.(operation);
      setSelectedAction(null);
    } catch (err) {
      console.error('Failed to export users:', err);
    }
  }, [createOperation, onOperationComplete]);

  const handleDownloadTemplate = useCallback(() => {
    const template = `username,email,firstName,lastName,roles,teams,department,jobTitle,manager,attributes
john.doe,john.doe@example.com,John,Doe,user|viewer,sales,Sales,Account Manager,jane.smith,"{""employeeId"":""12345""}"
jane.smith,jane.smith@example.com,Jane,Smith,admin,sales|marketing,Sales,Sales Director,,"{""employeeId"":""12346""}"`;

    const blob = new Blob([template], { type: 'text/csv' });
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'user-import-template.csv';
    a.click();
    window.URL.revokeObjectURL(url);
  }, []);

  const getOperationStatusColor = (status: string) => {
    switch (status) {
      case 'completed':
        return 'bg-green-100 text-green-800';
      case 'failed':
        return 'bg-red-100 text-red-800';
      case 'processing':
        return 'bg-blue-100 text-blue-800';
      case 'pending':
        return 'bg-yellow-100 text-yellow-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  const getProgressPercentage = (operation: BulkOperation) => {
    if (operation.totalItems === 0) return 0;
    return Math.round((operation.processedItems / operation.totalItems) * 100);
  };

  return (
    <div className={`bg-white shadow sm:rounded-lg ${className}`}>
      <div className="px-4 py-5 sm:p-6">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h3 className="text-lg font-medium text-gray-900">Bulk Actions</h3>
            <p className="mt-1 text-sm text-gray-500">
              Import, export, and manage users in bulk
            </p>
          </div>
        </div>

        <div className="grid grid-cols-1 gap-4 sm:grid-cols-3 mb-8">
          <button
            onClick={() => setSelectedAction('import_users')}
            className="relative rounded-lg border border-gray-300 bg-white px-6 py-5 shadow-sm flex items-center space-x-3 hover:border-gray-400 focus-within:ring-2 focus-within:ring-offset-2 focus-within:ring-indigo-500"
          >
            <div className="flex-shrink-0">
              <svg
                className="h-6 w-6 text-gray-400"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
                />
              </svg>
            </div>
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium text-gray-900">Import Users</p>
              <p className="text-sm text-gray-500">Upload CSV file</p>
            </div>
          </button>

          <button
            onClick={() => setSelectedAction('export_users')}
            className="relative rounded-lg border border-gray-300 bg-white px-6 py-5 shadow-sm flex items-center space-x-3 hover:border-gray-400 focus-within:ring-2 focus-within:ring-offset-2 focus-within:ring-indigo-500"
          >
            <div className="flex-shrink-0">
              <svg
                className="h-6 w-6 text-gray-400"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3 3m0 0l3-3m-3 3V10"
                />
              </svg>
            </div>
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium text-gray-900">Export Users</p>
              <p className="text-sm text-gray-500">Download CSV file</p>
            </div>
          </button>

          <button
            onClick={handleDownloadTemplate}
            className="relative rounded-lg border border-gray-300 bg-white px-6 py-5 shadow-sm flex items-center space-x-3 hover:border-gray-400 focus-within:ring-2 focus-within:ring-offset-2 focus-within:ring-indigo-500"
          >
            <div className="flex-shrink-0">
              <svg
                className="h-6 w-6 text-gray-400"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                />
              </svg>
            </div>
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium text-gray-900">Download Template</p>
              <p className="text-sm text-gray-500">CSV template file</p>
            </div>
          </button>
        </div>

        {selectedAction === 'import_users' && (
          <div className="border border-gray-200 rounded-lg p-6 mb-6">
            <h4 className="text-sm font-medium text-gray-900 mb-4">Import Users from CSV</h4>

            <div className="mb-4">
              <input
                ref={fileInputRef}
                type="file"
                accept=".csv"
                onChange={handleFileSelect}
                className="hidden"
              />
              <button
                onClick={() => fileInputRef.current?.click()}
                className="inline-flex items-center px-4 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
              >
                <svg
                  className="h-5 w-5 mr-2 text-gray-400"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
                  />
                </svg>
                {selectedFile ? 'Change File' : 'Select CSV File'}
              </button>
              {selectedFile && (
                <span className="ml-3 text-sm text-gray-600">{selectedFile.name}</span>
              )}
            </div>

            {uploading && (
              <div className="flex items-center justify-center py-8">
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
                <span className="ml-3 text-sm text-gray-600">Processing file...</span>
              </div>
            )}

            {importData.length > 0 && (
              <div>
                <div className="mb-4">
                  <p className="text-sm text-gray-700">
                    Found <span className="font-medium">{importData.length}</span> users to
                    import
                  </p>
                </div>

                <div className="max-h-64 overflow-y-auto border border-gray-200 rounded">
                  <table className="min-w-full divide-y divide-gray-200">
                    <thead className="bg-gray-50">
                      <tr>
                        <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">
                          Username
                        </th>
                        <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">
                          Email
                        </th>
                        <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">
                          Name
                        </th>
                        <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">
                          Roles
                        </th>
                      </tr>
                    </thead>
                    <tbody className="bg-white divide-y divide-gray-200">
                      {importData.slice(0, 10).map((user, idx) => (
                        <tr key={idx}>
                          <td className="px-4 py-2 whitespace-nowrap text-sm text-gray-900">
                            {user.username}
                          </td>
                          <td className="px-4 py-2 whitespace-nowrap text-sm text-gray-900">
                            {user.email}
                          </td>
                          <td className="px-4 py-2 whitespace-nowrap text-sm text-gray-900">
                            {user.firstName} {user.lastName}
                          </td>
                          <td className="px-4 py-2 whitespace-nowrap text-sm text-gray-900">
                            {user.roles?.join(', ') || '-'}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                  {importData.length > 10 && (
                    <div className="px-4 py-2 bg-gray-50 text-sm text-gray-500 text-center">
                      And {importData.length - 10} more...
                    </div>
                  )}
                </div>

                <div className="mt-4 flex justify-end space-x-3">
                  <button
                    onClick={() => {
                      setImportData([]);
                      setSelectedFile(null);
                      setSelectedAction(null);
                    }}
                    className="inline-flex items-center px-4 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
                  >
                    Cancel
                  </button>
                  <button
                    onClick={handleImportUsers}
                    className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700"
                  >
                    Import {importData.length} Users
                  </button>
                </div>
              </div>
            )}
          </div>
        )}

        {selectedAction === 'export_users' && (
          <div className="border border-gray-200 rounded-lg p-6 mb-6">
            <h4 className="text-sm font-medium text-gray-900 mb-4">Export Users to CSV</h4>
            <p className="text-sm text-gray-600 mb-4">
              This will export all users with their complete information to a CSV file.
            </p>
            <div className="flex justify-end space-x-3">
              <button
                onClick={() => setSelectedAction(null)}
                className="inline-flex items-center px-4 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
              >
                Cancel
              </button>
              <button
                onClick={handleExportUsers}
                className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700"
              >
                Start Export
              </button>
            </div>
          </div>
        )}

        <div className="mt-8">
          <h4 className="text-sm font-medium text-gray-900 mb-4">Recent Operations</h4>
          {loading ? (
            <div className="flex justify-center py-8">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
            </div>
          ) : operations.length === 0 ? (
            <div className="text-center py-8 text-sm text-gray-500">
              No bulk operations yet
            </div>
          ) : (
            <div className="space-y-4">
              {operations.slice(0, 10).map((operation) => (
                <div
                  key={operation.id}
                  className="border border-gray-200 rounded-lg p-4"
                >
                  <div className="flex items-start justify-between mb-2">
                    <div className="flex-1">
                      <div className="flex items-center space-x-2">
                        <h5 className="text-sm font-medium text-gray-900">
                          {operation.type.replace(/_/g, ' ').toUpperCase()}
                        </h5>
                        <span
                          className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${getOperationStatusColor(
                            operation.status
                          )}`}
                        >
                          {operation.status}
                        </span>
                      </div>
                      <p className="text-xs text-gray-500 mt-1">
                        Started {new Date(operation.startedAt).toLocaleString()}
                      </p>
                    </div>
                    <div className="text-right">
                      <p className="text-sm font-medium text-gray-900">
                        {operation.processedItems} / {operation.totalItems}
                      </p>
                      <p className="text-xs text-gray-500">items processed</p>
                    </div>
                  </div>

                  {operation.status === 'processing' && (
                    <div className="mt-3">
                      <div className="flex items-center justify-between mb-1">
                        <span className="text-xs text-gray-600">Progress</span>
                        <span className="text-xs text-gray-600">
                          {getProgressPercentage(operation)}%
                        </span>
                      </div>
                      <div className="w-full bg-gray-200 rounded-full h-2">
                        <div
                          className="bg-indigo-600 h-2 rounded-full transition-all duration-300"
                          style={{
                            width: `${getProgressPercentage(operation)}%`,
                          }}
                        ></div>
                      </div>
                    </div>
                  )}

                  {operation.status === 'completed' && (
                    <div className="mt-3 grid grid-cols-3 gap-4 text-sm">
                      <div>
                        <dt className="text-gray-500">Successful</dt>
                        <dd className="text-green-600 font-medium">
                          {operation.successfulItems}
                        </dd>
                      </div>
                      <div>
                        <dt className="text-gray-500">Failed</dt>
                        <dd className="text-red-600 font-medium">
                          {operation.failedItems}
                        </dd>
                      </div>
                      <div>
                        <dt className="text-gray-500">Duration</dt>
                        <dd className="text-gray-900 font-medium">
                          {operation.completedAt
                            ? Math.round(
                                (new Date(operation.completedAt).getTime() -
                                  new Date(operation.startedAt).getTime()) /
                                  1000
                              )
                            : 0}
                          s
                        </dd>
                      </div>
                    </div>
                  )}

                  {operation.errors.length > 0 && (
                    <div className="mt-3">
                      <details className="text-sm">
                        <summary className="cursor-pointer text-red-600 font-medium">
                          {operation.errors.length} Error(s)
                        </summary>
                        <div className="mt-2 space-y-1 max-h-32 overflow-y-auto">
                          {operation.errors.slice(0, 5).map((error, idx) => (
                            <div key={idx} className="text-xs text-red-600">
                              Row {error.row}: {error.error}
                            </div>
                          ))}
                          {operation.errors.length > 5 && (
                            <div className="text-xs text-gray-500">
                              And {operation.errors.length - 5} more errors...
                            </div>
                          )}
                        </div>
                      </details>
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default UserBulkActions;
