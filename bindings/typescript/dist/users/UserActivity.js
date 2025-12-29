import React, { useState, useMemo, useCallback } from 'react';
import { useUserActivity, useUserSessions } from './UserHooks';
export const UserActivity = ({ userId, limit = 100, showSessions = true, showSummary = true, className = '', }) => {
    const [activeTab, setActiveTab] = useState('activity');
    const [categoryFilter, setCategoryFilter] = useState([]);
    const [severityFilter, setSeverityFilter] = useState([]);
    const [dateRange, setDateRange] = useState({});
    const [searchTerm, setSearchTerm] = useState('');
    const { activity, loading: activityLoading } = useUserActivity(userId, limit);
    const { sessions, loading: sessionsLoading, terminateSession } = useUserSessions(userId);
    const { summary, loading: summaryLoading } = useUserActivity(userId);
    const filteredActivity = useMemo(() => {
        return activity.filter((log) => {
            if (categoryFilter.length > 0 && !categoryFilter.includes(log.category)) {
                return false;
            }
            if (severityFilter.length > 0 && !severityFilter.includes(log.severity)) {
                return false;
            }
            if (searchTerm) {
                const search = searchTerm.toLowerCase();
                return (log.action.toLowerCase().includes(search) ||
                    log.resource.toLowerCase().includes(search) ||
                    log.details.description.toLowerCase().includes(search));
            }
            return true;
        });
    }, [activity, categoryFilter, severityFilter, searchTerm]);
    const activeSessions = useMemo(() => sessions.filter((s) => s.status === 'active'), [sessions]);
    const handleExportActivity = useCallback(async () => {
        try {
            const data = JSON.stringify(filteredActivity, null, 2);
            const blob = new Blob([data], { type: 'application/json' });
            const url = window.URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `user-activity-${userId}-${new Date().toISOString()}.json`;
            a.click();
            window.URL.revokeObjectURL(url);
        }
        catch (err) {
            console.error('Failed to export activity:', err);
        }
    }, [filteredActivity, userId]);
    const getSeverityColor = (severity) => {
        switch (severity) {
            case 'critical':
                return 'bg-red-500';
            case 'error':
                return 'bg-red-400';
            case 'warning':
                return 'bg-yellow-500';
            default:
                return 'bg-green-500';
        }
    };
    const getCategoryIcon = (category) => {
        const iconClass = 'h-5 w-5 text-white';
        switch (category) {
            case 'authentication':
                return (React.createElement("svg", { className: iconClass, fill: "currentColor", viewBox: "0 0 20 20" },
                    React.createElement("path", { fillRule: "evenodd", d: "M5 9V7a5 5 0 0110 0v2a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2zm8-2v2H7V7a3 3 0 016 0z", clipRule: "evenodd" })));
            case 'authorization':
            case 'security':
                return (React.createElement("svg", { className: iconClass, fill: "currentColor", viewBox: "0 0 20 20" },
                    React.createElement("path", { fillRule: "evenodd", d: "M2.166 4.999A11.954 11.954 0 0010 1.944 11.954 11.954 0 0017.834 5c.11.65.166 1.32.166 2.001 0 5.225-3.34 9.67-8 11.317C5.34 16.67 2 12.225 2 7c0-.682.057-1.35.166-2.001zm11.541 3.708a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z", clipRule: "evenodd" })));
            default:
                return (React.createElement("svg", { className: iconClass, fill: "currentColor", viewBox: "0 0 20 20" },
                    React.createElement("path", { fillRule: "evenodd", d: "M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z", clipRule: "evenodd" })));
        }
    };
    if (activityLoading || sessionsLoading) {
        return (React.createElement("div", { className: "flex justify-center items-center h-64" },
            React.createElement("div", { className: "animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600" })));
    }
    return (React.createElement("div", { className: `bg-white shadow sm:rounded-lg ${className}` },
        React.createElement("div", { className: "px-4 py-5 sm:p-6" },
            React.createElement("div", { className: "flex items-center justify-between mb-6" },
                React.createElement("div", null,
                    React.createElement("h3", { className: "text-lg font-medium text-gray-900" }, "User Activity"),
                    React.createElement("p", { className: "mt-1 text-sm text-gray-500" }, "View activity logs, sessions, and usage patterns")),
                React.createElement("button", { onClick: handleExportActivity, className: "inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50" }, "Export")),
            React.createElement("div", { className: "border-b border-gray-200 mb-6" },
                React.createElement("nav", { className: "-mb-px flex space-x-8" },
                    React.createElement("button", { onClick: () => setActiveTab('activity'), className: `${activeTab === 'activity'
                            ? 'border-indigo-500 text-indigo-600'
                            : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'} whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm` },
                        "Activity Log",
                        React.createElement("span", { className: `ml-2 py-0.5 px-2.5 rounded-full text-xs font-medium ${activeTab === 'activity'
                                ? 'bg-indigo-100 text-indigo-600'
                                : 'bg-gray-100 text-gray-900'}` }, filteredActivity.length)),
                    showSessions && (React.createElement("button", { onClick: () => setActiveTab('sessions'), className: `${activeTab === 'sessions'
                            ? 'border-indigo-500 text-indigo-600'
                            : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'} whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm` },
                        "Sessions",
                        React.createElement("span", { className: `ml-2 py-0.5 px-2.5 rounded-full text-xs font-medium ${activeTab === 'sessions'
                                ? 'bg-indigo-100 text-indigo-600'
                                : 'bg-gray-100 text-gray-900'}` }, activeSessions.length))),
                    showSummary && (React.createElement("button", { onClick: () => setActiveTab('summary'), className: `${activeTab === 'summary'
                            ? 'border-indigo-500 text-indigo-600'
                            : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'} whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm` }, "Summary")))),
            activeTab === 'activity' && (React.createElement("div", null,
                React.createElement("div", { className: "mb-4 grid grid-cols-1 gap-4 sm:grid-cols-3" },
                    React.createElement("input", { type: "text", value: searchTerm, onChange: (e) => setSearchTerm(e.target.value), placeholder: "Search activity...", className: "shadow-sm focus:ring-indigo-500 focus:border-indigo-500 block w-full sm:text-sm border-gray-300 rounded-md" }),
                    React.createElement("select", { multiple: true, value: categoryFilter, onChange: (e) => setCategoryFilter(Array.from(e.target.selectedOptions, (o) => o.value)), className: "block w-full text-sm border-gray-300 rounded-md" },
                        React.createElement("option", { value: "authentication" }, "Authentication"),
                        React.createElement("option", { value: "authorization" }, "Authorization"),
                        React.createElement("option", { value: "user_management" }, "User Management"),
                        React.createElement("option", { value: "role_management" }, "Role Management"),
                        React.createElement("option", { value: "team_management" }, "Team Management"),
                        React.createElement("option", { value: "sso" }, "SSO"),
                        React.createElement("option", { value: "data_access" }, "Data Access"),
                        React.createElement("option", { value: "configuration" }, "Configuration"),
                        React.createElement("option", { value: "security" }, "Security")),
                    React.createElement("select", { multiple: true, value: severityFilter, onChange: (e) => setSeverityFilter(Array.from(e.target.selectedOptions, (o) => o.value)), className: "block w-full text-sm border-gray-300 rounded-md" },
                        React.createElement("option", { value: "info" }, "Info"),
                        React.createElement("option", { value: "warning" }, "Warning"),
                        React.createElement("option", { value: "error" }, "Error"),
                        React.createElement("option", { value: "critical" }, "Critical"))),
                React.createElement("div", { className: "flow-root" },
                    React.createElement("ul", { className: "-mb-8" }, filteredActivity.map((log, idx) => (React.createElement("li", { key: log.id },
                        React.createElement("div", { className: "relative pb-8" },
                            idx !== filteredActivity.length - 1 && (React.createElement("span", { className: "absolute top-5 left-5 -ml-px h-full w-0.5 bg-gray-200", "aria-hidden": "true" })),
                            React.createElement("div", { className: "relative flex items-start space-x-3" },
                                React.createElement("div", { className: "relative" },
                                    React.createElement("span", { className: `h-10 w-10 rounded-full flex items-center justify-center ring-8 ring-white ${getSeverityColor(log.severity)}` }, getCategoryIcon(log.category))),
                                React.createElement("div", { className: "min-w-0 flex-1" },
                                    React.createElement("div", null,
                                        React.createElement("div", { className: "text-sm" },
                                            React.createElement("span", { className: "font-medium text-gray-900" }, log.details.description)),
                                        React.createElement("p", { className: "mt-0.5 text-xs text-gray-500" },
                                            log.resource,
                                            " \u2022 ",
                                            log.action,
                                            " \u2022",
                                            ' ',
                                            new Date(log.timestamp).toLocaleString())),
                                    React.createElement("div", { className: "mt-2 text-xs text-gray-700" },
                                        React.createElement("div", { className: "flex items-center space-x-4" },
                                            React.createElement("span", null,
                                                "IP: ",
                                                log.metadata.ipAddress),
                                            log.metadata.location && (React.createElement("span", null,
                                                "Location: ",
                                                log.metadata.location)),
                                            React.createElement("span", { className: `inline-flex items-center px-2 py-0.5 rounded ${log.status === 'success'
                                                    ? 'bg-green-100 text-green-800'
                                                    : log.status === 'failure'
                                                        ? 'bg-red-100 text-red-800'
                                                        : 'bg-yellow-100 text-yellow-800'}` }, log.status))),
                                    log.details.changes && log.details.changes.length > 0 && (React.createElement("div", { className: "mt-2 text-xs bg-gray-50 p-2 rounded" }, log.details.changes.map((change, i) => (React.createElement("div", { key: i, className: "flex items-center space-x-2" },
                                        React.createElement("span", { className: "font-medium" },
                                            change.field,
                                            ":"),
                                        React.createElement("span", { className: "text-gray-500" }, JSON.stringify(change.oldValue)),
                                        React.createElement("span", null, "\u2192"),
                                        React.createElement("span", { className: "text-gray-900" }, JSON.stringify(change.newValue))))))))))))))))),
            activeTab === 'sessions' && (React.createElement("div", { className: "space-y-4" },
                sessions.map((session) => (React.createElement("div", { key: session.id, className: "border border-gray-200 rounded-lg p-4" },
                    React.createElement("div", { className: "flex items-start justify-between" },
                        React.createElement("div", { className: "flex-1" },
                            React.createElement("div", { className: "flex items-center space-x-2 mb-2" },
                                React.createElement("h4", { className: "text-sm font-medium text-gray-900" }, session.deviceName),
                                React.createElement("span", { className: `inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${session.status === 'active'
                                        ? 'bg-green-100 text-green-800'
                                        : 'bg-gray-100 text-gray-800'}` }, session.status)),
                            React.createElement("div", { className: "grid grid-cols-2 gap-4 text-sm" },
                                React.createElement("div", null,
                                    React.createElement("dt", { className: "text-gray-500" }, "Browser"),
                                    React.createElement("dd", { className: "text-gray-900" }, session.browser)),
                                React.createElement("div", null,
                                    React.createElement("dt", { className: "text-gray-500" }, "Operating System"),
                                    React.createElement("dd", { className: "text-gray-900" }, session.os)),
                                React.createElement("div", null,
                                    React.createElement("dt", { className: "text-gray-500" }, "IP Address"),
                                    React.createElement("dd", { className: "text-gray-900" }, session.ipAddress)),
                                React.createElement("div", null,
                                    React.createElement("dt", { className: "text-gray-500" }, "Location"),
                                    React.createElement("dd", { className: "text-gray-900" }, session.location
                                        ? `${session.location.city}, ${session.location.country}`
                                        : 'Unknown')),
                                React.createElement("div", null,
                                    React.createElement("dt", { className: "text-gray-500" }, "Started"),
                                    React.createElement("dd", { className: "text-gray-900" }, new Date(session.createdAt).toLocaleString())),
                                React.createElement("div", null,
                                    React.createElement("dt", { className: "text-gray-500" }, "Last Activity"),
                                    React.createElement("dd", { className: "text-gray-900" }, new Date(session.lastActivityAt).toLocaleString())))),
                        session.status === 'active' && (React.createElement("button", { onClick: () => terminateSession(session.id), className: "ml-4 text-sm text-red-600 hover:text-red-900 font-medium" }, "Terminate")))))),
                sessions.length === 0 && (React.createElement("div", { className: "text-center py-8 text-sm text-gray-500" }, "No sessions found")))),
            activeTab === 'summary' && summary && (React.createElement("div", { className: "space-y-6" },
                React.createElement("div", { className: "grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4" },
                    React.createElement("div", { className: "bg-white overflow-hidden shadow rounded-lg" },
                        React.createElement("div", { className: "p-5" },
                            React.createElement("div", { className: "flex items-center" },
                                React.createElement("div", { className: "flex-shrink-0" },
                                    React.createElement("svg", { className: "h-6 w-6 text-gray-400", fill: "none", stroke: "currentColor", viewBox: "0 0 24 24" },
                                        React.createElement("path", { strokeLinecap: "round", strokeLinejoin: "round", strokeWidth: 2, d: "M11 16l-4-4m0 0l4-4m-4 4h14m-5 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h7a3 3 0 013 3v1" }))),
                                React.createElement("div", { className: "ml-5 w-0 flex-1" },
                                    React.createElement("dl", null,
                                        React.createElement("dt", { className: "text-sm font-medium text-gray-500 truncate" }, "Total Logins"),
                                        React.createElement("dd", { className: "text-lg font-medium text-gray-900" }, summary.loginCount)))))),
                    React.createElement("div", { className: "bg-white overflow-hidden shadow rounded-lg" },
                        React.createElement("div", { className: "p-5" },
                            React.createElement("div", { className: "flex items-center" },
                                React.createElement("div", { className: "flex-shrink-0" },
                                    React.createElement("svg", { className: "h-6 w-6 text-gray-400", fill: "none", stroke: "currentColor", viewBox: "0 0 24 24" },
                                        React.createElement("path", { strokeLinecap: "round", strokeLinejoin: "round", strokeWidth: 2, d: "M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" }))),
                                React.createElement("div", { className: "ml-5 w-0 flex-1" },
                                    React.createElement("dl", null,
                                        React.createElement("dt", { className: "text-sm font-medium text-gray-500 truncate" }, "Avg Session"),
                                        React.createElement("dd", { className: "text-lg font-medium text-gray-900" },
                                            Math.round(summary.averageSessionDuration / 60),
                                            "m")))))),
                    React.createElement("div", { className: "bg-white overflow-hidden shadow rounded-lg" },
                        React.createElement("div", { className: "p-5" },
                            React.createElement("div", { className: "flex items-center" },
                                React.createElement("div", { className: "flex-shrink-0" },
                                    React.createElement("svg", { className: "h-6 w-6 text-gray-400", fill: "none", stroke: "currentColor", viewBox: "0 0 24 24" },
                                        React.createElement("path", { strokeLinecap: "round", strokeLinejoin: "round", strokeWidth: 2, d: "M13 10V3L4 14h7v7l9-11h-7z" }))),
                                React.createElement("div", { className: "ml-5 w-0 flex-1" },
                                    React.createElement("dl", null,
                                        React.createElement("dt", { className: "text-sm font-medium text-gray-500 truncate" }, "Total Actions"),
                                        React.createElement("dd", { className: "text-lg font-medium text-gray-900" }, summary.totalActions)))))),
                    React.createElement("div", { className: "bg-white overflow-hidden shadow rounded-lg" },
                        React.createElement("div", { className: "p-5" },
                            React.createElement("div", { className: "flex items-center" },
                                React.createElement("div", { className: "flex-shrink-0" },
                                    React.createElement("svg", { className: "h-6 w-6 text-gray-400", fill: "none", stroke: "currentColor", viewBox: "0 0 24 24" },
                                        React.createElement("path", { strokeLinecap: "round", strokeLinejoin: "round", strokeWidth: 2, d: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" }))),
                                React.createElement("div", { className: "ml-5 w-0 flex-1" },
                                    React.createElement("dl", null,
                                        React.createElement("dt", { className: "text-sm font-medium text-gray-500 truncate" }, "Security Events"),
                                        React.createElement("dd", { className: "text-lg font-medium text-gray-900" }, summary.securityEvents))))))),
                React.createElement("div", { className: "grid grid-cols-1 gap-6 lg:grid-cols-2" },
                    React.createElement("div", null,
                        React.createElement("h4", { className: "text-sm font-medium text-gray-900 mb-3" }, "Top Actions"),
                        React.createElement("div", { className: "space-y-2" }, summary.topActions.map((item, idx) => (React.createElement("div", { key: idx, className: "flex items-center justify-between p-3 bg-gray-50 rounded" },
                            React.createElement("span", { className: "text-sm text-gray-900" }, item.action),
                            React.createElement("span", { className: "text-sm font-medium text-gray-600" }, item.count)))))),
                    React.createElement("div", null,
                        React.createElement("h4", { className: "text-sm font-medium text-gray-900 mb-3" }, "Top Resources"),
                        React.createElement("div", { className: "space-y-2" }, summary.topResources.map((item, idx) => (React.createElement("div", { key: idx, className: "flex items-center justify-between p-3 bg-gray-50 rounded" },
                            React.createElement("span", { className: "text-sm text-gray-900" }, item.resource),
                            React.createElement("span", { className: "text-sm font-medium text-gray-600" }, item.count))))))))))));
};
export default UserActivity;
//# sourceMappingURL=UserActivity.js.map