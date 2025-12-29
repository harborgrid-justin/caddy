import React, { useState, useCallback, useEffect, useRef } from 'react';
export const FileSearch = ({ tenantId, onFileSelect, onFileOpen, initialQuery = '', className = '', }) => {
    const [query, setQuery] = useState(initialQuery);
    const [filters, setFilters] = useState({});
    const [sortField, setSortField] = useState('modified');
    const [sortDirection, setSortDirection] = useState('desc');
    const [page, setPage] = useState(1);
    const [limit] = useState(20);
    const [result, setResult] = useState(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState(null);
    const [showFilters, setShowFilters] = useState(false);
    const searchTimeoutRef = useRef(undefined);
    useEffect(() => {
        if (searchTimeoutRef.current) {
            clearTimeout(searchTimeoutRef.current);
        }
        if (query.trim().length >= 2 || Object.keys(filters).length > 0) {
            searchTimeoutRef.current = setTimeout(() => {
                performSearch();
            }, 300);
        }
        else {
            setResult(null);
        }
        return () => {
            if (searchTimeoutRef.current) {
                clearTimeout(searchTimeoutRef.current);
            }
        };
    }, [query, filters, sortField, sortDirection, page]);
    const performSearch = async () => {
        setLoading(true);
        setError(null);
        try {
            const searchQuery = {
                query: query.trim(),
                filters,
                sort: {
                    field: sortField,
                    direction: sortDirection,
                },
                pagination: {
                    page,
                    limit,
                },
            };
            const response = await fetch(`/api/v1/tenants/${tenantId}/files/search`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(searchQuery),
            });
            if (!response.ok) {
                throw new Error('Search failed');
            }
            const data = await response.json();
            setResult(data);
        }
        catch (err) {
            setError(err instanceof Error ? err.message : 'Search failed');
        }
        finally {
            setLoading(false);
        }
    };
    const updateFilter = useCallback((key, value) => {
        setFilters(prev => {
            if (value === undefined || value === null || value === '' || (Array.isArray(value) && value.length === 0)) {
                const { [key]: removed, ...rest } = prev;
                return rest;
            }
            return { ...prev, [key]: value };
        });
        setPage(1);
    }, []);
    const clearFilters = useCallback(() => {
        setFilters({});
        setPage(1);
    }, []);
    const handleSort = useCallback((field) => {
        setSortDirection(prev => sortField === field && prev === 'asc' ? 'desc' : 'asc');
        setSortField(field);
        setPage(1);
    }, [sortField]);
    const handlePageChange = useCallback((newPage) => {
        setPage(newPage);
        window.scrollTo(0, 0);
    }, []);
    const formatBytes = (bytes) => {
        if (bytes === 0)
            return '0 Bytes';
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
    };
    const formatDate = (date) => {
        const d = new Date(date);
        const now = new Date();
        const diffMs = now.getTime() - d.getTime();
        const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
        if (diffDays === 0)
            return 'Today';
        if (diffDays === 1)
            return 'Yesterday';
        if (diffDays < 7)
            return `${diffDays} days ago`;
        return d.toLocaleDateString();
    };
    const highlightText = (text, fileId) => {
        if (!result?.highlights?.[fileId])
            return text;
        const highlights = result.highlights[fileId];
        if (!highlights || highlights.length === 0)
            return text;
        let highlightedText = text;
        highlights.forEach(highlight => {
            const regex = new RegExp(`(${highlight})`, 'gi');
            highlightedText = highlightedText.replace(regex, '<mark>$1</mark>');
        });
        return React.createElement("span", { dangerouslySetInnerHTML: { __html: highlightedText } });
    };
    const activeFilterCount = Object.keys(filters).length;
    return (React.createElement("div", { className: `file-search ${className}` },
        React.createElement("div", { className: "search-header" },
            React.createElement("div", { className: "search-input-container" },
                React.createElement("input", { type: "text", placeholder: "Search files and folders...", value: query, onChange: (e) => setQuery(e.target.value), className: "search-input", autoFocus: true }),
                loading && React.createElement("div", { className: "search-spinner" }, "\u23F3")),
            React.createElement("button", { onClick: () => setShowFilters(!showFilters), className: `btn ${activeFilterCount > 0 ? 'btn-primary' : ''}` },
                "Filters ",
                activeFilterCount > 0 && `(${activeFilterCount})`)),
        showFilters && (React.createElement("div", { className: "search-filters" },
            React.createElement("div", { className: "filters-grid" },
                React.createElement("div", { className: "filter-group" },
                    React.createElement("label", null, "Type"),
                    React.createElement("select", { value: filters.type || '', onChange: (e) => updateFilter('type', e.target.value || undefined), className: "form-select" },
                        React.createElement("option", { value: "" }, "All"),
                        React.createElement("option", { value: "file" }, "Files"),
                        React.createElement("option", { value: "folder" }, "Folders"))),
                React.createElement("div", { className: "filter-group" },
                    React.createElement("label", null, "File Format"),
                    React.createElement("select", { value: filters.mimeTypes?.[0] || '', onChange: (e) => updateFilter('mimeTypes', e.target.value ? [e.target.value] : undefined), className: "form-select" },
                        React.createElement("option", { value: "" }, "All formats"),
                        React.createElement("optgroup", { label: "Documents" },
                            React.createElement("option", { value: "application/pdf" }, "PDF"),
                            React.createElement("option", { value: "application/msword" }, "Word"),
                            React.createElement("option", { value: "application/vnd.ms-excel" }, "Excel"),
                            React.createElement("option", { value: "text/plain" }, "Text")),
                        React.createElement("optgroup", { label: "Images" },
                            React.createElement("option", { value: "image/jpeg" }, "JPEG"),
                            React.createElement("option", { value: "image/png" }, "PNG"),
                            React.createElement("option", { value: "image/gif" }, "GIF")),
                        React.createElement("optgroup", { label: "Media" },
                            React.createElement("option", { value: "video/mp4" }, "Video (MP4)"),
                            React.createElement("option", { value: "audio/mpeg" }, "Audio (MP3)")))),
                React.createElement("div", { className: "filter-group" },
                    React.createElement("label", null, "Min Size"),
                    React.createElement("select", { value: filters.minSize || '', onChange: (e) => updateFilter('minSize', e.target.value ? parseInt(e.target.value) : undefined), className: "form-select" },
                        React.createElement("option", { value: "" }, "No minimum"),
                        React.createElement("option", { value: 1024 }, "1 KB"),
                        React.createElement("option", { value: 1024 * 100 }, "100 KB"),
                        React.createElement("option", { value: 1024 * 1024 }, "1 MB"),
                        React.createElement("option", { value: 1024 * 1024 * 10 }, "10 MB"),
                        React.createElement("option", { value: 1024 * 1024 * 100 }, "100 MB"))),
                React.createElement("div", { className: "filter-group" },
                    React.createElement("label", null, "Max Size"),
                    React.createElement("select", { value: filters.maxSize || '', onChange: (e) => updateFilter('maxSize', e.target.value ? parseInt(e.target.value) : undefined), className: "form-select" },
                        React.createElement("option", { value: "" }, "No maximum"),
                        React.createElement("option", { value: 1024 * 1024 }, "1 MB"),
                        React.createElement("option", { value: 1024 * 1024 * 10 }, "10 MB"),
                        React.createElement("option", { value: 1024 * 1024 * 100 }, "100 MB"),
                        React.createElement("option", { value: 1024 * 1024 * 1024 }, "1 GB"))),
                React.createElement("div", { className: "filter-group" },
                    React.createElement("label", null, "Modified After"),
                    React.createElement("input", { type: "date", value: filters.modifiedAfter ? new Date(filters.modifiedAfter).toISOString().split('T')[0] : '', onChange: (e) => updateFilter('modifiedAfter', e.target.value ? new Date(e.target.value) : undefined), className: "form-input" })),
                React.createElement("div", { className: "filter-group" },
                    React.createElement("label", null, "Modified Before"),
                    React.createElement("input", { type: "date", value: filters.modifiedBefore ? new Date(filters.modifiedBefore).toISOString().split('T')[0] : '', onChange: (e) => updateFilter('modifiedBefore', e.target.value ? new Date(e.target.value) : undefined), className: "form-input" })),
                React.createElement("div", { className: "filter-group" },
                    React.createElement("label", null,
                        React.createElement("input", { type: "checkbox", checked: filters.isStarred || false, onChange: (e) => updateFilter('isStarred', e.target.checked || undefined) }),
                        ' ',
                        "Starred only")),
                React.createElement("div", { className: "filter-group" },
                    React.createElement("label", null,
                        React.createElement("input", { type: "checkbox", checked: filters.isTrashed || false, onChange: (e) => updateFilter('isTrashed', e.target.checked || undefined) }),
                        ' ',
                        "Include trash"))),
            React.createElement("div", { className: "filter-actions" },
                React.createElement("button", { onClick: clearFilters, className: "btn" }, "Clear Filters")))),
        React.createElement("div", { className: "search-results" }, error ? (React.createElement("div", { className: "error-state" }, error)) : !result && query.trim().length < 2 && activeFilterCount === 0 ? (React.createElement("div", { className: "empty-state" }, "Enter a search query or apply filters to find files")) : !result ? (React.createElement("div", { className: "empty-state" }, "Type to search...")) : result.items.length === 0 ? (React.createElement("div", { className: "empty-state" },
            "No results found for \"",
            query,
            "\"",
            activeFilterCount > 0 && ' with the applied filters')) : (React.createElement(React.Fragment, null,
            React.createElement("div", { className: "results-header" },
                React.createElement("div", { className: "results-info" },
                    "Found ",
                    result.total.toLocaleString(),
                    " result",
                    result.total !== 1 ? 's' : '',
                    result.took && ` in ${result.took}ms`),
                React.createElement("div", { className: "results-sort" },
                    React.createElement("label", null, "Sort by:"),
                    React.createElement("select", { value: sortField, onChange: (e) => handleSort(e.target.value), className: "form-select form-select-sm" },
                        React.createElement("option", { value: "name" }, "Name"),
                        React.createElement("option", { value: "modified" }, "Modified"),
                        React.createElement("option", { value: "created" }, "Created"),
                        React.createElement("option", { value: "size" }, "Size"),
                        React.createElement("option", { value: "type" }, "Type")),
                    React.createElement("button", { onClick: () => setSortDirection(prev => prev === 'asc' ? 'desc' : 'asc'), className: "btn btn-sm" }, sortDirection === 'asc' ? '↑' : '↓'))),
            React.createElement("div", { className: "results-list" }, result.items.map(file => (React.createElement("div", { key: file.id, className: "result-item", onClick: () => onFileSelect?.(file), onDoubleClick: () => onFileOpen?.(file) },
                React.createElement("div", { className: "result-icon" }, file.type === 'folder' ? '📁' : '📄'),
                React.createElement("div", { className: "result-info" },
                    React.createElement("div", { className: "result-name" },
                        highlightText(file.name, file.id),
                        file.isStarred && React.createElement("span", { className: "star-icon" }, "\u2B50")),
                    React.createElement("div", { className: "result-path" }, file.path),
                    React.createElement("div", { className: "result-meta" },
                        file.type === 'file' && (React.createElement(React.Fragment, null,
                            React.createElement("span", null, formatBytes(file.size)),
                            React.createElement("span", null, "\u2022"))),
                        React.createElement("span", null, formatDate(file.modifiedAt)),
                        React.createElement("span", null, "\u2022"),
                        React.createElement("span", null,
                            "Modified by ",
                            file.modifiedBy))))))),
            result.totalPages > 1 && (React.createElement("div", { className: "results-pagination" },
                React.createElement("button", { onClick: () => handlePageChange(page - 1), disabled: page === 1, className: "btn btn-sm" }, "Previous"),
                React.createElement("div", { className: "pagination-info" },
                    "Page ",
                    page,
                    " of ",
                    result.totalPages),
                React.createElement("button", { onClick: () => handlePageChange(page + 1), disabled: page === result.totalPages, className: "btn btn-sm" }, "Next"))))))));
};
export default FileSearch;
//# sourceMappingURL=FileSearch.js.map