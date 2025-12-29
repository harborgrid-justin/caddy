import React, { useState, useEffect } from 'react';
export const APIWebhooks = ({ onWebhookCreate, onWebhookUpdate, onWebhookDelete, onWebhookTest, }) => {
    const [webhooks, setWebhooks] = useState([]);
    const [selectedWebhook, setSelectedWebhook] = useState(null);
    const [deliveries, setDeliveries] = useState([]);
    const [isCreating, setIsCreating] = useState(false);
    const [isEditing, setIsEditing] = useState(false);
    const [isTesting, setIsTesting] = useState(false);
    useEffect(() => {
        loadWebhooks();
    }, []);
    useEffect(() => {
        if (selectedWebhook) {
            loadDeliveries(selectedWebhook.id);
        }
    }, [selectedWebhook]);
    const loadWebhooks = async () => {
        try {
            const mockWebhooks = [
                {
                    id: '1',
                    name: 'Order Created Webhook',
                    url: 'https://example.com/webhooks/orders',
                    events: ['order.created', 'order.updated'],
                    secret: 'whsec_**********************',
                    active: true,
                    retryPolicy: {
                        maxRetries: 3,
                        backoffMultiplier: 2,
                        maxBackoff: 3600,
                        retryOn: [500, 502, 503, 504],
                    },
                    headers: {
                        'X-Custom-Header': 'value',
                    },
                    createdAt: Date.now() - 86400000 * 15,
                    updatedAt: Date.now() - 3600000,
                },
                {
                    id: '2',
                    name: 'User Events Webhook',
                    url: 'https://example.com/webhooks/users',
                    events: ['user.created', 'user.updated', 'user.deleted'],
                    secret: 'whsec_**********************',
                    active: true,
                    retryPolicy: {
                        maxRetries: 3,
                        backoffMultiplier: 2,
                        maxBackoff: 3600,
                        retryOn: [500, 502, 503, 504],
                    },
                    headers: {},
                    createdAt: Date.now() - 86400000 * 30,
                    updatedAt: Date.now() - 86400000,
                },
            ];
            setWebhooks(mockWebhooks);
        }
        catch (error) {
            console.error('Failed to load webhooks:', error);
        }
    };
    const loadDeliveries = async (webhookId) => {
        try {
            const mockDeliveries = [
                {
                    id: '1',
                    webhookId,
                    event: 'order.created',
                    payload: { orderId: '12345', total: 99.99 },
                    status: 'success',
                    attempts: 1,
                    lastAttemptAt: Date.now() - 300000,
                    response: {
                        statusCode: 200,
                        headers: { 'content-type': 'application/json' },
                        body: '{"success":true}',
                        duration: 234,
                    },
                    createdAt: Date.now() - 300000,
                },
                {
                    id: '2',
                    webhookId,
                    event: 'order.updated',
                    payload: { orderId: '12345', status: 'shipped' },
                    status: 'failed',
                    attempts: 3,
                    lastAttemptAt: Date.now() - 600000,
                    response: {
                        statusCode: 500,
                        headers: { 'content-type': 'text/plain' },
                        body: 'Internal Server Error',
                        duration: 5234,
                    },
                    createdAt: Date.now() - 650000,
                },
            ];
            setDeliveries(mockDeliveries);
        }
        catch (error) {
            console.error('Failed to load deliveries:', error);
        }
    };
    const handleCreate = async (webhook) => {
        try {
            if (onWebhookCreate) {
                await onWebhookCreate(webhook);
            }
            const newWebhook = {
                id: Date.now().toString(),
                ...webhook,
                secret: `whsec_${generateRandomString(32)}`,
                createdAt: Date.now(),
                updatedAt: Date.now(),
            };
            setWebhooks([...webhooks, newWebhook]);
            setIsCreating(false);
        }
        catch (error) {
            console.error('Failed to create webhook:', error);
        }
    };
    const handleUpdate = async (id, webhook) => {
        try {
            if (onWebhookUpdate) {
                await onWebhookUpdate(id, webhook);
            }
            setWebhooks(webhooks.map((w) => w.id === id ? { ...w, ...webhook, updatedAt: Date.now() } : w));
            setIsEditing(false);
            setSelectedWebhook(null);
        }
        catch (error) {
            console.error('Failed to update webhook:', error);
        }
    };
    const handleDelete = async (id) => {
        if (!confirm('Are you sure you want to delete this webhook?'))
            return;
        try {
            if (onWebhookDelete) {
                await onWebhookDelete(id);
            }
            setWebhooks(webhooks.filter((w) => w.id !== id));
            if (selectedWebhook?.id === id) {
                setSelectedWebhook(null);
            }
        }
        catch (error) {
            console.error('Failed to delete webhook:', error);
        }
    };
    const handleTest = async (id) => {
        setIsTesting(true);
        try {
            if (onWebhookTest) {
                await onWebhookTest(id);
            }
            else {
                await new Promise((resolve) => setTimeout(resolve, 1000));
            }
            alert('Test webhook sent successfully!');
        }
        catch (error) {
            console.error('Failed to test webhook:', error);
            alert('Failed to send test webhook');
        }
        finally {
            setIsTesting(false);
        }
    };
    const toggleActive = async (id) => {
        const webhook = webhooks.find((w) => w.id === id);
        if (!webhook)
            return;
        await handleUpdate(id, { active: !webhook.active });
    };
    return (React.createElement("div", { className: "min-h-screen bg-gray-50 dark:bg-gray-900" },
        React.createElement("div", { className: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8" },
            React.createElement("div", { className: "flex items-center justify-between mb-6" },
                React.createElement("h1", { className: "text-2xl font-bold text-gray-900 dark:text-white" }, "Webhooks"),
                React.createElement("button", { onClick: () => setIsCreating(true), className: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors" }, "+ Create Webhook")),
            React.createElement("div", { className: "grid grid-cols-1 lg:grid-cols-3 gap-6" },
                React.createElement("div", { className: "lg:col-span-2 space-y-4" }, !isCreating && !isEditing ? (React.createElement(React.Fragment, null,
                    webhooks.map((webhook) => (React.createElement("div", { key: webhook.id, onClick: () => setSelectedWebhook(webhook), className: `bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6 cursor-pointer transition-all ${selectedWebhook?.id === webhook.id ? 'ring-2 ring-blue-500' : 'hover:border-blue-300'}` },
                        React.createElement("div", { className: "flex items-start justify-between mb-4" },
                            React.createElement("div", { className: "flex-1" },
                                React.createElement("div", { className: "flex items-center space-x-3 mb-2" },
                                    React.createElement("h3", { className: "text-lg font-semibold text-gray-900 dark:text-white" }, webhook.name),
                                    React.createElement("span", { className: `px-2 py-1 rounded text-xs font-semibold ${webhook.active
                                            ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
                                            : 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200'}` }, webhook.active ? 'Active' : 'Inactive')),
                                React.createElement("code", { className: "text-sm text-gray-600 dark:text-gray-400 block mb-3" }, webhook.url),
                                React.createElement("div", { className: "flex flex-wrap gap-2" }, webhook.events.map((event) => (React.createElement("span", { key: event, className: "px-2 py-1 rounded text-xs bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300" }, event))))),
                            React.createElement("div", { className: "flex flex-col space-y-2 ml-4" },
                                React.createElement("button", { onClick: (e) => {
                                        e.stopPropagation();
                                        handleTest(webhook.id);
                                    }, disabled: isTesting, className: "text-sm text-blue-600 hover:text-blue-800 dark:text-blue-400 disabled:opacity-50" }, "Test"),
                                React.createElement("button", { onClick: (e) => {
                                        e.stopPropagation();
                                        setSelectedWebhook(webhook);
                                        setIsEditing(true);
                                    }, className: "text-sm text-gray-600 hover:text-gray-800 dark:text-gray-400" }, "Edit"),
                                React.createElement("button", { onClick: (e) => {
                                        e.stopPropagation();
                                        handleDelete(webhook.id);
                                    }, className: "text-sm text-red-600 hover:text-red-800 dark:text-red-400" }, "Delete"))),
                        React.createElement("div", { className: "flex items-center justify-between text-sm text-gray-500 dark:text-gray-400 mt-4 pt-4 border-t border-gray-200 dark:border-gray-700" },
                            React.createElement("div", null,
                                "Created ",
                                formatDate(webhook.createdAt)),
                            React.createElement("div", null,
                                "Updated ",
                                formatTimeAgo(webhook.updatedAt)))))),
                    webhooks.length === 0 && (React.createElement("div", { className: "text-center py-12 text-gray-500 dark:text-gray-400" }, "No webhooks configured")))) : (React.createElement(WebhookEditor, { webhook: selectedWebhook, onSave: (webhook) => {
                        if (isCreating) {
                            handleCreate(webhook);
                        }
                        else if (selectedWebhook) {
                            handleUpdate(selectedWebhook.id, webhook);
                        }
                    }, onCancel: () => {
                        setIsCreating(false);
                        setIsEditing(false);
                        setSelectedWebhook(null);
                    } }))),
                React.createElement("div", { className: "lg:col-span-1" },
                    React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 sticky top-4" },
                        React.createElement("div", { className: "px-6 py-4 border-b border-gray-200 dark:border-gray-700" },
                            React.createElement("h2", { className: "text-lg font-semibold text-gray-900 dark:text-white" }, "Recent Deliveries")),
                        React.createElement("div", { className: "p-6 max-h-[600px] overflow-y-auto" }, selectedWebhook ? (React.createElement("div", { className: "space-y-4" },
                            deliveries.map((delivery) => (React.createElement("div", { key: delivery.id, className: "border border-gray-200 dark:border-gray-700 rounded-lg p-4" },
                                React.createElement("div", { className: "flex items-center justify-between mb-2" },
                                    React.createElement("span", { className: "text-sm font-medium text-gray-900 dark:text-white" }, delivery.event),
                                    React.createElement("span", { className: `px-2 py-1 rounded text-xs font-semibold ${delivery.status === 'success'
                                            ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
                                            : delivery.status === 'failed'
                                                ? 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200'
                                                : 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200'}` }, delivery.status)),
                                delivery.response && (React.createElement("div", { className: "text-sm space-y-1 mb-2" },
                                    React.createElement("div", { className: "text-gray-600 dark:text-gray-400" },
                                        "Status: ",
                                        delivery.response.statusCode),
                                    React.createElement("div", { className: "text-gray-600 dark:text-gray-400" },
                                        "Duration: ",
                                        delivery.response.duration,
                                        "ms"),
                                    React.createElement("div", { className: "text-gray-600 dark:text-gray-400" },
                                        "Attempts: ",
                                        delivery.attempts))),
                                React.createElement("div", { className: "text-xs text-gray-500 dark:text-gray-400" }, formatTimeAgo(delivery.createdAt))))),
                            deliveries.length === 0 && (React.createElement("div", { className: "text-center py-8 text-gray-500 dark:text-gray-400" }, "No deliveries yet")))) : (React.createElement("div", { className: "text-center py-8 text-gray-500 dark:text-gray-400" }, "Select a webhook to view deliveries")))))))));
};
const WebhookEditor = ({ webhook, onSave, onCancel }) => {
    const [formData, setFormData] = useState(webhook || {
        name: '',
        url: '',
        events: [],
        active: true,
        retryPolicy: {
            maxRetries: 3,
            backoffMultiplier: 2,
            maxBackoff: 3600,
            retryOn: [500, 502, 503, 504],
        },
        headers: {},
    });
    const availableEvents = [
        'user.created',
        'user.updated',
        'user.deleted',
        'order.created',
        'order.updated',
        'order.cancelled',
        'payment.success',
        'payment.failed',
    ];
    const handleSubmit = (e) => {
        e.preventDefault();
        onSave(formData);
    };
    const toggleEvent = (event) => {
        const events = formData.events || [];
        if (events.includes(event)) {
            setFormData({ ...formData, events: events.filter((e) => e !== event) });
        }
        else {
            setFormData({ ...formData, events: [...events, event] });
        }
    };
    return (React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6" },
        React.createElement("h2", { className: "text-xl font-semibold text-gray-900 dark:text-white mb-6" }, webhook ? 'Edit Webhook' : 'Create Webhook'),
        React.createElement("form", { onSubmit: handleSubmit, className: "space-y-6" },
            React.createElement("div", null,
                React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2" }, "Name"),
                React.createElement("input", { type: "text", value: formData.name, onChange: (e) => setFormData({ ...formData, name: e.target.value }), className: "w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white", required: true })),
            React.createElement("div", null,
                React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2" }, "Webhook URL"),
                React.createElement("input", { type: "url", value: formData.url, onChange: (e) => setFormData({ ...formData, url: e.target.value }), className: "w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg dark:bg-gray-700 dark:text-white", placeholder: "https://example.com/webhook", required: true })),
            React.createElement("div", null,
                React.createElement("label", { className: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2" }, "Events to Subscribe"),
                React.createElement("div", { className: "space-y-2" }, availableEvents.map((event) => (React.createElement("label", { key: event, className: "flex items-center" },
                    React.createElement("input", { type: "checkbox", checked: formData.events?.includes(event), onChange: () => toggleEvent(event), className: "h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded" }),
                    React.createElement("span", { className: "ml-2 text-sm text-gray-700 dark:text-gray-300" }, event)))))),
            React.createElement("div", { className: "flex items-center" },
                React.createElement("input", { type: "checkbox", id: "active", checked: formData.active, onChange: (e) => setFormData({ ...formData, active: e.target.checked }), className: "h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded" }),
                React.createElement("label", { htmlFor: "active", className: "ml-2 block text-sm text-gray-700 dark:text-gray-300" }, "Active")),
            React.createElement("div", { className: "flex justify-end space-x-4" },
                React.createElement("button", { type: "button", onClick: onCancel, className: "px-6 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700" }, "Cancel"),
                React.createElement("button", { type: "submit", className: "px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700", disabled: !formData.events || formData.events.length === 0 }, "Save")))));
};
function formatDate(timestamp) {
    return new Date(timestamp).toLocaleDateString();
}
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
function generateRandomString(length) {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    let result = '';
    for (let i = 0; i < length; i++) {
        result += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    return result;
}
export default APIWebhooks;
//# sourceMappingURL=APIWebhooks.js.map