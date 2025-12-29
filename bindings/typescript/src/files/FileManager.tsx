/**
 * CADDY v0.4.0 - Enterprise File Manager
 * Main file browser interface with complete file management capabilities
 */

import React, { useState, useEffect, useCallback, useRef } from 'react';
import {
  FileItem,
  FileManagerState,
  FileManagerConfig,
  FileManagerCallbacks,
  ViewMode,
  SortField,
  SortDirection,
  BreadcrumbItem,
  FileOperation,
} from './types';

interface FileManagerProps {
  rootFolderId?: string | null;
  config?: Partial<FileManagerConfig>;
  callbacks?: FileManagerCallbacks;
  tenantId: string;
  userId: string;
  className?: string;
}

const DEFAULT_CONFIG: FileManagerConfig = {
  chunkSize: 5 * 1024 * 1024, // 5MB chunks
  maxConcurrentUploads: 3,
  maxRetries: 3,
  allowedFileTypes: [],
  maxFileSize: 5 * 1024 * 1024 * 1024, // 5GB
  thumbnailSize: 200,
  previewFormats: {
    images: ['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg', 'bmp'],
    videos: ['mp4', 'webm', 'ogg', 'mov', 'avi'],
    documents: ['pdf', 'doc', 'docx', 'xls', 'xlsx', 'ppt', 'pptx', 'txt', 'md'],
    audio: ['mp3', 'wav', 'ogg', 'flac', 'm4a'],
  },
  enableVersioning: true,
  enableCloudSync: true,
  enableFullTextSearch: true,
  trashRetentionDays: 30,
};

export const FileManager: React.FC<FileManagerProps> = ({
  rootFolderId = null,
  config: userConfig = {},
  callbacks = {},
  tenantId,
  userId,
  className = '',
}) => {
  const config = { ...DEFAULT_CONFIG, ...userConfig };
  const [state, setState] = useState<FileManagerState>({
    currentPath: '/',
    currentFolderId: rootFolderId,
    files: [],
    selectedFiles: [],
    viewMode: 'grid',
    sortField: 'name',
    sortDirection: 'asc',
    loading: false,
    error: null,
    breadcrumbs: [{ id: 'root', name: 'My Files', path: '/' }],
    clipboard: {
      items: [],
      operation: null,
    },
  });

  const [showUploadPanel, setShowUploadPanel] = useState(false);
  const [showPreview, setShowPreview] = useState(false);
  const [previewFile, setPreviewFile] = useState<FileItem | null>(null);
  const [showShareDialog, setShowShareDialog] = useState(false);
  const [shareFile, setShareFile] = useState<FileItem | null>(null);
  const [showVersions, setShowVersions] = useState(false);
  const [versionsFile, setVersionsFile] = useState<FileItem | null>(null);
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; file: FileItem } | null>(null);
  const [operations, setOperations] = useState<FileOperation[]>([]);
  const fileListRef = useRef<HTMLDivElement>(null);

  // Load files for current folder
  const loadFiles = useCallback(async () => {
    setState(prev => ({ ...prev, loading: true, error: null }));
    try {
      const response = await fetch(
        `/api/v1/tenants/${tenantId}/files?folderId=${state.currentFolderId || 'root'}&sort=${state.sortField}&direction=${state.sortDirection}`,
        {
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('token')}`,
          },
        }
      );

      if (!response.ok) {
        throw new Error(`Failed to load files: ${response.statusText}`);
      }

      const data = await response.json();
      setState(prev => ({ ...prev, files: data.files, loading: false }));
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to load files';
      setState(prev => ({ ...prev, error: errorMessage, loading: false }));
      callbacks.onError?.(error instanceof Error ? error : new Error(errorMessage));
    }
  }, [tenantId, state.currentFolderId, state.sortField, state.sortDirection, callbacks]);

  useEffect(() => {
    loadFiles();
  }, [loadFiles]);

  // Navigate to folder
  const navigateToFolder = useCallback(async (folderId: string | null, folderName?: string) => {
    let newBreadcrumbs: BreadcrumbItem[] = [];
    let newPath = '/';

    if (folderId) {
      // Fetch folder hierarchy for breadcrumbs
      try {
        const response = await fetch(`/api/v1/tenants/${tenantId}/files/${folderId}/path`, {
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('token')}`,
          },
        });

        if (response.ok) {
          const pathData = await response.json();
          newBreadcrumbs = pathData.breadcrumbs;
          newPath = pathData.path;
        }
      } catch (error) {
        console.error('Failed to fetch folder path:', error);
      }
    } else {
      newBreadcrumbs = [{ id: 'root', name: 'My Files', path: '/' }];
    }

    setState(prev => ({
      ...prev,
      currentFolderId: folderId,
      currentPath: newPath,
      breadcrumbs: newBreadcrumbs,
      selectedFiles: [],
    }));

    callbacks.onFolderChange?.(folderId);
  }, [tenantId, callbacks]);

  // File selection
  const toggleFileSelection = useCallback((fileId: string, multi: boolean = false) => {
    setState(prev => {
      if (multi) {
        const isSelected = prev.selectedFiles.includes(fileId);
        return {
          ...prev,
          selectedFiles: isSelected
            ? prev.selectedFiles.filter(id => id !== fileId)
            : [...prev.selectedFiles, fileId],
        };
      } else {
        return {
          ...prev,
          selectedFiles: [fileId],
        };
      }
    });
  }, []);

  const selectAll = useCallback(() => {
    setState(prev => ({
      ...prev,
      selectedFiles: prev.files.map(f => f.id),
    }));
  }, []);

  const clearSelection = useCallback(() => {
    setState(prev => ({
      ...prev,
      selectedFiles: [],
    }));
  }, []);

  // File operations
  const createFolder = useCallback(async (name: string) => {
    try {
      const response = await fetch(`/api/v1/tenants/${tenantId}/files/folders`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          name,
          parentId: state.currentFolderId,
        }),
      });

      if (!response.ok) {
        throw new Error('Failed to create folder');
      }

      await loadFiles();
    } catch (error) {
      callbacks.onError?.(error instanceof Error ? error : new Error('Failed to create folder'));
    }
  }, [tenantId, state.currentFolderId, loadFiles, callbacks]);

  const deleteFiles = useCallback(async (fileIds: string[]) => {
    const operationId = `delete-${Date.now()}`;
    const operation: FileOperation = {
      id: operationId,
      type: 'delete',
      fileIds,
      status: 'processing',
      progress: 0,
      total: fileIds.length,
      completed: 0,
      failed: 0,
      createdAt: new Date(),
    };

    setOperations(prev => [...prev, operation]);

    try {
      const response = await fetch(`/api/v1/tenants/${tenantId}/files/bulk-delete`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ fileIds }),
      });

      if (!response.ok) {
        throw new Error('Failed to delete files');
      }

      const result = await response.json();

      setOperations(prev =>
        prev.map(op =>
          op.id === operationId
            ? { ...op, status: 'complete', progress: 100, completed: result.successCount, failed: result.failedCount, completedAt: new Date() }
            : op
        )
      );

      callbacks.onFileDelete?.(result.success);
      await loadFiles();
      clearSelection();
    } catch (error) {
      setOperations(prev =>
        prev.map(op =>
          op.id === operationId
            ? { ...op, status: 'error', error: error instanceof Error ? error.message : 'Delete failed', completedAt: new Date() }
            : op
        )
      );
      callbacks.onError?.(error instanceof Error ? error : new Error('Failed to delete files'));
    }
  }, [tenantId, loadFiles, clearSelection, callbacks]);

  const renameFile = useCallback(async (fileId: string, newName: string) => {
    try {
      const response = await fetch(`/api/v1/tenants/${tenantId}/files/${fileId}/rename`, {
        method: 'PATCH',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ name: newName }),
      });

      if (!response.ok) {
        throw new Error('Failed to rename file');
      }

      await loadFiles();
    } catch (error) {
      callbacks.onError?.(error instanceof Error ? error : new Error('Failed to rename file'));
    }
  }, [tenantId, loadFiles, callbacks]);

  const moveFiles = useCallback(async (fileIds: string[], destinationId: string | null) => {
    const operationId = `move-${Date.now()}`;
    const operation: FileOperation = {
      id: operationId,
      type: 'move',
      fileIds,
      status: 'processing',
      progress: 0,
      total: fileIds.length,
      completed: 0,
      failed: 0,
      destination: destinationId || 'root',
      createdAt: new Date(),
    };

    setOperations(prev => [...prev, operation]);

    try {
      const response = await fetch(`/api/v1/tenants/${tenantId}/files/bulk-move`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ fileIds, destinationId }),
      });

      if (!response.ok) {
        throw new Error('Failed to move files');
      }

      const result = await response.json();

      setOperations(prev =>
        prev.map(op =>
          op.id === operationId
            ? { ...op, status: 'complete', progress: 100, completed: result.successCount, failed: result.failedCount, completedAt: new Date() }
            : op
        )
      );

      await loadFiles();
      clearSelection();
    } catch (error) {
      setOperations(prev =>
        prev.map(op =>
          op.id === operationId
            ? { ...op, status: 'error', error: error instanceof Error ? error.message : 'Move failed', completedAt: new Date() }
            : op
        )
      );
      callbacks.onError?.(error instanceof Error ? error : new Error('Failed to move files'));
    }
  }, [tenantId, loadFiles, clearSelection, callbacks]);

  const copyFiles = useCallback(async (fileIds: string[], destinationId: string | null) => {
    const operationId = `copy-${Date.now()}`;
    const operation: FileOperation = {
      id: operationId,
      type: 'copy',
      fileIds,
      status: 'processing',
      progress: 0,
      total: fileIds.length,
      completed: 0,
      failed: 0,
      destination: destinationId || 'root',
      createdAt: new Date(),
    };

    setOperations(prev => [...prev, operation]);

    try {
      const response = await fetch(`/api/v1/tenants/${tenantId}/files/bulk-copy`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ fileIds, destinationId }),
      });

      if (!response.ok) {
        throw new Error('Failed to copy files');
      }

      const result = await response.json();

      setOperations(prev =>
        prev.map(op =>
          op.id === operationId
            ? { ...op, status: 'complete', progress: 100, completed: result.successCount, failed: result.failedCount, completedAt: new Date() }
            : op
        )
      );

      await loadFiles();
    } catch (error) {
      setOperations(prev =>
        prev.map(op =>
          op.id === operationId
            ? { ...op, status: 'error', error: error instanceof Error ? error.message : 'Copy failed', completedAt: new Date() }
            : op
        )
      );
      callbacks.onError?.(error instanceof Error ? error : new Error('Failed to copy files'));
    }
  }, [tenantId, loadFiles, callbacks]);

  const toggleStar = useCallback(async (fileId: string) => {
    try {
      const file = state.files.find(f => f.id === fileId);
      if (!file) return;

      const response = await fetch(`/api/v1/tenants/${tenantId}/files/${fileId}/star`, {
        method: file.isStarred ? 'DELETE' : 'POST',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
      });

      if (!response.ok) {
        throw new Error('Failed to toggle star');
      }

      await loadFiles();
    } catch (error) {
      callbacks.onError?.(error instanceof Error ? error : new Error('Failed to toggle star'));
    }
  }, [tenantId, state.files, loadFiles, callbacks]);

  const downloadFiles = useCallback(async (fileIds: string[]) => {
    try {
      if (fileIds.length === 1) {
        // Single file download
        window.open(`/api/v1/tenants/${tenantId}/files/${fileIds[0]}/download?token=${localStorage.getItem('token')}`, '_blank');
      } else {
        // Multiple files - create zip
        const response = await fetch(`/api/v1/tenants/${tenantId}/files/bulk-download`, {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('token')}`,
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ fileIds }),
        });

        if (!response.ok) {
          throw new Error('Failed to prepare download');
        }

        const { downloadUrl } = await response.json();
        window.open(downloadUrl, '_blank');
      }
    } catch (error) {
      callbacks.onError?.(error instanceof Error ? error : new Error('Failed to download files'));
    }
  }, [tenantId, callbacks]);

  // View and sort controls
  const setViewMode = useCallback((mode: ViewMode) => {
    setState(prev => ({ ...prev, viewMode: mode }));
  }, []);

  const setSorting = useCallback((field: SortField, direction?: SortDirection) => {
    setState(prev => ({
      ...prev,
      sortField: field,
      sortDirection: direction || (prev.sortField === field && prev.sortDirection === 'asc' ? 'desc' : 'asc'),
    }));
  }, []);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl/Cmd + A - Select all
      if ((e.ctrlKey || e.metaKey) && e.key === 'a') {
        e.preventDefault();
        selectAll();
      }

      // Delete - Delete selected
      if (e.key === 'Delete' && state.selectedFiles.length > 0) {
        e.preventDefault();
        deleteFiles(state.selectedFiles);
      }

      // Escape - Clear selection
      if (e.key === 'Escape') {
        clearSelection();
        setContextMenu(null);
      }

      // Ctrl/Cmd + C - Copy
      if ((e.ctrlKey || e.metaKey) && e.key === 'c' && state.selectedFiles.length > 0) {
        e.preventDefault();
        setState(prev => ({
          ...prev,
          clipboard: { items: prev.selectedFiles, operation: 'copy' },
        }));
      }

      // Ctrl/Cmd + X - Cut
      if ((e.ctrlKey || e.metaKey) && e.key === 'x' && state.selectedFiles.length > 0) {
        e.preventDefault();
        setState(prev => ({
          ...prev,
          clipboard: { items: prev.selectedFiles, operation: 'cut' },
        }));
      }

      // Ctrl/Cmd + V - Paste
      if ((e.ctrlKey || e.metaKey) && e.key === 'v' && state.clipboard.items.length > 0) {
        e.preventDefault();
        if (state.clipboard.operation === 'copy') {
          copyFiles(state.clipboard.items, state.currentFolderId);
        } else if (state.clipboard.operation === 'cut') {
          moveFiles(state.clipboard.items, state.currentFolderId);
          setState(prev => ({ ...prev, clipboard: { items: [], operation: null } }));
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [state.selectedFiles, state.clipboard, state.currentFolderId, selectAll, clearSelection, deleteFiles, copyFiles, moveFiles]);

  // Context menu
  const handleContextMenu = useCallback((e: React.MouseEvent, file: FileItem) => {
    e.preventDefault();
    setContextMenu({ x: e.clientX, y: e.clientY, file });
  }, []);

  useEffect(() => {
    const handleClick = () => setContextMenu(null);
    if (contextMenu) {
      window.addEventListener('click', handleClick);
      return () => window.removeEventListener('click', handleClick);
    }
    return undefined;
  }, [contextMenu]);

  const handleFileOpen = useCallback((file: FileItem) => {
    if (file.type === 'folder') {
      navigateToFolder(file.id, file.name);
    } else {
      setPreviewFile(file);
      setShowPreview(true);
      callbacks.onFileOpen?.(file);
    }
  }, [navigateToFolder, callbacks]);

  return (
    <div className={`caddy-file-manager ${className}`}>
      {/* Toolbar */}
      <div className="file-manager-toolbar">
        <div className="toolbar-left">
          <button onClick={() => setShowUploadPanel(true)} className="btn btn-primary">
            Upload Files
          </button>
          <button onClick={() => createFolder(prompt('Folder name:') || 'New Folder')} className="btn">
            New Folder
          </button>
          {state.selectedFiles.length > 0 && (
            <>
              <button onClick={() => downloadFiles(state.selectedFiles)} className="btn">
                Download ({state.selectedFiles.length})
              </button>
              <button onClick={() => deleteFiles(state.selectedFiles)} className="btn btn-danger">
                Delete ({state.selectedFiles.length})
              </button>
            </>
          )}
        </div>
        <div className="toolbar-right">
          <button
            onClick={() => setViewMode('grid')}
            className={`btn ${state.viewMode === 'grid' ? 'active' : ''}`}
          >
            Grid
          </button>
          <button
            onClick={() => setViewMode('list')}
            className={`btn ${state.viewMode === 'list' ? 'active' : ''}`}
          >
            List
          </button>
        </div>
      </div>

      {/* Breadcrumbs */}
      <div className="file-manager-breadcrumbs">
        {state.breadcrumbs.map((crumb, index) => (
          <React.Fragment key={crumb.id}>
            <button
              onClick={() => navigateToFolder(crumb.id === 'root' ? null : crumb.id)}
              className="breadcrumb-item"
            >
              {crumb.name}
            </button>
            {index < state.breadcrumbs.length - 1 && <span className="breadcrumb-separator">/</span>}
          </React.Fragment>
        ))}
      </div>

      {/* File List */}
      <div ref={fileListRef} className="file-manager-content">
        {state.loading ? (
          <div className="loading-state">Loading files...</div>
        ) : state.error ? (
          <div className="error-state">{state.error}</div>
        ) : state.files.length === 0 ? (
          <div className="empty-state">This folder is empty</div>
        ) : (
          <div className={`file-list file-list-${state.viewMode}`}>
            {state.files.map(file => (
              <div
                key={file.id}
                className={`file-item ${state.selectedFiles.includes(file.id) ? 'selected' : ''}`}
                onClick={(e) => toggleFileSelection(file.id, e.ctrlKey || e.metaKey)}
                onDoubleClick={() => handleFileOpen(file)}
                onContextMenu={(e) => handleContextMenu(e, file)}
              >
                <div className="file-icon">
                  {file.type === 'folder' ? '📁' : '📄'}
                </div>
                <div className="file-name">{file.name}</div>
                {state.viewMode === 'list' && (
                  <>
                    <div className="file-size">{formatBytes(file.size)}</div>
                    <div className="file-modified">{formatDate(file.modifiedAt)}</div>
                  </>
                )}
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Context Menu */}
      {contextMenu && (
        <div
          className="context-menu"
          style={{ left: contextMenu.x, top: contextMenu.y }}
        >
          <button onClick={() => handleFileOpen(contextMenu.file)}>Open</button>
          <button onClick={() => downloadFiles([contextMenu.file.id])}>Download</button>
          <button onClick={() => { setShareFile(contextMenu.file); setShowShareDialog(true); }}>Share</button>
          <button onClick={() => toggleStar(contextMenu.file.id)}>
            {contextMenu.file.isStarred ? 'Unstar' : 'Star'}
          </button>
          <button onClick={() => { setVersionsFile(contextMenu.file); setShowVersions(true); }}>
            Version History
          </button>
          <hr />
          <button onClick={() => renameFile(contextMenu.file.id, prompt('New name:') || contextMenu.file.name)}>
            Rename
          </button>
          <button onClick={() => deleteFiles([contextMenu.file.id])} className="danger">
            Delete
          </button>
        </div>
      )}

      {/* Operations Panel */}
      {operations.filter(op => op.status === 'processing').length > 0 && (
        <div className="operations-panel">
          {operations.filter(op => op.status === 'processing').map(op => (
            <div key={op.id} className="operation-item">
              <div className="operation-title">{op.type} ({op.completed}/{op.total})</div>
              <div className="operation-progress">
                <div className="progress-bar" style={{ width: `${op.progress}%` }} />
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

// Helper functions
function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
}

function formatDate(date: Date): string {
  const d = new Date(date);
  const now = new Date();
  const diffMs = now.getTime() - d.getTime();
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

  if (diffDays === 0) return 'Today';
  if (diffDays === 1) return 'Yesterday';
  if (diffDays < 7) return `${diffDays} days ago`;

  return d.toLocaleDateString();
}

export default FileManager;
