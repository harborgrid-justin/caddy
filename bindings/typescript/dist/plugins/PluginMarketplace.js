import React, { useState, useEffect } from 'react';
import { useMarketplace } from './usePlugin';
import { SortBy, } from './types';
export const PluginMarketplace = ({ className = '', onClose, onPluginInstalled, }) => {
    const { results, loading, error, search, install } = useMarketplace();
    const [searchQuery, setSearchQuery] = useState('');
    const [selectedCategory, setSelectedCategory] = useState('');
    const [sortBy, setSortBy] = useState(SortBy.Relevance);
    const [verifiedOnly, setVerifiedOnly] = useState(false);
    const [minRating, setMinRating] = useState(undefined);
    const [currentPage, setCurrentPage] = useState(0);
    const [categories, setCategories] = useState([]);
    const perPage = 12;
    useEffect(() => {
        loadCategories();
    }, []);
    useEffect(() => {
        performSearch();
    }, [searchQuery, selectedCategory, sortBy, verifiedOnly, minRating, currentPage]);
    const loadCategories = async () => {
        try {
            const manager = window.__caddyPluginManager;
            const mockCategories = [
                { id: 'modeling', name: 'Modeling', description: 'Tools for 3D modeling', pluginCount: 15 },
                { id: 'rendering', name: 'Rendering', description: 'Rendering and visualization', pluginCount: 8 },
                { id: 'import-export', name: 'Import/Export', description: 'File format converters', pluginCount: 12 },
                { id: 'automation', name: 'Automation', description: 'Workflow automation', pluginCount: 6 },
                { id: 'analysis', name: 'Analysis', description: 'Engineering analysis tools', pluginCount: 10 },
            ];
            setCategories(mockCategories);
        }
        catch (err) {
            console.error('Failed to load categories:', err);
        }
    };
    const performSearch = async () => {
        const filters = {
            query: searchQuery || undefined,
            category: selectedCategory || undefined,
            sortBy,
            verifiedOnly,
            minRating,
            page: currentPage,
            perPage,
        };
        await search(filters);
    };
    const handleInstall = async (plugin) => {
        try {
            await install(plugin.id);
            onPluginInstalled?.(plugin.id);
            alert(`Successfully installed ${plugin.name}!`);
        }
        catch (err) {
            alert(`Failed to install plugin: ${err.message}`);
        }
    };
    const handlePageChange = (newPage) => {
        setCurrentPage(newPage);
    };
    return (React.createElement("div", { className: `plugin-marketplace ${className}` },
        React.createElement("div", { className: "marketplace-header" },
            React.createElement("h1", null, "Plugin Marketplace"),
            onClose && (React.createElement("button", { className: "btn-close", onClick: onClose }, "Close"))),
        React.createElement("div", { className: "marketplace-layout" },
            React.createElement("aside", { className: "marketplace-sidebar" },
                React.createElement("div", { className: "sidebar-section" },
                    React.createElement("h3", null, "Categories"),
                    React.createElement("ul", { className: "category-list" },
                        React.createElement("li", null,
                            React.createElement("button", { className: selectedCategory === '' ? 'active' : '', onClick: () => setSelectedCategory('') }, "All Categories")),
                        categories.map((category) => (React.createElement("li", { key: category.id },
                            React.createElement("button", { className: selectedCategory === category.id ? 'active' : '', onClick: () => setSelectedCategory(category.id) },
                                category.name,
                                React.createElement("span", { className: "count" },
                                    "(",
                                    category.pluginCount,
                                    ")"))))))),
                React.createElement("div", { className: "sidebar-section" },
                    React.createElement("h3", null, "Filters"),
                    React.createElement("label", { className: "filter-option" },
                        React.createElement("input", { type: "checkbox", checked: verifiedOnly, onChange: (e) => setVerifiedOnly(e.target.checked) }),
                        React.createElement("span", null, "Verified plugins only")),
                    React.createElement("div", { className: "filter-option" },
                        React.createElement("label", null, "Minimum rating:"),
                        React.createElement("select", { value: minRating || '', onChange: (e) => setMinRating(e.target.value ? Number(e.target.value) : undefined) },
                            React.createElement("option", { value: "" }, "Any"),
                            React.createElement("option", { value: "4" }, "4+ Stars"),
                            React.createElement("option", { value: "3" }, "3+ Stars"),
                            React.createElement("option", { value: "2" }, "2+ Stars"))))),
            React.createElement("div", { className: "marketplace-content" },
                React.createElement("div", { className: "marketplace-controls" },
                    React.createElement("input", { type: "search", className: "search-input", placeholder: "Search plugins...", value: searchQuery, onChange: (e) => setSearchQuery(e.target.value) }),
                    React.createElement("select", { className: "sort-select", value: sortBy, onChange: (e) => setSortBy(e.target.value) },
                        React.createElement("option", { value: SortBy.Relevance }, "Most Relevant"),
                        React.createElement("option", { value: SortBy.Downloads }, "Most Downloads"),
                        React.createElement("option", { value: SortBy.Rating }, "Highest Rated"),
                        React.createElement("option", { value: SortBy.Updated }, "Recently Updated"),
                        React.createElement("option", { value: SortBy.Name }, "Name (A-Z)"))),
                loading && (React.createElement("div", { className: "loading-spinner" }, "Searching marketplace...")),
                error && (React.createElement("div", { className: "error-message" },
                    React.createElement("p", null,
                        "Failed to load marketplace: ",
                        error.message),
                    React.createElement("button", { onClick: performSearch }, "Retry"))),
                results && !loading && (React.createElement(React.Fragment, null,
                    React.createElement("div", { className: "results-info" },
                        React.createElement("p", null,
                            "Showing ",
                            results.plugins.length,
                            " of ",
                            results.totalCount,
                            " plugins",
                            selectedCategory && ` in ${categories.find((c) => c.id === selectedCategory)?.name}`)),
                    React.createElement("div", { className: "plugin-grid" }, results.plugins.map((plugin) => (React.createElement(MarketplacePluginCard, { key: plugin.id, plugin: plugin, onInstall: handleInstall })))),
                    results.totalPages > 1 && (React.createElement(Pagination, { currentPage: results.page, totalPages: results.totalPages, onPageChange: handlePageChange }))))))));
};
const MarketplacePluginCard = ({ plugin, onInstall, }) => {
    const [installing, setInstalling] = useState(false);
    const handleInstall = async () => {
        setInstalling(true);
        try {
            await onInstall(plugin);
        }
        finally {
            setInstalling(false);
        }
    };
    const formatDownloads = (downloads) => {
        if (downloads >= 1000000) {
            return `${(downloads / 1000000).toFixed(1)}M`;
        }
        else if (downloads >= 1000) {
            return `${(downloads / 1000).toFixed(1)}K`;
        }
        return downloads.toString();
    };
    const formatSize = (bytes) => {
        if (bytes >= 1024 * 1024) {
            return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
        }
        else if (bytes >= 1024) {
            return `${(bytes / 1024).toFixed(1)} KB`;
        }
        return `${bytes} B`;
    };
    return (React.createElement("div", { className: "marketplace-plugin-card" },
        React.createElement("div", { className: "plugin-card-header" },
            plugin.iconUrl ? (React.createElement("img", { src: plugin.iconUrl, alt: plugin.name, className: "plugin-icon" })) : (React.createElement("div", { className: "plugin-icon-placeholder" }, plugin.name[0])),
            plugin.verified && (React.createElement("div", { className: "verified-badge", title: "Verified Plugin" }, "\u2713"))),
        React.createElement("div", { className: "plugin-card-body" },
            React.createElement("h3", null, plugin.name),
            React.createElement("p", { className: "plugin-author" },
                "by ",
                plugin.author.name),
            React.createElement("p", { className: "plugin-description" }, plugin.description),
            React.createElement("div", { className: "plugin-tags" }, plugin.categories.slice(0, 3).map((category) => (React.createElement("span", { key: category, className: "tag" }, category)))),
            React.createElement("div", { className: "plugin-stats" },
                React.createElement("div", { className: "stat" },
                    React.createElement("span", { className: "rating" },
                        '★'.repeat(Math.round(plugin.rating)),
                        '☆'.repeat(5 - Math.round(plugin.rating))),
                    React.createElement("span", { className: "rating-count" },
                        "(",
                        plugin.ratingCount,
                        ")")),
                React.createElement("div", { className: "stat" },
                    React.createElement("span", null,
                        "\u2193 ",
                        formatDownloads(plugin.downloads))))),
        React.createElement("div", { className: "plugin-card-footer" },
            React.createElement("div", { className: "plugin-meta" },
                React.createElement("span", null,
                    "v",
                    plugin.version),
                React.createElement("span", null, formatSize(plugin.sizeBytes)),
                React.createElement("span", null, plugin.license)),
            React.createElement("button", { className: "btn-install", onClick: handleInstall, disabled: installing }, installing ? 'Installing...' : 'Install'))));
};
const Pagination = ({ currentPage, totalPages, onPageChange, }) => {
    const pages = [];
    const maxVisible = 7;
    let startPage = Math.max(0, currentPage - Math.floor(maxVisible / 2));
    let endPage = Math.min(totalPages - 1, startPage + maxVisible - 1);
    if (endPage - startPage < maxVisible - 1) {
        startPage = Math.max(0, endPage - maxVisible + 1);
    }
    for (let i = startPage; i <= endPage; i++) {
        pages.push(i);
    }
    return (React.createElement("div", { className: "pagination" },
        React.createElement("button", { className: "page-btn", disabled: currentPage === 0, onClick: () => onPageChange(currentPage - 1) }, "Previous"),
        startPage > 0 && (React.createElement(React.Fragment, null,
            React.createElement("button", { className: "page-btn", onClick: () => onPageChange(0) }, "1"),
            startPage > 1 && React.createElement("span", { className: "page-ellipsis" }, "..."))),
        pages.map((page) => (React.createElement("button", { key: page, className: `page-btn ${page === currentPage ? 'active' : ''}`, onClick: () => onPageChange(page) }, page + 1))),
        endPage < totalPages - 1 && (React.createElement(React.Fragment, null,
            endPage < totalPages - 2 && React.createElement("span", { className: "page-ellipsis" }, "..."),
            React.createElement("button", { className: "page-btn", onClick: () => onPageChange(totalPages - 1) }, totalPages))),
        React.createElement("button", { className: "page-btn", disabled: currentPage === totalPages - 1, onClick: () => onPageChange(currentPage + 1) }, "Next")));
};
export default PluginMarketplace;
//# sourceMappingURL=PluginMarketplace.js.map