/**
 * CADDY v0.4.0 - File Upload Component
 * Drag-and-drop multi-file upload with chunked upload and progress tracking
 */

import React, { useState, useCallback, useRef, useEffect } from 'react';
import { UploadTask, FileItem } from './types';

interface FileUploadProps {
  tenantId: string;
  parentId?: string | null;
  chunkSize?: number;
  maxConcurrentUploads?: number;
  maxRetries?: number;
  allowedFileTypes?: string[];
  maxFileSize?: number;
  onUploadComplete?: (files: FileItem[]) => void;
  onUploadError?: (error: Error) => void;
  onClose?: () => void;
  className?: string;
}

const DEFAULT_CHUNK_SIZE = 5 * 1024 * 1024; // 5MB
const DEFAULT_MAX_CONCURRENT = 3;
const DEFAULT_MAX_RETRIES = 3;
const DEFAULT_MAX_FILE_SIZE = 5 * 1024 * 1024 * 1024; // 5GB

export const FileUpload: React.FC<FileUploadProps> = ({
  tenantId,
  parentId = null,
  chunkSize = DEFAULT_CHUNK_SIZE,
  maxConcurrentUploads = DEFAULT_MAX_CONCURRENT,
  maxRetries = DEFAULT_MAX_RETRIES,
  allowedFileTypes = [],
  maxFileSize = DEFAULT_MAX_FILE_SIZE,
  onUploadComplete,
  onUploadError,
  onClose,
  className = '',
}) => {
  const [tasks, setTasks] = useState<UploadTask[]>([]);
  const [isDragging, setIsDragging] = useState(false);
  const [activeUploads, setActiveUploads] = useState(0);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const dragCounter = useRef(0);
  const uploadQueueRef = useRef<UploadTask[]>([]);

  // Process upload queue
  useEffect(() => {
    processQueue();
  }, [tasks, activeUploads]);

  const processQueue = async () => {
    if (activeUploads >= maxConcurrentUploads) return;

    const pendingTask = tasks.find(t => t.status === 'pending');
    if (!pendingTask) return;

    setActiveUploads(prev => prev + 1);
    await uploadFile(pendingTask);
    setActiveUploads(prev => prev - 1);
  };

  // Validate file
  const validateFile = (file: File): { valid: boolean; error?: string } => {
    if (file.size > maxFileSize) {
      return {
        valid: false,
        error: `File size exceeds maximum of ${formatBytes(maxFileSize)}`,
      };
    }

    if (allowedFileTypes.length > 0) {
      const extension = file.name.split('.').pop()?.toLowerCase() || '';
      const mimeType = file.type;
      const isAllowed = allowedFileTypes.some(
        type => type === mimeType || type === `.${extension}` || type === extension
      );

      if (!isAllowed) {
        return {
          valid: false,
          error: `File type not allowed. Allowed types: ${allowedFileTypes.join(', ')}`,
        };
      }
    }

    return { valid: true };
  };

  // Add files to upload queue
  const addFiles = useCallback(
    (files: FileList | File[]) => {
      const fileArray = Array.from(files);
      const newTasks: UploadTask[] = [];

      fileArray.forEach(file => {
        const validation = validateFile(file);

        if (!validation.valid) {
          const errorTask: UploadTask = {
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
        const task: UploadTask = {
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
    },
    [chunkSize, maxFileSize, allowedFileTypes, parentId]
  );

  // Upload single file with chunking
  const uploadFile = async (task: UploadTask) => {
    try {
      // Update task status
      setTasks(prev =>
        prev.map(t => (t.id === task.id ? { ...t, status: 'uploading' as const } : t))
      );

      // Initialize upload session
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

      // Upload chunks
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
            const chunkSpeed = (chunk.size / chunkTime) * 1000; // bytes per second
            const uploaded = end;
            const progress = (uploaded / task.size) * 100;
            const remaining = task.size - uploaded;
            const remainingTime = remaining / chunkSpeed;

            setTasks(prev =>
              prev.map(t =>
                t.id === task.id
                  ? {
                      ...t,
                      uploaded,
                      progress,
                      speed: chunkSpeed,
                      remainingTime,
                      uploadedChunks: chunkIndex + 1,
                    }
                  : t
              )
            );

            chunkUploaded = true;
          } catch (error) {
            retries++;
            if (retries > maxRetries) {
              throw error;
            }
            await new Promise(resolve => setTimeout(resolve, 1000 * retries));
          }
        }
      }

      // Finalize upload
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

      const uploadedFile: FileItem = await finalizeResponse.json();

      setTasks(prev =>
        prev.map(t =>
          t.id === task.id
            ? {
                ...t,
                status: 'complete' as const,
                progress: 100,
                uploaded: t.size,
                endTime: Date.now(),
                url: uploadedFile.id,
              }
            : t
        )
      );

      onUploadComplete?.([uploadedFile]);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Upload failed';

      setTasks(prev =>
        prev.map(t =>
          t.id === task.id
            ? {
                ...t,
                status: 'error' as const,
                error: errorMessage,
                endTime: Date.now(),
              }
            : t
        )
      );

      onUploadError?.(error instanceof Error ? error : new Error(errorMessage));
    }
  };

  // Pause upload
  const pauseUpload = useCallback((taskId: string) => {
    setTasks(prev =>
      prev.map(t => (t.id === taskId && t.status === 'uploading' ? { ...t, status: 'paused' as const } : t))
    );
  }, []);

  // Resume upload
  const resumeUpload = useCallback((taskId: string) => {
    setTasks(prev =>
      prev.map(t => (t.id === taskId && t.status === 'paused' ? { ...t, status: 'pending' as const } : t))
    );
  }, []);

  // Cancel upload
  const cancelUpload = useCallback((taskId: string) => {
    setTasks(prev =>
      prev.map(t => (t.id === taskId ? { ...t, status: 'cancelled' as const } : t))
    );
  }, []);

  // Retry upload
  const retryUpload = useCallback((taskId: string) => {
    setTasks(prev =>
      prev.map(t =>
        t.id === taskId
          ? {
              ...t,
              status: 'pending' as const,
              error: undefined,
              uploaded: 0,
              progress: 0,
              uploadedChunks: 0,
              retryCount: t.retryCount + 1,
            }
          : t
      )
    );
  }, []);

  // Remove task
  const removeTask = useCallback((taskId: string) => {
    setTasks(prev => prev.filter(t => t.id !== taskId));
  }, []);

  // Clear completed
  const clearCompleted = useCallback(() => {
    setTasks(prev => prev.filter(t => t.status !== 'complete'));
  }, []);

  // Drag and drop handlers
  const handleDragEnter = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    dragCounter.current++;
    if (e.dataTransfer.items && e.dataTransfer.items.length > 0) {
      setIsDragging(true);
    }
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    dragCounter.current--;
    if (dragCounter.current === 0) {
      setIsDragging(false);
    }
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setIsDragging(false);
      dragCounter.current = 0;

      if (e.dataTransfer.files && e.dataTransfer.files.length > 0) {
        addFiles(e.dataTransfer.files);
      }
    },
    [addFiles]
  );

  // File input handler
  const handleFileInput = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      if (e.target.files && e.target.files.length > 0) {
        addFiles(e.target.files);
        e.target.value = '';
      }
    },
    [addFiles]
  );

  const handleBrowseClick = useCallback(() => {
    fileInputRef.current?.click();
  }, []);

  // Statistics
  const stats = {
    total: tasks.length,
    pending: tasks.filter(t => t.status === 'pending').length,
    uploading: tasks.filter(t => t.status === 'uploading').length,
    complete: tasks.filter(t => t.status === 'complete').length,
    error: tasks.filter(t => t.status === 'error').length,
    totalSize: tasks.reduce((sum, t) => sum + t.size, 0),
    uploadedSize: tasks.reduce((sum, t) => sum + t.uploaded, 0),
  };

  return (
    <div className={`file-upload-container ${className}`}>
      <div className="upload-header">
        <h2>Upload Files</h2>
        <button onClick={onClose} className="btn-close">
          ✕
        </button>
      </div>

      {/* Drop Zone */}
      <div
        className={`upload-dropzone ${isDragging ? 'dragging' : ''}`}
        onDragEnter={handleDragEnter}
        onDragLeave={handleDragLeave}
        onDragOver={handleDragOver}
        onDrop={handleDrop}
        onClick={handleBrowseClick}
      >
        <input
          ref={fileInputRef}
          type="file"
          multiple
          onChange={handleFileInput}
          style={{ display: 'none' }}
          accept={allowedFileTypes.join(',')}
        />
        <div className="dropzone-content">
          <div className="dropzone-icon">📁</div>
          <div className="dropzone-text">
            <strong>Drag and drop files here</strong>
            <br />
            or click to browse
          </div>
          {maxFileSize && (
            <div className="dropzone-limit">
              Maximum file size: {formatBytes(maxFileSize)}
            </div>
          )}
        </div>
      </div>

      {/* Statistics */}
      {tasks.length > 0 && (
        <div className="upload-stats">
          <div className="stat-item">
            <span className="stat-label">Total:</span>
            <span className="stat-value">{stats.total}</span>
          </div>
          <div className="stat-item">
            <span className="stat-label">Uploading:</span>
            <span className="stat-value">{stats.uploading}</span>
          </div>
          <div className="stat-item">
            <span className="stat-label">Complete:</span>
            <span className="stat-value success">{stats.complete}</span>
          </div>
          <div className="stat-item">
            <span className="stat-label">Failed:</span>
            <span className="stat-value error">{stats.error}</span>
          </div>
          <div className="stat-item">
            <span className="stat-label">Progress:</span>
            <span className="stat-value">
              {formatBytes(stats.uploadedSize)} / {formatBytes(stats.totalSize)}
            </span>
          </div>
          {stats.complete > 0 && (
            <button onClick={clearCompleted} className="btn btn-sm">
              Clear Completed
            </button>
          )}
        </div>
      )}

      {/* Upload Tasks */}
      <div className="upload-tasks">
        {tasks.map(task => (
          <div key={task.id} className={`upload-task upload-task-${task.status}`}>
            <div className="task-info">
              <div className="task-name">{task.name}</div>
              <div className="task-meta">
                <span>{formatBytes(task.size)}</span>
                {task.status === 'uploading' && (
                  <>
                    <span>•</span>
                    <span>{formatBytes(task.speed)}/s</span>
                    <span>•</span>
                    <span>{formatTime(task.remainingTime)} remaining</span>
                  </>
                )}
                {task.status === 'complete' && task.startTime && task.endTime && (
                  <>
                    <span>•</span>
                    <span>Completed in {formatTime((task.endTime - task.startTime) / 1000)}</span>
                  </>
                )}
              </div>
            </div>

            <div className="task-progress">
              {task.status === 'error' ? (
                <div className="task-error">{task.error}</div>
              ) : (
                <>
                  <div className="progress-bar-container">
                    <div className="progress-bar" style={{ width: `${task.progress}%` }} />
                  </div>
                  <div className="progress-text">
                    {task.status === 'complete' ? '100%' : `${Math.round(task.progress)}%`}
                  </div>
                </>
              )}
            </div>

            <div className="task-actions">
              {task.status === 'uploading' && (
                <button onClick={() => pauseUpload(task.id)} className="btn btn-sm">
                  Pause
                </button>
              )}
              {task.status === 'paused' && (
                <button onClick={() => resumeUpload(task.id)} className="btn btn-sm">
                  Resume
                </button>
              )}
              {task.status === 'error' && (
                <button onClick={() => retryUpload(task.id)} className="btn btn-sm">
                  Retry
                </button>
              )}
              {(task.status === 'pending' ||
                task.status === 'uploading' ||
                task.status === 'paused') && (
                <button onClick={() => cancelUpload(task.id)} className="btn btn-sm btn-danger">
                  Cancel
                </button>
              )}
              {(task.status === 'complete' || task.status === 'error' || task.status === 'cancelled') && (
                <button onClick={() => removeTask(task.id)} className="btn btn-sm">
                  Remove
                </button>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}

function formatTime(seconds: number): string {
  if (seconds < 60) return `${Math.round(seconds)}s`;
  if (seconds < 3600) return `${Math.round(seconds / 60)}m`;
  return `${Math.round(seconds / 3600)}h`;
}

export default FileUpload;
