import React, { useState, useEffect } from 'react';
export const APIPortal = ({ projectId = 'default', onNavigate, config = {}, }) => {
    const [activeView, setActiveView] = useState('explorer');
    const [stats, setStats] = useState({
        totalEndpoints: 0,
        activeKeys: 0,
        todayRequests: 0,
        avgResponseTime: 0,
        successRate: 0,
        activeWebhooks: 0,
    });
    const [recentActivity, setRecentActivity] = useState([]);
    const [popularEndpoints, setPopularEndpoints] = useState([]);
    const [isLoading, setIsLoading] = useState(true);
    const [searchQuery, setSearchQuery] = useState('');
    useEffect(() => {
        loadDashboardData();
    }, [projectId]);
    const loadDashboardData = async () => {
        setIsLoading(true);
        try {
            await new Promise((resolve) => setTimeout(resolve, 500));
            setStats({
                totalEndpoints: 47,
                activeKeys: 12,
                todayRequests: 15847,
                avgResponseTime: 142,
                successRate: 99.2,
                activeWebhooks: 5,
            });
            setRecentActivity([
                {
                    id: '1',
                    type: 'endpoint_created',
                    message: 'New endpoint created: POST /api/v1/users',
                    timestamp: Date.now() - 300000,
                    user: 'admin@example.com',
                },
                {
                    id: '2',
                    type: 'key_generated',
                    message: 'API key generated for production',
                    timestamp: Date.now() - 600000,
                    user: 'developer@example.com',
                },
                {
                    id: '3',
                    type: 'rate_limit_hit',
                    message: 'Rate limit reached for /api/v1/search',
                    timestamp: Date.now() - 900000,
                    user: 'system',
                },
            ]);
            setPopularEndpoints([
                { path: '/api/v1/users', method: 'GET', requests: 5234, avgTime: 98 },
                { path: '/api/v1/auth/login', method: 'POST', requests: 3421, avgTime: 156 },
                { path: '/api/v1/products', method: 'GET', requests: 2987, avgTime: 203 },
                { path: '/api/v1/orders', method: 'POST', requests: 1876, avgTime: 287 },
                { path: '/api/v1/search', method: 'GET', requests: 1654, avgTime: 421 },
            ]);
        }
        catch (error) {
            console.error('Failed to load dashboard data:', error);
        }
        finally {
            setIsLoading(false);
        }
    };
    const handleNavigate = (view) => {
        setActiveView(view);
        onNavigate?.(view);
    };
    const formatNumber = (num) => {
        if (num >= 1000000)
            return `${(num / 1000000).toFixed(1)}M`;
        if (num >= 1000)
            return `${(num / 1000).toFixed(1)}K`;
        return num.toString();
    };
    const formatDuration = (ms) => {
        if (ms < 1000)
            return `${ms}ms`;
        return `${(ms / 1000).toFixed(2)}s`;
    };
    const getActivityIcon = (type) => {
        const icons = {
            endpoint_created: '➕',
            endpoint_updated: '✏️',
            key_generated: '🔑',
            rate_limit_hit: '⚠️',
            webhook_triggered: '🔔',
            test_passed: '✅',
            test_failed: '❌',
        };
        return icons[type] || '📌';
    };
    const getMethodColor = (method) => {
        const colors = {
            GET: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
            POST: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
            PUT: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200',
            PATCH: 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200',
            DELETE: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
            HEAD: 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200',
            OPTIONS: 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200',
        };
        return colors[method] || 'bg-gray-100 text-gray-800';
    };
    if (isLoading) {
        return (React.createElement("div", { className: "flex items-center justify-center h-screen bg-gray-50 dark:bg-gray-900" },
            React.createElement("div", { className: "text-center" },
                React.createElement("div", { className: "inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600" }),
                React.createElement("p", { className: "mt-4 text-gray-600 dark:text-gray-400" }, "Loading API Portal..."))));
    }
    return (React.createElement("div", { className: "min-h-screen bg-gray-50 dark:bg-gray-900" },
        React.createElement("header", { className: "bg-white dark:bg-gray-800 shadow-sm border-b border-gray-200 dark:border-gray-700" },
            React.createElement("div", { className: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4" },
                React.createElement("div", { className: "flex items-center justify-between" },
                    React.createElement("div", { className: "flex items-center space-x-4" },
                        config.logo && React.createElement("img", { src: config.logo, alt: "Logo", className: "h-8 w-8" }),
                        React.createElement("h1", { className: "text-2xl font-bold text-gray-900 dark:text-white" }, config.title || 'CADDY API Portal')),
                    React.createElement("div", { className: "flex items-center space-x-4" },
                        React.createElement("div", { className: "relative" },
                            React.createElement("input", { type: "text", placeholder: "Search endpoints, docs...", value: searchQuery, onChange: (e) => setSearchQuery(e.target.value), className: "w-64 px-4 py-2 pl-10 border border-gray-300 dark:border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" }),
                            React.createElement("span", { className: "absolute left-3 top-2.5 text-gray-400" }, "\uD83D\uDD0D")),
                        React.createElement("button", { className: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors" }, "Generate API Key"))))),
        React.createElement("nav", { className: "bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700" },
            React.createElement("div", { className: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8" },
                React.createElement("div", { className: "flex space-x-8 overflow-x-auto" }, [
                    { id: 'explorer', label: 'API Explorer', icon: '🔍' },
                    { id: 'documentation', label: 'Documentation', icon: '📚' },
                    { id: 'endpoints', label: 'Endpoints', icon: '🔗' },
                    { id: 'keys', label: 'API Keys', icon: '🔑' },
                    { id: 'rate-limits', label: 'Rate Limits', icon: '⚡' },
                    { id: 'analytics', label: 'Analytics', icon: '📊' },
                    { id: 'webhooks', label: 'Webhooks', icon: '🔔' },
                    { id: 'mocking', label: 'Mock Server', icon: '🎭' },
                    { id: 'versioning', label: 'Versions', icon: '📝' },
                    { id: 'testing', label: 'Testing', icon: '🧪' },
                ].map((item) => (React.createElement("button", { key: item.id, onClick: () => handleNavigate(item.id), className: `flex items-center space-x-2 px-4 py-3 border-b-2 transition-colors whitespace-nowrap ${activeView === item.id
                        ? 'border-blue-600 text-blue-600 dark:text-blue-400'
                        : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'}` },
                    React.createElement("span", null, item.icon),
                    React.createElement("span", { className: "font-medium" }, item.label))))))),
        React.createElement("main", { className: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8" },
            React.createElement("div", { className: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6 gap-6 mb-8" },
                React.createElement(StatCard, { title: "Endpoints", value: stats.totalEndpoints.toString(), icon: "\uD83D\uDD17", color: "blue" }),
                React.createElement(StatCard, { title: "Active Keys", value: stats.activeKeys.toString(), icon: "\uD83D\uDD11", color: "green" }),
                React.createElement(StatCard, { title: "Requests Today", value: formatNumber(stats.todayRequests), icon: "\uD83D\uDCCA", color: "purple" }),
                React.createElement(StatCard, { title: "Avg Response", value: `${stats.avgResponseTime}ms`, icon: "\u26A1", color: "yellow" }),
                React.createElement(StatCard, { title: "Success Rate", value: `${stats.successRate}%`, icon: "\u2705", color: "green" }),
                React.createElement(StatCard, { title: "Webhooks", value: stats.activeWebhooks.toString(), icon: "\uD83D\uDD14", color: "indigo" })),
            React.createElement("div", { className: "grid grid-cols-1 lg:grid-cols-3 gap-8" },
                React.createElement("div", { className: "lg:col-span-2 bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700" },
                    React.createElement("div", { className: "px-6 py-4 border-b border-gray-200 dark:border-gray-700" },
                        React.createElement("h2", { className: "text-lg font-semibold text-gray-900 dark:text-white" }, "Popular Endpoints")),
                    React.createElement("div", { className: "divide-y divide-gray-200 dark:divide-gray-700" }, popularEndpoints.map((endpoint, index) => (React.createElement("div", { key: index, className: "px-6 py-4 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors" },
                        React.createElement("div", { className: "flex items-center justify-between" },
                            React.createElement("div", { className: "flex items-center space-x-3 flex-1" },
                                React.createElement("span", { className: `px-2 py-1 rounded text-xs font-semibold ${getMethodColor(endpoint.method)}` }, endpoint.method),
                                React.createElement("code", { className: "text-sm text-gray-900 dark:text-gray-100 font-mono" }, endpoint.path)),
                            React.createElement("div", { className: "flex items-center space-x-6 text-sm" },
                                React.createElement("div", { className: "text-right" },
                                    React.createElement("div", { className: "text-gray-900 dark:text-white font-medium" }, formatNumber(endpoint.requests)),
                                    React.createElement("div", { className: "text-gray-500 dark:text-gray-400 text-xs" }, "requests")),
                                React.createElement("div", { className: "text-right" },
                                    React.createElement("div", { className: "text-gray-900 dark:text-white font-medium" },
                                        endpoint.avgTime,
                                        "ms"),
                                    React.createElement("div", { className: "text-gray-500 dark:text-gray-400 text-xs" }, "avg time"))))))))),
                React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700" },
                    React.createElement("div", { className: "px-6 py-4 border-b border-gray-200 dark:border-gray-700" },
                        React.createElement("h2", { className: "text-lg font-semibold text-gray-900 dark:text-white" }, "Recent Activity")),
                    React.createElement("div", { className: "divide-y divide-gray-200 dark:divide-gray-700" }, recentActivity.map((activity) => (React.createElement("div", { key: activity.id, className: "px-6 py-4" },
                        React.createElement("div", { className: "flex items-start space-x-3" },
                            React.createElement("span", { className: "text-2xl" }, getActivityIcon(activity.type)),
                            React.createElement("div", { className: "flex-1 min-w-0" },
                                React.createElement("p", { className: "text-sm text-gray-900 dark:text-gray-100" }, activity.message),
                                React.createElement("div", { className: "mt-1 flex items-center space-x-2 text-xs text-gray-500 dark:text-gray-400" },
                                    React.createElement("span", null, activity.user),
                                    React.createElement("span", null, "\u2022"),
                                    React.createElement("span", null, formatTimeAgo(activity.timestamp))))))))))),
            config.showQuickStart !== false && (React.createElement("div", { className: "mt-8 bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 rounded-lg border border-blue-200 dark:border-blue-800 p-6" },
                React.createElement("h2", { className: "text-xl font-semibold text-gray-900 dark:text-white mb-4" }, "Quick Start Guide"),
                React.createElement("div", { className: "grid grid-cols-1 md:grid-cols-4 gap-4" },
                    React.createElement(QuickStartCard, { step: 1, title: "Explore APIs", description: "Browse and test endpoints", action: "Open Explorer", onClick: () => handleNavigate('explorer') }),
                    React.createElement(QuickStartCard, { step: 2, title: "Generate Key", description: "Create an API key", action: "Create Key", onClick: () => handleNavigate('keys') }),
                    React.createElement(QuickStartCard, { step: 3, title: "View Docs", description: "Read API documentation", action: "View Docs", onClick: () => handleNavigate('documentation') }),
                    React.createElement(QuickStartCard, { step: 4, title: "Test APIs", description: "Run automated tests", action: "Start Testing", onClick: () => handleNavigate('testing') })))))));
};
const StatCard = ({ title, value, icon, color }) => {
    const colors = {
        blue: 'bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400',
        green: 'bg-green-50 dark:bg-green-900/20 text-green-600 dark:text-green-400',
        purple: 'bg-purple-50 dark:bg-purple-900/20 text-purple-600 dark:text-purple-400',
        yellow: 'bg-yellow-50 dark:bg-yellow-900/20 text-yellow-600 dark:text-yellow-400',
        indigo: 'bg-indigo-50 dark:bg-indigo-900/20 text-indigo-600 dark:text-indigo-400',
    };
    return (React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-4" },
        React.createElement("div", { className: "flex items-center justify-between" },
            React.createElement("div", null,
                React.createElement("p", { className: "text-sm text-gray-600 dark:text-gray-400" }, title),
                React.createElement("p", { className: "mt-1 text-2xl font-semibold text-gray-900 dark:text-white" }, value)),
            React.createElement("div", { className: `p-3 rounded-lg ${colors[color] || colors.blue}` },
                React.createElement("span", { className: "text-2xl" }, icon)))));
};
const QuickStartCard = ({ step, title, description, action, onClick, }) => {
    return (React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700" },
        React.createElement("div", { className: "flex items-center space-x-3 mb-2" },
            React.createElement("div", { className: "flex items-center justify-center w-6 h-6 rounded-full bg-blue-600 text-white text-xs font-bold" }, step),
            React.createElement("h3", { className: "font-semibold text-gray-900 dark:text-white" }, title)),
        React.createElement("p", { className: "text-sm text-gray-600 dark:text-gray-400 mb-3" }, description),
        React.createElement("button", { onClick: onClick, className: "w-full px-3 py-2 text-sm bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 rounded hover:bg-blue-100 dark:hover:bg-blue-900/30 transition-colors" }, action)));
};
function formatTimeAgo(timestamp) {
    const seconds = Math.floor((Date.now() - timestamp) / 1000);
    if (seconds < 60)
        return 'just now';
    if (seconds < 3600)
        return `${Math.floor(seconds / 60)}m ago`;
    if (seconds < 86400)
        return `${Math.floor(seconds / 3600)}h ago`;
    return `${Math.floor(seconds / 86400)}d ago`;
}
export default APIPortal;
//# sourceMappingURL=APIPortal.js.map