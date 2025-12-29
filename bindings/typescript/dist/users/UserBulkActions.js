import React, { useState, useCallback, useRef } from 'react';
import { useBulkOperations } from './UserHooks';
export const UserBulkActions = ({ onOperationComplete, className = '', }) => {
    const [selectedAction, setSelectedAction] = useState(null);
    const [uploading, setUploading] = useState(false);
    const [importData, setImportData] = useState([]);
    const [selectedFile, setSelectedFile] = useState(null);
    const fileInputRef = useRef(null);
    const { operations, loading, createOperation, getOperation } = useBulkOperations();
    const handleFileSelect = useCallback(async (event) => {
        const file = event.target.files?.[0];
        if (!file)
            return;
        setSelectedFile(file);
        setUploading(true);
        try {
            const text = await file.text();
            const lines = text.split('\n').filter((line) => line.trim());
            const headers = lines[0].split(',').map((h) => h.trim());
            const data = lines.slice(1).map((line) => {
                const values = line.split(',').map((v) => v.trim());
                const user = {};
                headers.forEach((header, index) => {
                    const value = values[index];
                    if (header === 'roles' || header === 'teams') {
                        user[header] = value ? value.split('|') : [];
                    }
                    else if (header === 'attributes') {
                        try {
                            user[header] = value ? JSON.parse(value) : {};
                        }
                        catch {
                            user[header] = {};
                        }
                    }
                    else {
                        user[header] = value;
                    }
                });
                return user;
            });
            setImportData(data);
        }
        catch (err) {
            console.error('Failed to parse CSV:', err);
            alert('Failed to parse CSV file. Please check the format.');
        }
        finally {
            setUploading(false);
        }
    }, []);
    const handleImportUsers = useCallback(async () => {
        if (importData.length === 0)
            return;
        try {
            const operation = await createOperation('import_users', {
                users: importData,
            });
            onOperationComplete?.(operation);
            setImportData([]);
            setSelectedFile(null);
            setSelectedAction(null);
        }
        catch (err) {
            console.error('Failed to import users:', err);
            alert('Failed to start import operation.');
        }
    }, [importData, createOperation, onOperationComplete]);
    const handleExportUsers = useCallback(async () => {
        try {
            const operation = await createOperation('export_users', {});
            onOperationComplete?.(operation);
            setSelectedAction(null);
        }
        catch (err) {
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
    const getOperationStatusColor = (status) => {
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
    const getProgressPercentage = (operation) => {
        if (operation.totalItems === 0)
            return 0;
        return Math.round((operation.processedItems / operation.totalItems) * 100);
    };
    return (React.createElement("div", { className: `bg-white shadow sm:rounded-lg ${className}` },
        React.createElement("div", { className: "px-4 py-5 sm:p-6" },
            React.createElement("div", { className: "flex items-center justify-between mb-6" },
                React.createElement("div", null,
                    React.createElement("h3", { className: "text-lg font-medium text-gray-900" }, "Bulk Actions"),
                    React.createElement("p", { className: "mt-1 text-sm text-gray-500" }, "Import, export, and manage users in bulk"))),
            React.createElement("div", { className: "grid grid-cols-1 gap-4 sm:grid-cols-3 mb-8" },
                React.createElement("button", { onClick: () => setSelectedAction('import_users'), className: "relative rounded-lg border border-gray-300 bg-white px-6 py-5 shadow-sm flex items-center space-x-3 hover:border-gray-400 focus-within:ring-2 focus-within:ring-offset-2 focus-within:ring-indigo-500" },
                    React.createElement("div", { className: "flex-shrink-0" },
                        React.createElement("svg", { className: "h-6 w-6 text-gray-400", fill: "none", stroke: "currentColor", viewBox: "0 0 24 24" },
                            React.createElement("path", { strokeLinecap: "round", strokeLinejoin: "round", strokeWidth: 2, d: "M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" }))),
                    React.createElement("div", { className: "flex-1 min-w-0" },
                        React.createElement("p", { className: "text-sm font-medium text-gray-900" }, "Import Users"),
                        React.createElement("p", { className: "text-sm text-gray-500" }, "Upload CSV file"))),
                React.createElement("button", { onClick: () => setSelectedAction('export_users'), className: "relative rounded-lg border border-gray-300 bg-white px-6 py-5 shadow-sm flex items-center space-x-3 hover:border-gray-400 focus-within:ring-2 focus-within:ring-offset-2 focus-within:ring-indigo-500" },
                    React.createElement("div", { className: "flex-shrink-0" },
                        React.createElement("svg", { className: "h-6 w-6 text-gray-400", fill: "none", stroke: "currentColor", viewBox: "0 0 24 24" },
                            React.createElement("path", { strokeLinecap: "round", strokeLinejoin: "round", strokeWidth: 2, d: "M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3 3m0 0l3-3m-3 3V10" }))),
                    React.createElement("div", { className: "flex-1 min-w-0" },
                        React.createElement("p", { className: "text-sm font-medium text-gray-900" }, "Export Users"),
                        React.createElement("p", { className: "text-sm text-gray-500" }, "Download CSV file"))),
                React.createElement("button", { onClick: handleDownloadTemplate, className: "relative rounded-lg border border-gray-300 bg-white px-6 py-5 shadow-sm flex items-center space-x-3 hover:border-gray-400 focus-within:ring-2 focus-within:ring-offset-2 focus-within:ring-indigo-500" },
                    React.createElement("div", { className: "flex-shrink-0" },
                        React.createElement("svg", { className: "h-6 w-6 text-gray-400", fill: "none", stroke: "currentColor", viewBox: "0 0 24 24" },
                            React.createElement("path", { strokeLinecap: "round", strokeLinejoin: "round", strokeWidth: 2, d: "M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" }))),
                    React.createElement("div", { className: "flex-1 min-w-0" },
                        React.createElement("p", { className: "text-sm font-medium text-gray-900" }, "Download Template"),
                        React.createElement("p", { className: "text-sm text-gray-500" }, "CSV template file")))),
            selectedAction === 'import_users' && (React.createElement("div", { className: "border border-gray-200 rounded-lg p-6 mb-6" },
                React.createElement("h4", { className: "text-sm font-medium text-gray-900 mb-4" }, "Import Users from CSV"),
                React.createElement("div", { className: "mb-4" },
                    React.createElement("input", { ref: fileInputRef, type: "file", accept: ".csv", onChange: handleFileSelect, className: "hidden" }),
                    React.createElement("button", { onClick: () => fileInputRef.current?.click(), className: "inline-flex items-center px-4 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50" },
                        React.createElement("svg", { className: "h-5 w-5 mr-2 text-gray-400", fill: "none", stroke: "currentColor", viewBox: "0 0 24 24" },
                            React.createElement("path", { strokeLinecap: "round", strokeLinejoin: "round", strokeWidth: 2, d: "M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" })),
                        selectedFile ? 'Change File' : 'Select CSV File'),
                    selectedFile && (React.createElement("span", { className: "ml-3 text-sm text-gray-600" }, selectedFile.name))),
                uploading && (React.createElement("div", { className: "flex items-center justify-center py-8" },
                    React.createElement("div", { className: "animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600" }),
                    React.createElement("span", { className: "ml-3 text-sm text-gray-600" }, "Processing file..."))),
                importData.length > 0 && (React.createElement("div", null,
                    React.createElement("div", { className: "mb-4" },
                        React.createElement("p", { className: "text-sm text-gray-700" },
                            "Found ",
                            React.createElement("span", { className: "font-medium" }, importData.length),
                            " users to import")),
                    React.createElement("div", { className: "max-h-64 overflow-y-auto border border-gray-200 rounded" },
                        React.createElement("table", { className: "min-w-full divide-y divide-gray-200" },
                            React.createElement("thead", { className: "bg-gray-50" },
                                React.createElement("tr", null,
                                    React.createElement("th", { className: "px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase" }, "Username"),
                                    React.createElement("th", { className: "px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase" }, "Email"),
                                    React.createElement("th", { className: "px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase" }, "Name"),
                                    React.createElement("th", { className: "px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase" }, "Roles"))),
                            React.createElement("tbody", { className: "bg-white divide-y divide-gray-200" }, importData.slice(0, 10).map((user, idx) => (React.createElement("tr", { key: idx },
                                React.createElement("td", { className: "px-4 py-2 whitespace-nowrap text-sm text-gray-900" }, user.username),
                                React.createElement("td", { className: "px-4 py-2 whitespace-nowrap text-sm text-gray-900" }, user.email),
                                React.createElement("td", { className: "px-4 py-2 whitespace-nowrap text-sm text-gray-900" },
                                    user.firstName,
                                    " ",
                                    user.lastName),
                                React.createElement("td", { className: "px-4 py-2 whitespace-nowrap text-sm text-gray-900" }, user.roles?.join(', ') || '-')))))),
                        importData.length > 10 && (React.createElement("div", { className: "px-4 py-2 bg-gray-50 text-sm text-gray-500 text-center" },
                            "And ",
                            importData.length - 10,
                            " more..."))),
                    React.createElement("div", { className: "mt-4 flex justify-end space-x-3" },
                        React.createElement("button", { onClick: () => {
                                setImportData([]);
                                setSelectedFile(null);
                                setSelectedAction(null);
                            }, className: "inline-flex items-center px-4 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50" }, "Cancel"),
                        React.createElement("button", { onClick: handleImportUsers, className: "inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700" },
                            "Import ",
                            importData.length,
                            " Users")))))),
            selectedAction === 'export_users' && (React.createElement("div", { className: "border border-gray-200 rounded-lg p-6 mb-6" },
                React.createElement("h4", { className: "text-sm font-medium text-gray-900 mb-4" }, "Export Users to CSV"),
                React.createElement("p", { className: "text-sm text-gray-600 mb-4" }, "This will export all users with their complete information to a CSV file."),
                React.createElement("div", { className: "flex justify-end space-x-3" },
                    React.createElement("button", { onClick: () => setSelectedAction(null), className: "inline-flex items-center px-4 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50" }, "Cancel"),
                    React.createElement("button", { onClick: handleExportUsers, className: "inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700" }, "Start Export")))),
            React.createElement("div", { className: "mt-8" },
                React.createElement("h4", { className: "text-sm font-medium text-gray-900 mb-4" }, "Recent Operations"),
                loading ? (React.createElement("div", { className: "flex justify-center py-8" },
                    React.createElement("div", { className: "animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600" }))) : operations.length === 0 ? (React.createElement("div", { className: "text-center py-8 text-sm text-gray-500" }, "No bulk operations yet")) : (React.createElement("div", { className: "space-y-4" }, operations.slice(0, 10).map((operation) => (React.createElement("div", { key: operation.id, className: "border border-gray-200 rounded-lg p-4" },
                    React.createElement("div", { className: "flex items-start justify-between mb-2" },
                        React.createElement("div", { className: "flex-1" },
                            React.createElement("div", { className: "flex items-center space-x-2" },
                                React.createElement("h5", { className: "text-sm font-medium text-gray-900" }, operation.type.replace(/_/g, ' ').toUpperCase()),
                                React.createElement("span", { className: `inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${getOperationStatusColor(operation.status)}` }, operation.status)),
                            React.createElement("p", { className: "text-xs text-gray-500 mt-1" },
                                "Started ",
                                new Date(operation.startedAt).toLocaleString())),
                        React.createElement("div", { className: "text-right" },
                            React.createElement("p", { className: "text-sm font-medium text-gray-900" },
                                operation.processedItems,
                                " / ",
                                operation.totalItems),
                            React.createElement("p", { className: "text-xs text-gray-500" }, "items processed"))),
                    operation.status === 'processing' && (React.createElement("div", { className: "mt-3" },
                        React.createElement("div", { className: "flex items-center justify-between mb-1" },
                            React.createElement("span", { className: "text-xs text-gray-600" }, "Progress"),
                            React.createElement("span", { className: "text-xs text-gray-600" },
                                getProgressPercentage(operation),
                                "%")),
                        React.createElement("div", { className: "w-full bg-gray-200 rounded-full h-2" },
                            React.createElement("div", { className: "bg-indigo-600 h-2 rounded-full transition-all duration-300", style: {
                                    width: `${getProgressPercentage(operation)}%`,
                                } })))),
                    operation.status === 'completed' && (React.createElement("div", { className: "mt-3 grid grid-cols-3 gap-4 text-sm" },
                        React.createElement("div", null,
                            React.createElement("dt", { className: "text-gray-500" }, "Successful"),
                            React.createElement("dd", { className: "text-green-600 font-medium" }, operation.successfulItems)),
                        React.createElement("div", null,
                            React.createElement("dt", { className: "text-gray-500" }, "Failed"),
                            React.createElement("dd", { className: "text-red-600 font-medium" }, operation.failedItems)),
                        React.createElement("div", null,
                            React.createElement("dt", { className: "text-gray-500" }, "Duration"),
                            React.createElement("dd", { className: "text-gray-900 font-medium" },
                                operation.completedAt
                                    ? Math.round((new Date(operation.completedAt).getTime() -
                                        new Date(operation.startedAt).getTime()) /
                                        1000)
                                    : 0,
                                "s")))),
                    operation.errors.length > 0 && (React.createElement("div", { className: "mt-3" },
                        React.createElement("details", { className: "text-sm" },
                            React.createElement("summary", { className: "cursor-pointer text-red-600 font-medium" },
                                operation.errors.length,
                                " Error(s)"),
                            React.createElement("div", { className: "mt-2 space-y-1 max-h-32 overflow-y-auto" },
                                operation.errors.slice(0, 5).map((error, idx) => (React.createElement("div", { key: idx, className: "text-xs text-red-600" },
                                    "Row ",
                                    error.row,
                                    ": ",
                                    error.error))),
                                operation.errors.length > 5 && (React.createElement("div", { className: "text-xs text-gray-500" },
                                    "And ",
                                    operation.errors.length - 5,
                                    " more errors...")))))))))))))));
};
export default UserBulkActions;
//# sourceMappingURL=UserBulkActions.js.map