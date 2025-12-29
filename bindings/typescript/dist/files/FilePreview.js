import React, { useState, useEffect, useCallback, useRef } from 'react';
const DEFAULT_CONFIG = {
    maxSize: 100 * 1024 * 1024,
    supportedTypes: [
        'image/jpeg', 'image/png', 'image/gif', 'image/webp', 'image/svg+xml',
        'video/mp4', 'video/webm', 'video/ogg',
        'audio/mpeg', 'audio/wav', 'audio/ogg',
        'application/pdf',
        'text/plain', 'text/html', 'text/css', 'text/javascript',
        'application/json', 'application/xml',
    ],
    enableAnnotations: true,
    enableComments: true,
    enableDownload: true,
    enablePrint: true,
    enableShare: true,
};
export const FilePreview = ({ file, tenantId, config: userConfig = {}, onClose, onDownload, onShare, onDelete, onPrevious, onNext, className = '', }) => {
    const config = { ...DEFAULT_CONFIG, ...userConfig };
    const [previewData, setPreviewData] = useState(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const [currentPage, setCurrentPage] = useState(1);
    const [zoom, setZoom] = useState(100);
    const [rotation, setRotation] = useState(0);
    const contentRef = useRef(null);
    const videoRef = useRef(null);
    const audioRef = useRef(null);
    useEffect(() => {
        loadPreview();
    }, [file.id]);
    const loadPreview = async () => {
        setLoading(true);
        setError(null);
        try {
            if (!config.supportedTypes.includes(file.mimeType)) {
                setPreviewData({
                    file,
                    url: '',
                    type: 'unsupported',
                });
                setLoading(false);
                return;
            }
            if (file.size > config.maxSize) {
                throw new Error(`File too large for preview (max ${formatBytes(config.maxSize)})`);
            }
            const token = localStorage.getItem('token');
            const url = `/api/v1/tenants/${tenantId}/files/${file.id}/content?token=${token}`;
            const type = getPreviewType(file.mimeType);
            let content;
            if (type === 'text' || type === 'code') {
                const response = await fetch(url);
                if (!response.ok)
                    throw new Error('Failed to load file content');
                content = await response.text();
            }
            setPreviewData({
                file,
                url,
                type,
                content,
                pages: type === 'pdf' ? await getPdfPageCount(url) : undefined,
                currentPage: 1,
            });
            setLoading(false);
        }
        catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to load preview');
            setLoading(false);
        }
    };
    const getPreviewType = (mimeType) => {
        if (mimeType.startsWith('image/'))
            return 'image';
        if (mimeType.startsWith('video/'))
            return 'video';
        if (mimeType.startsWith('audio/'))
            return 'audio';
        if (mimeType === 'application/pdf')
            return 'pdf';
        if (mimeType.startsWith('text/') || mimeType.includes('json') || mimeType.includes('xml')) {
            return mimeType.includes('javascript') || mimeType.includes('json') || mimeType.includes('xml')
                ? 'code'
                : 'text';
        }
        return 'document';
    };
    const getPdfPageCount = async (url) => {
        return 1;
    };
    useEffect(() => {
        const handleKeyDown = (e) => {
            switch (e.key) {
                case 'Escape':
                    onClose();
                    break;
                case 'ArrowLeft':
                    if (e.ctrlKey || e.metaKey) {
                        onPrevious?.();
                    }
                    else if (previewData?.type === 'pdf' && currentPage > 1) {
                        setCurrentPage(p => p - 1);
                    }
                    break;
                case 'ArrowRight':
                    if (e.ctrlKey || e.metaKey) {
                        onNext?.();
                    }
                    else if (previewData?.type === 'pdf' && previewData.pages && currentPage < previewData.pages) {
                        setCurrentPage(p => p + 1);
                    }
                    break;
                case '+':
                case '=':
                    if (e.ctrlKey || e.metaKey) {
                        e.preventDefault();
                        setZoom(z => Math.min(z + 10, 200));
                    }
                    break;
                case '-':
                    if (e.ctrlKey || e.metaKey) {
                        e.preventDefault();
                        setZoom(z => Math.max(z - 10, 50));
                    }
                    break;
                case '0':
                    if (e.ctrlKey || e.metaKey) {
                        e.preventDefault();
                        setZoom(100);
                        setRotation(0);
                    }
                    break;
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [onClose, onPrevious, onNext, currentPage, previewData]);
    const handleDownload = useCallback(() => {
        if (previewData?.url) {
            const a = document.createElement('a');
            a.href = previewData.url;
            a.download = file.name;
            a.click();
        }
        onDownload?.();
    }, [previewData, file.name, onDownload]);
    const handlePrint = useCallback(() => {
        if (previewData?.type === 'image' || previewData?.type === 'pdf') {
            window.print();
        }
    }, [previewData]);
    const handleRotate = useCallback(() => {
        setRotation(r => (r + 90) % 360);
    }, []);
    const handleZoomIn = useCallback(() => {
        setZoom(z => Math.min(z + 10, 200));
    }, []);
    const handleZoomOut = useCallback(() => {
        setZoom(z => Math.max(z - 10, 50));
    }, []);
    const handleZoomReset = useCallback(() => {
        setZoom(100);
        setRotation(0);
    }, []);
    const renderPreviewContent = () => {
        if (loading) {
            return (React.createElement("div", { className: "preview-loading" },
                React.createElement("div", { className: "spinner" }),
                React.createElement("div", null, "Loading preview...")));
        }
        if (error) {
            return (React.createElement("div", { className: "preview-error" },
                React.createElement("div", { className: "error-icon" }, "\u26A0\uFE0F"),
                React.createElement("div", { className: "error-message" }, error),
                config.enableDownload && (React.createElement("button", { onClick: handleDownload, className: "btn btn-primary" }, "Download File"))));
        }
        if (!previewData) {
            return null;
        }
        switch (previewData.type) {
            case 'image':
                return (React.createElement("div", { className: "preview-image-container" },
                    React.createElement("img", { src: previewData.url, alt: file.name, style: {
                            transform: `scale(${zoom / 100}) rotate(${rotation}deg)`,
                            maxWidth: '100%',
                            maxHeight: '100%',
                        } })));
            case 'video':
                return (React.createElement("div", { className: "preview-video-container" },
                    React.createElement("video", { ref: videoRef, src: previewData.url, controls: true, style: { maxWidth: '100%', maxHeight: '100%' } }, "Your browser does not support the video tag.")));
            case 'audio':
                return (React.createElement("div", { className: "preview-audio-container" },
                    React.createElement("div", { className: "audio-info" },
                        React.createElement("div", { className: "audio-icon" }, "\uD83C\uDFB5"),
                        React.createElement("div", { className: "audio-name" }, file.name),
                        React.createElement("div", { className: "audio-size" }, formatBytes(file.size))),
                    React.createElement("audio", { ref: audioRef, src: previewData.url, controls: true, style: { width: '100%', maxWidth: '600px' } }, "Your browser does not support the audio tag.")));
            case 'pdf':
                return (React.createElement("div", { className: "preview-pdf-container" },
                    React.createElement("div", { className: "pdf-toolbar" },
                        React.createElement("button", { onClick: () => setCurrentPage(p => Math.max(1, p - 1)), disabled: currentPage === 1 }, "Previous"),
                        React.createElement("span", { className: "page-info" },
                            "Page ",
                            currentPage,
                            " of ",
                            previewData.pages || 1),
                        React.createElement("button", { onClick: () => setCurrentPage(p => Math.min(previewData.pages || 1, p + 1)), disabled: currentPage === (previewData.pages || 1) }, "Next")),
                    React.createElement("iframe", { src: `${previewData.url}#page=${currentPage}&zoom=${zoom}`, style: { width: '100%', height: '100%', border: 'none' }, title: file.name })));
            case 'text':
            case 'code':
                return (React.createElement("div", { className: "preview-text-container" },
                    React.createElement("pre", { className: previewData.type === 'code' ? 'code-preview' : 'text-preview' },
                        React.createElement("code", null, previewData.content))));
            case 'unsupported':
                return (React.createElement("div", { className: "preview-unsupported" },
                    React.createElement("div", { className: "unsupported-icon" }, "\uD83D\uDCC4"),
                    React.createElement("div", { className: "unsupported-message" }, "Preview not available for this file type"),
                    React.createElement("div", { className: "file-info" },
                        React.createElement("div", { className: "info-item" },
                            React.createElement("strong", null, "Name:"),
                            " ",
                            file.name),
                        React.createElement("div", { className: "info-item" },
                            React.createElement("strong", null, "Type:"),
                            " ",
                            file.mimeType),
                        React.createElement("div", { className: "info-item" },
                            React.createElement("strong", null, "Size:"),
                            " ",
                            formatBytes(file.size)),
                        React.createElement("div", { className: "info-item" },
                            React.createElement("strong", null, "Modified:"),
                            " ",
                            new Date(file.modifiedAt).toLocaleString())),
                    config.enableDownload && (React.createElement("button", { onClick: handleDownload, className: "btn btn-primary" }, "Download File"))));
            default:
                return null;
        }
    };
    return (React.createElement("div", { className: `file-preview-modal ${className}` },
        React.createElement("div", { className: "preview-overlay", onClick: onClose }),
        React.createElement("div", { className: "preview-container" },
            React.createElement("div", { className: "preview-header" },
                React.createElement("div", { className: "preview-title" },
                    React.createElement("span", { className: "preview-icon" }, getFileIcon(file)),
                    React.createElement("span", { className: "preview-filename" }, file.name)),
                React.createElement("div", { className: "preview-actions" },
                    onPrevious && (React.createElement("button", { onClick: onPrevious, className: "btn btn-icon", title: "Previous (Ctrl+\u2190)" }, "\u2190")),
                    onNext && (React.createElement("button", { onClick: onNext, className: "btn btn-icon", title: "Next (Ctrl+\u2192)" }, "\u2192")),
                    (previewData?.type === 'image' || previewData?.type === 'pdf') && (React.createElement(React.Fragment, null,
                        React.createElement("button", { onClick: handleZoomOut, className: "btn btn-icon", title: "Zoom out (Ctrl+-)" }, "\u2212"),
                        React.createElement("span", { className: "zoom-level" },
                            zoom,
                            "%"),
                        React.createElement("button", { onClick: handleZoomIn, className: "btn btn-icon", title: "Zoom in (Ctrl++)" }, "+"),
                        React.createElement("button", { onClick: handleZoomReset, className: "btn btn-icon", title: "Reset (Ctrl+0)" }, "Reset"))),
                    previewData?.type === 'image' && (React.createElement("button", { onClick: handleRotate, className: "btn btn-icon", title: "Rotate" }, "\u21BB")),
                    config.enableDownload && (React.createElement("button", { onClick: handleDownload, className: "btn btn-icon", title: "Download" }, "\u2B07")),
                    config.enablePrint && (previewData?.type === 'image' || previewData?.type === 'pdf') && (React.createElement("button", { onClick: handlePrint, className: "btn btn-icon", title: "Print" }, "\uD83D\uDDA8\uFE0F")),
                    config.enableShare && onShare && (React.createElement("button", { onClick: onShare, className: "btn btn-icon", title: "Share" }, "\uD83D\uDD17")),
                    onDelete && (React.createElement("button", { onClick: onDelete, className: "btn btn-icon btn-danger", title: "Delete" }, "\uD83D\uDDD1\uFE0F")),
                    React.createElement("button", { onClick: onClose, className: "btn btn-icon", title: "Close (Esc)" }, "\u2715"))),
            React.createElement("div", { className: "preview-content", ref: contentRef }, renderPreviewContent()),
            React.createElement("div", { className: "preview-footer" },
                React.createElement("div", { className: "preview-meta" },
                    React.createElement("span", null, formatBytes(file.size)),
                    React.createElement("span", null, "\u2022"),
                    React.createElement("span", null, new Date(file.modifiedAt).toLocaleString()),
                    React.createElement("span", null, "\u2022"),
                    React.createElement("span", null,
                        "Modified by ",
                        file.modifiedBy))))));
};
function getFileIcon(file) {
    if (file.type === 'folder')
        return '📁';
    const ext = file.name.split('.').pop()?.toLowerCase() || '';
    const iconMap = {
        jpg: '🖼️', jpeg: '🖼️', png: '🖼️', gif: '🖼️', svg: '🖼️', webp: '🖼️',
        pdf: '📕', doc: '📘', docx: '📘', xls: '📗', xlsx: '📗', ppt: '📙', pptx: '📙',
        txt: '📄', md: '📝',
        zip: '📦', rar: '📦', '7z': '📦',
        mp4: '🎬', mov: '🎬', avi: '🎬',
        mp3: '🎵', wav: '🎵', flac: '🎵',
    };
    return iconMap[ext] || '📄';
}
function formatBytes(bytes) {
    if (bytes === 0)
        return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}
export default FilePreview;
//# sourceMappingURL=FilePreview.js.map