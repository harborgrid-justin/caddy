import React, { useState, useCallback, useRef, useEffect } from 'react';
const DEFAULT_CHUNK_SIZE = 5 * 1024 * 1024;
const DEFAULT_MAX_CONCURRENT = 3;
const DEFAULT_MAX_RETRIES = 3;
const DEFAULT_MAX_FILE_SIZE = 5 * 1024 * 1024 * 1024;
export const FileUpload = ({ tenantId, parentId = null, chunkSize = DEFAULT_CHUNK_SIZE, maxConcurrentUploads = DEFAULT_MAX_CONCURRENT, maxRetries = DEFAULT_MAX_RETRIES, allowedFileTypes = [], maxFileSize = DEFAULT_MAX_FILE_SIZE, onUploadComplete, onUploadError, onClose, className = '', }) => {
    const [tasks, setTasks] = useState([]);
    const [isDragging, setIsDragging] = useState(false);
    const [activeUploads, setActiveUploads] = useState(0);
    const fileInputRef = useRef(null);
    const dragCounter = useRef(0);
    const uploadQueueRef = useRef([]);
    useEffect(() => {
        processQueue();
    }, [tasks, activeUploads]);
    const processQueue = async () => {
        if (activeUploads >= maxConcurrentUploads)
            return;
        const pendingTask = tasks.find(t => t.status === 'pending');
        if (!pendingTask)
            return;
        setActiveUploads(prev => prev + 1);
        await uploadFile(pendingTask);
        setActiveUploads(prev => prev - 1);
    };
    const validateFile = (file) => {
        if (file.size > maxFileSize) {
            return {
                valid: false,
                error: `File size exceeds maximum of ${formatBytes(maxFileSize)}`,
            };
        }
        if (allowedFileTypes.length > 0) {
            const extension = file.name.split('.').pop()?.toLowerCase() || '';
            const mimeType = file.type;
            const isAllowed = allowedFileTypes.some(type => type === mimeType || type === `.${extension}` || type === extension);
            if (!isAllowed) {
                return {
                    valid: false,
                    error: `File type not allowed. Allowed types: ${allowedFileTypes.join(', ')}`,
                };
            }
        }
        return { valid: true };
    };
    const addFiles = useCallback((files) => {
        const fileArray = Array.from(files);
        const newTasks = [];
        fileArray.forEach(file => {
            const validation = validateFile(file);
            if (!validation.valid) {
                const errorTask = {
                    id: `upload-${Date.now()}-${Math.random()}`,
                    file,
                    name: file.name,
                    size: file.size,
                    uploaded: 0,
                    status: 'error',
                    progress: 0,
                    speed: 0,
                    remainingTime: 0,
                    error: validation.error,
                    chunkSize,
                    uploadedChunks: 0,
                    totalChunks: 0,
                    retryCount: 0,
                    startTime: Date.now(),
                    parentId: parentId ?? undefined,
                };
                newTasks.push(errorTask);
                return;
            }
            const totalChunks = Math.ceil(file.size / chunkSize);
            const task = {
                id: `upload-${Date.now()}-${Math.random()}`,
                file,
                name: file.name,
                size: file.size,
                uploaded: 0,
                status: 'pending',
                progress: 0,
                speed: 0,
                remainingTime: 0,
                chunkSize,
                uploadedChunks: 0,
                totalChunks,
                retryCount: 0,
                startTime: Date.now(),
                parentId: parentId ?? undefined,
            };
            newTasks.push(task);
        });
        setTasks(prev => [...prev, ...newTasks]);
    }, [chunkSize, maxFileSize, allowedFileTypes, parentId]);
    const uploadFile = async (task) => {
        try {
            setTasks(prev => prev.map(t => (t.id === task.id ? { ...t, status: 'uploading' } : t)));
            const initResponse = await fetch(`/api/v1/tenants/${tenantId}/files/upload/init`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    fileName: task.name,
                    fileSize: task.size,
                    mimeType: task.file.type,
                    parentId: task.parentId,
                    totalChunks: task.totalChunks,
                }),
            });
            if (!initResponse.ok) {
                throw new Error('Failed to initialize upload');
            }
            const { uploadId, uploadUrl } = await initResponse.json();
            for (let chunkIndex = 0; chunkIndex < task.totalChunks; chunkIndex++) {
                const start = chunkIndex * task.chunkSize;
                const end = Math.min(start + task.chunkSize, task.size);
                const chunk = task.file.slice(start, end);
                let retries = 0;
                let chunkUploaded = false;
                while (!chunkUploaded && retries <= maxRetries) {
                    try {
                        const chunkStartTime = Date.now();
                        const formData = new FormData();
                        formData.append('chunk', chunk);
                        formData.append('chunkIndex', chunkIndex.toString());
                        formData.append('uploadId', uploadId);
                        const chunkResponse = await fetch(uploadUrl, {
                            method: 'POST',
                            headers: {
                                'Authorization': `Bearer ${localStorage.getItem('token')}`,
                            },
                            body: formData,
                        });
                        if (!chunkResponse.ok) {
                            throw new Error(`Chunk ${chunkIndex} upload failed`);
                        }
                        const chunkTime = Date.now() - chunkStartTime;
                        const chunkSpeed = (chunk.size / chunkTime) * 1000;
                        const uploaded = end;
                        const progress = (uploaded / task.size) * 100;
                        const remaining = task.size - uploaded;
                        const remainingTime = remaining / chunkSpeed;
                        setTasks(prev => prev.map(t => t.id === task.id
                            ? {
                                ...t,
                                uploaded,
                                progress,
                                speed: chunkSpeed,
                                remainingTime,
                                uploadedChunks: chunkIndex + 1,
                            }
                            : t));
                        chunkUploaded = true;
                    }
                    catch (error) {
                        retries++;
                        if (retries > maxRetries) {
                            throw error;
                        }
                        await new Promise(resolve => setTimeout(resolve, 1000 * retries));
                    }
                }
            }
            const finalizeResponse = await fetch(`/api/v1/tenants/${tenantId}/files/upload/finalize`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('token')}`,
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ uploadId }),
            });
            if (!finalizeResponse.ok) {
                throw new Error('Failed to finalize upload');
            }
            const uploadedFile = await finalizeResponse.json();
            setTasks(prev => prev.map(t => t.id === task.id
                ? {
                    ...t,
                    status: 'complete',
                    progress: 100,
                    uploaded: t.size,
                    endTime: Date.now(),
                    url: uploadedFile.id,
                }
                : t));
            onUploadComplete?.([uploadedFile]);
        }
        catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Upload failed';
            setTasks(prev => prev.map(t => t.id === task.id
                ? {
                    ...t,
                    status: 'error',
                    error: errorMessage,
                    endTime: Date.now(),
                }
                : t));
            onUploadError?.(error instanceof Error ? error : new Error(errorMessage));
        }
    };
    const pauseUpload = useCallback((taskId) => {
        setTasks(prev => prev.map(t => (t.id === taskId && t.status === 'uploading' ? { ...t, status: 'paused' } : t)));
    }, []);
    const resumeUpload = useCallback((taskId) => {
        setTasks(prev => prev.map(t => (t.id === taskId && t.status === 'paused' ? { ...t, status: 'pending' } : t)));
    }, []);
    const cancelUpload = useCallback((taskId) => {
        setTasks(prev => prev.map(t => (t.id === taskId ? { ...t, status: 'cancelled' } : t)));
    }, []);
    const retryUpload = useCallback((taskId) => {
        setTasks(prev => prev.map(t => t.id === taskId
            ? {
                ...t,
                status: 'pending',
                error: undefined,
                uploaded: 0,
                progress: 0,
                uploadedChunks: 0,
                retryCount: t.retryCount + 1,
            }
            : t));
    }, []);
    const removeTask = useCallback((taskId) => {
        setTasks(prev => prev.filter(t => t.id !== taskId));
    }, []);
    const clearCompleted = useCallback(() => {
        setTasks(prev => prev.filter(t => t.status !== 'complete'));
    }, []);
    const handleDragEnter = useCallback((e) => {
        e.preventDefault();
        e.stopPropagation();
        dragCounter.current++;
        if (e.dataTransfer.items && e.dataTransfer.items.length > 0) {
            setIsDragging(true);
        }
    }, []);
    const handleDragLeave = useCallback((e) => {
        e.preventDefault();
        e.stopPropagation();
        dragCounter.current--;
        if (dragCounter.current === 0) {
            setIsDragging(false);
        }
    }, []);
    const handleDragOver = useCallback((e) => {
        e.preventDefault();
        e.stopPropagation();
    }, []);
    const handleDrop = useCallback((e) => {
        e.preventDefault();
        e.stopPropagation();
        setIsDragging(false);
        dragCounter.current = 0;
        if (e.dataTransfer.files && e.dataTransfer.files.length > 0) {
            addFiles(e.dataTransfer.files);
        }
    }, [addFiles]);
    const handleFileInput = useCallback((e) => {
        if (e.target.files && e.target.files.length > 0) {
            addFiles(e.target.files);
            e.target.value = '';
        }
    }, [addFiles]);
    const handleBrowseClick = useCallback(() => {
        fileInputRef.current?.click();
    }, []);
    const stats = {
        total: tasks.length,
        pending: tasks.filter(t => t.status === 'pending').length,
        uploading: tasks.filter(t => t.status === 'uploading').length,
        complete: tasks.filter(t => t.status === 'complete').length,
        error: tasks.filter(t => t.status === 'error').length,
        totalSize: tasks.reduce((sum, t) => sum + t.size, 0),
        uploadedSize: tasks.reduce((sum, t) => sum + t.uploaded, 0),
    };
    return (React.createElement("div", { className: `file-upload-container ${className}` },
        React.createElement("div", { className: "upload-header" },
            React.createElement("h2", null, "Upload Files"),
            React.createElement("button", { onClick: onClose, className: "btn-close" }, "\u2715")),
        React.createElement("div", { className: `upload-dropzone ${isDragging ? 'dragging' : ''}`, onDragEnter: handleDragEnter, onDragLeave: handleDragLeave, onDragOver: handleDragOver, onDrop: handleDrop, onClick: handleBrowseClick },
            React.createElement("input", { ref: fileInputRef, type: "file", multiple: true, onChange: handleFileInput, style: { display: 'none' }, accept: allowedFileTypes.join(',') }),
            React.createElement("div", { className: "dropzone-content" },
                React.createElement("div", { className: "dropzone-icon" }, "\uD83D\uDCC1"),
                React.createElement("div", { className: "dropzone-text" },
                    React.createElement("strong", null, "Drag and drop files here"),
                    React.createElement("br", null),
                    "or click to browse"),
                maxFileSize && (React.createElement("div", { className: "dropzone-limit" },
                    "Maximum file size: ",
                    formatBytes(maxFileSize))))),
        tasks.length > 0 && (React.createElement("div", { className: "upload-stats" },
            React.createElement("div", { className: "stat-item" },
                React.createElement("span", { className: "stat-label" }, "Total:"),
                React.createElement("span", { className: "stat-value" }, stats.total)),
            React.createElement("div", { className: "stat-item" },
                React.createElement("span", { className: "stat-label" }, "Uploading:"),
                React.createElement("span", { className: "stat-value" }, stats.uploading)),
            React.createElement("div", { className: "stat-item" },
                React.createElement("span", { className: "stat-label" }, "Complete:"),
                React.createElement("span", { className: "stat-value success" }, stats.complete)),
            React.createElement("div", { className: "stat-item" },
                React.createElement("span", { className: "stat-label" }, "Failed:"),
                React.createElement("span", { className: "stat-value error" }, stats.error)),
            React.createElement("div", { className: "stat-item" },
                React.createElement("span", { className: "stat-label" }, "Progress:"),
                React.createElement("span", { className: "stat-value" },
                    formatBytes(stats.uploadedSize),
                    " / ",
                    formatBytes(stats.totalSize))),
            stats.complete > 0 && (React.createElement("button", { onClick: clearCompleted, className: "btn btn-sm" }, "Clear Completed")))),
        React.createElement("div", { className: "upload-tasks" }, tasks.map(task => (React.createElement("div", { key: task.id, className: `upload-task upload-task-${task.status}` },
            React.createElement("div", { className: "task-info" },
                React.createElement("div", { className: "task-name" }, task.name),
                React.createElement("div", { className: "task-meta" },
                    React.createElement("span", null, formatBytes(task.size)),
                    task.status === 'uploading' && (React.createElement(React.Fragment, null,
                        React.createElement("span", null, "\u2022"),
                        React.createElement("span", null,
                            formatBytes(task.speed),
                            "/s"),
                        React.createElement("span", null, "\u2022"),
                        React.createElement("span", null,
                            formatTime(task.remainingTime),
                            " remaining"))),
                    task.status === 'complete' && task.startTime && task.endTime && (React.createElement(React.Fragment, null,
                        React.createElement("span", null, "\u2022"),
                        React.createElement("span", null,
                            "Completed in ",
                            formatTime((task.endTime - task.startTime) / 1000)))))),
            React.createElement("div", { className: "task-progress" }, task.status === 'error' ? (React.createElement("div", { className: "task-error" }, task.error)) : (React.createElement(React.Fragment, null,
                React.createElement("div", { className: "progress-bar-container" },
                    React.createElement("div", { className: "progress-bar", style: { width: `${task.progress}%` } })),
                React.createElement("div", { className: "progress-text" }, task.status === 'complete' ? '100%' : `${Math.round(task.progress)}%`)))),
            React.createElement("div", { className: "task-actions" },
                task.status === 'uploading' && (React.createElement("button", { onClick: () => pauseUpload(task.id), className: "btn btn-sm" }, "Pause")),
                task.status === 'paused' && (React.createElement("button", { onClick: () => resumeUpload(task.id), className: "btn btn-sm" }, "Resume")),
                task.status === 'error' && (React.createElement("button", { onClick: () => retryUpload(task.id), className: "btn btn-sm" }, "Retry")),
                (task.status === 'pending' ||
                    task.status === 'uploading' ||
                    task.status === 'paused') && (React.createElement("button", { onClick: () => cancelUpload(task.id), className: "btn btn-sm btn-danger" }, "Cancel")),
                (task.status === 'complete' || task.status === 'error' || task.status === 'cancelled') && (React.createElement("button", { onClick: () => removeTask(task.id), className: "btn btn-sm" }, "Remove")))))))));
};
function formatBytes(bytes) {
    if (bytes === 0)
        return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}
function formatTime(seconds) {
    if (seconds < 60)
        return `${Math.round(seconds)}s`;
    if (seconds < 3600)
        return `${Math.round(seconds / 60)}m`;
    return `${Math.round(seconds / 3600)}h`;
}
export default FileUpload;
//# sourceMappingURL=FileUpload.js.map