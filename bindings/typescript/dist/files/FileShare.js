import React, { useState, useEffect } from 'react';
export const FileShareComponent = ({ file, tenantId, userId, onClose, onShareUpdate, className = '', }) => {
    const [share, setShare] = useState(null);
    const [loading, setLoading] = useState(true);
    const [activeTab, setActiveTab] = useState('users');
    const [recipientEmail, setRecipientEmail] = useState('');
    const [recipientPermission, setRecipientPermission] = useState('view');
    const [linkPassword, setLinkPassword] = useState('');
    const [linkExpiration, setLinkExpiration] = useState('');
    const [linkMaxDownloads, setLinkMaxDownloads] = useState('');
    const [linkPermission, setLinkPermission] = useState('view');
    const [showLinkSettings, setShowLinkSettings] = useState(false);
    const [copiedLinkId, setCopiedLinkId] = useState(null);
    useEffect(() => {
        loadShare();
    }, [file.id]);
    const loadShare = async () => {
        setLoading(true);
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/${file.id}/share`, {
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                },
            });
            if (response.ok) {
                const data = await response.json();
                setShare(data);
            }
            else if (response.status === 404) {
                setShare({
                    id: '',
                    fileId: file.id,
                    sharedWith: [],
                    links: [],
                    createdAt: new Date(),
                    updatedAt: new Date(),
                });
            }
        }
        catch (error) {
            console.error('Failed to load share:', error);
        }
        finally {
            setLoading(false);
        }
    };
    const addRecipient = async () => {
        if (!recipientEmail.trim())
            return;
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/${file.id}/share/recipients`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    email: recipientEmail,
                    permission: recipientPermission,
                }),
            });
            if (!response.ok) {
                throw new Error('Failed to add recipient');
            }
            const updatedShare = await response.json();
            setShare(updatedShare);
            setRecipientEmail('');
            onShareUpdate?.(updatedShare);
        }
        catch (error) {
            console.error('Failed to add recipient:', error);
            alert(error instanceof Error ? error.message : 'Failed to add recipient');
        }
    };
    const removeRecipient = async (recipientId) => {
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/${file.id}/share/recipients/${recipientId}`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                },
            });
            if (!response.ok) {
                throw new Error('Failed to remove recipient');
            }
            const updatedShare = await response.json();
            setShare(updatedShare);
            onShareUpdate?.(updatedShare);
        }
        catch (error) {
            console.error('Failed to remove recipient:', error);
            alert(error instanceof Error ? error.message : 'Failed to remove recipient');
        }
    };
    const updateRecipientPermission = async (recipientId, permission) => {
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/${file.id}/share/recipients/${recipientId}`, {
                method: 'PATCH',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ permission }),
            });
            if (!response.ok) {
                throw new Error('Failed to update permission');
            }
            const updatedShare = await response.json();
            setShare(updatedShare);
            onShareUpdate?.(updatedShare);
        }
        catch (error) {
            console.error('Failed to update permission:', error);
            alert(error instanceof Error ? error.message : 'Failed to update permission');
        }
    };
    const createShareLink = async () => {
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/${file.id}/share/links`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    permission: linkPermission,
                    password: linkPassword || undefined,
                    expiresAt: linkExpiration ? new Date(linkExpiration).toISOString() : undefined,
                    maxDownloads: linkMaxDownloads ? parseInt(linkMaxDownloads) : undefined,
                }),
            });
            if (!response.ok) {
                throw new Error('Failed to create share link');
            }
            const updatedShare = await response.json();
            setShare(updatedShare);
            setShowLinkSettings(false);
            setLinkPassword('');
            setLinkExpiration('');
            setLinkMaxDownloads('');
            onShareUpdate?.(updatedShare);
        }
        catch (error) {
            console.error('Failed to create share link:', error);
            alert(error instanceof Error ? error.message : 'Failed to create share link');
        }
    };
    const deleteShareLink = async (linkId) => {
        try {
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/${file.id}/share/links/${linkId}`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                },
            });
            if (!response.ok) {
                throw new Error('Failed to delete share link');
            }
            const updatedShare = await response.json();
            setShare(updatedShare);
            onShareUpdate?.(updatedShare);
        }
        catch (error) {
            console.error('Failed to delete share link:', error);
            alert(error instanceof Error ? error.message : 'Failed to delete share link');
        }
    };
    const copyLink = async (link) => {
        try {
            await navigator.clipboard.writeText(link.url);
            setCopiedLinkId(link.id);
            setTimeout(() => setCopiedLinkId(null), 2000);
        }
        catch (error) {
            console.error('Failed to copy link:', error);
        }
    };
    const getPermissionLabel = (permission) => {
        const labels = {
            view: 'Can view',
            comment: 'Can comment',
            edit: 'Can edit',
            admin: 'Full access',
        };
        return labels[permission];
    };
    const getPermissionIcon = (permission) => {
        const icons = {
            view: '👁️',
            comment: '💬',
            edit: '✏️',
            admin: '⚙️',
        };
        return icons[permission];
    };
    if (loading) {
        return (React.createElement("div", { className: `file-share-modal ${className}` },
            React.createElement("div", { className: "share-overlay", onClick: onClose }),
            React.createElement("div", { className: "share-container" },
                React.createElement("div", { className: "loading" }, "Loading share settings..."))));
    }
    return (React.createElement("div", { className: `file-share-modal ${className}` },
        React.createElement("div", { className: "share-overlay", onClick: onClose }),
        React.createElement("div", { className: "share-container" },
            React.createElement("div", { className: "share-header" },
                React.createElement("h2", null,
                    "Share \"",
                    file.name,
                    "\""),
                React.createElement("button", { onClick: onClose, className: "btn-close" }, "\u2715")),
            React.createElement("div", { className: "share-tabs" },
                React.createElement("button", { className: `tab ${activeTab === 'users' ? 'active' : ''}`, onClick: () => setActiveTab('users') },
                    "People (",
                    share?.sharedWith.length || 0,
                    ")"),
                React.createElement("button", { className: `tab ${activeTab === 'links' ? 'active' : ''}`, onClick: () => setActiveTab('links') },
                    "Links (",
                    share?.links.length || 0,
                    ")")),
            React.createElement("div", { className: "share-content" },
                activeTab === 'users' && (React.createElement("div", { className: "share-users" },
                    React.createElement("div", { className: "add-recipient" },
                        React.createElement("input", { type: "email", placeholder: "Enter email address", value: recipientEmail, onChange: (e) => setRecipientEmail(e.target.value), onKeyPress: (e) => e.key === 'Enter' && addRecipient(), className: "form-input" }),
                        React.createElement("select", { value: recipientPermission, onChange: (e) => setRecipientPermission(e.target.value), className: "form-select" },
                            React.createElement("option", { value: "view" }, "Can view"),
                            React.createElement("option", { value: "comment" }, "Can comment"),
                            React.createElement("option", { value: "edit" }, "Can edit"),
                            React.createElement("option", { value: "admin" }, "Full access")),
                        React.createElement("button", { onClick: addRecipient, className: "btn btn-primary" }, "Add")),
                    React.createElement("div", { className: "recipients-list" }, share?.sharedWith.length === 0 ? (React.createElement("div", { className: "empty-state" }, "No one has access yet. Add people above to share this file.")) : (share?.sharedWith.map(recipient => (React.createElement("div", { key: recipient.id, className: "recipient-item" },
                        React.createElement("div", { className: "recipient-info" },
                            React.createElement("div", { className: "recipient-avatar" }, recipient.name.charAt(0).toUpperCase()),
                            React.createElement("div", { className: "recipient-details" },
                                React.createElement("div", { className: "recipient-name" }, recipient.name),
                                React.createElement("div", { className: "recipient-email" }, recipient.email))),
                        React.createElement("div", { className: "recipient-actions" },
                            React.createElement("select", { value: recipient.permission, onChange: (e) => updateRecipientPermission(recipient.id, e.target.value), className: "form-select form-select-sm" },
                                React.createElement("option", { value: "view" }, "Can view"),
                                React.createElement("option", { value: "comment" }, "Can comment"),
                                React.createElement("option", { value: "edit" }, "Can edit"),
                                React.createElement("option", { value: "admin" }, "Full access")),
                            React.createElement("button", { onClick: () => removeRecipient(recipient.id), className: "btn btn-sm btn-danger" }, "Remove"))))))))),
                activeTab === 'links' && (React.createElement("div", { className: "share-links" },
                    !showLinkSettings && (React.createElement("button", { onClick: () => setShowLinkSettings(true), className: "btn btn-primary btn-block" }, "Create Share Link")),
                    showLinkSettings && (React.createElement("div", { className: "link-settings" },
                        React.createElement("h3", null, "Link Settings"),
                        React.createElement("div", { className: "form-group" },
                            React.createElement("label", null, "Permission Level"),
                            React.createElement("select", { value: linkPermission, onChange: (e) => setLinkPermission(e.target.value), className: "form-select" },
                                React.createElement("option", { value: "view" }, "Can view"),
                                React.createElement("option", { value: "comment" }, "Can comment"),
                                React.createElement("option", { value: "edit" }, "Can edit"))),
                        React.createElement("div", { className: "form-group" },
                            React.createElement("label", null, "Password (optional)"),
                            React.createElement("input", { type: "password", placeholder: "Set password for link", value: linkPassword, onChange: (e) => setLinkPassword(e.target.value), className: "form-input" })),
                        React.createElement("div", { className: "form-group" },
                            React.createElement("label", null, "Expiration Date (optional)"),
                            React.createElement("input", { type: "datetime-local", value: linkExpiration, onChange: (e) => setLinkExpiration(e.target.value), className: "form-input" })),
                        React.createElement("div", { className: "form-group" },
                            React.createElement("label", null, "Maximum Downloads (optional)"),
                            React.createElement("input", { type: "number", min: "1", placeholder: "Unlimited", value: linkMaxDownloads, onChange: (e) => setLinkMaxDownloads(e.target.value), className: "form-input" })),
                        React.createElement("div", { className: "form-actions" },
                            React.createElement("button", { onClick: createShareLink, className: "btn btn-primary" }, "Create Link"),
                            React.createElement("button", { onClick: () => setShowLinkSettings(false), className: "btn" }, "Cancel")))),
                    React.createElement("div", { className: "links-list" }, share?.links.length === 0 ? (React.createElement("div", { className: "empty-state" }, "No share links created yet. Create one above to share this file.")) : (share?.links.map(link => (React.createElement("div", { key: link.id, className: `link-item ${!link.isActive ? 'inactive' : ''}` },
                        React.createElement("div", { className: "link-header" },
                            React.createElement("div", { className: "link-permission" },
                                React.createElement("span", { className: "permission-icon" }, getPermissionIcon(link.permission)),
                                React.createElement("span", { className: "permission-label" }, getPermissionLabel(link.permission))),
                            React.createElement("div", { className: "link-status" }, link.isActive ? (React.createElement("span", { className: "status-badge status-active" }, "Active")) : (React.createElement("span", { className: "status-badge status-inactive" }, "Inactive")))),
                        React.createElement("div", { className: "link-url" },
                            React.createElement("input", { type: "text", value: link.url, readOnly: true, className: "form-input" }),
                            React.createElement("button", { onClick: () => copyLink(link), className: "btn btn-sm" }, copiedLinkId === link.id ? 'Copied!' : 'Copy')),
                        React.createElement("div", { className: "link-details" },
                            link.password && (React.createElement("div", { className: "link-detail" },
                                React.createElement("span", { className: "detail-icon" }, "\uD83D\uDD12"),
                                React.createElement("span", null, "Password protected"))),
                            link.expiresAt && (React.createElement("div", { className: "link-detail" },
                                React.createElement("span", { className: "detail-icon" }, "\u23F0"),
                                React.createElement("span", null,
                                    "Expires ",
                                    formatDate(link.expiresAt)))),
                            link.maxDownloads && (React.createElement("div", { className: "link-detail" },
                                React.createElement("span", { className: "detail-icon" }, "\uD83D\uDCE5"),
                                React.createElement("span", null,
                                    link.downloadCount,
                                    "/",
                                    link.maxDownloads,
                                    " downloads"))),
                            React.createElement("div", { className: "link-detail" },
                                React.createElement("span", { className: "detail-icon" }, "\uD83D\uDCC5"),
                                React.createElement("span", null,
                                    "Created ",
                                    formatDate(link.createdAt)))),
                        React.createElement("div", { className: "link-actions" },
                            React.createElement("button", { onClick: () => deleteShareLink(link.id), className: "btn btn-sm btn-danger" }, "Delete Link")))))))))),
            React.createElement("div", { className: "share-footer" },
                React.createElement("div", { className: "share-info" }, file.permissions.canShare ? (React.createElement("span", null, "You can share this file with others")) : (React.createElement("span", { className: "warning" }, "You have limited sharing permissions")))))));
};
function formatDate(date) {
    const d = new Date(date);
    const now = new Date();
    const diffMs = now.getTime() - d.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
    if (diffDays === 0)
        return 'today';
    if (diffDays === 1)
        return 'yesterday';
    if (diffDays < 7)
        return `${diffDays} days ago`;
    if (diffDays < 30)
        return `${Math.floor(diffDays / 7)} weeks ago`;
    return d.toLocaleDateString();
}
export default FileShareComponent;
//# sourceMappingURL=FileShare.js.map