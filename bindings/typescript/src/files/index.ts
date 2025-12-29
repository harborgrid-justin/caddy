/**
 * CADDY v0.4.0 - File Management Module
 * Enterprise file management with cloud storage integration
 *
 * @module files
 * @version 0.4.0
 * @description Complete file management system with:
 * - File browser with grid/list views
 * - Chunked upload for large files
 * - File preview (image, PDF, video, documents)
 * - File sharing with permissions and links
 * - Version history and comparison
 * - Full-text search
 * - Storage quota management
 * - Cloud storage integration (S3, GCS, Azure)
 * - Favorites and recent files
 * - Recycle bin with restore
 */

// Main Components
export { FileManager } from './FileManager';
export { FileList } from './FileList';
export { FilePreview } from './FilePreview';
export { FileUpload } from './FileUpload';
export { FileShareComponent as FileShare } from './FileShare';
export { FileVersions } from './FileVersions';
export { FileSearch } from './FileSearch';
export { FileStorage } from './FileStorage';
export { FileCloud } from './FileCloud';
export { FileFavorites } from './FileFavorites';
export { FileRecent } from './FileRecent';
export { FileTrash } from './FileTrash';

// Type Exports
export type {
  // Core Types
  FileType,
  ViewMode,
  SortField,
  SortDirection,
  SharePermission,
  CloudProvider,

  // File and Folder
  FileItem,
  FileMetadata,
  FilePermissions,
  FolderHierarchy,
  BreadcrumbItem,

  // Versioning
  FileVersion,

  // Upload
  UploadTask,

  // Sharing
  ShareLink,
  ShareRecipient,

  // Storage
  StorageQuota,
  StorageBreakdown,
  QuotaLimits,

  // Cloud Sync
  CloudSyncInfo,
  CloudConfig,
  CloudCredentials,
  CloudSettings,
  SyncRule,

  // Search
  SearchQuery,
  SearchFilters,
  SearchResult,

  // Operations
  FileOperation,
  BulkOperationResult,

  // Recent and Activity
  RecentFile,
  FileActivity,

  // State and Config
  FileManagerState,
  FileManagerConfig,
  FileManagerCallbacks,
  PreviewConfig,
  FilePreviewData,

  // Validation
  ValidationResult,
  ValidationError,
} from './types';

// Default Exports
export { default as FileManagerDefault } from './FileManager';
export { default as FileListDefault } from './FileList';
export { default as FilePreviewDefault } from './FilePreview';
export { default as FileUploadDefault } from './FileUpload';
export { default as FileShareDefault } from './FileShare';
export { default as FileVersionsDefault } from './FileVersions';
export { default as FileSearchDefault } from './FileSearch';
export { default as FileStorageDefault } from './FileStorage';
export { default as FileCloudDefault } from './FileCloud';
export { default as FileFavoritesDefault } from './FileFavorites';
export { default as FileRecentDefault } from './FileRecent';
export { default as FileTrashDefault } from './FileTrash';

/**
 * File Management Module
 *
 * @example Basic Usage
 * ```tsx
 * import { FileManager } from '@caddy/files';
 *
 * function App() {
 *   return (
 *     <FileManager
 *       tenantId="tenant-123"
 *       userId="user-456"
 *       config={{
 *         chunkSize: 5 * 1024 * 1024,
 *         maxFileSize: 5 * 1024 * 1024 * 1024,
 *         enableVersioning: true,
 *         enableCloudSync: true,
 *       }}
 *       callbacks={{
 *         onFileSelect: (file) => console.log('Selected:', file),
 *         onFileOpen: (file) => console.log('Opened:', file),
 *         onError: (error) => console.error('Error:', error),
 *       }}
 *     />
 *   );
 * }
 * ```
 *
 * @example File Upload
 * ```tsx
 * import { FileUpload } from '@caddy/files';
 *
 * function UploadDialog() {
 *   return (
 *     <FileUpload
 *       tenantId="tenant-123"
 *       parentId={currentFolderId}
 *       chunkSize={5 * 1024 * 1024}
 *       maxConcurrentUploads={3}
 *       maxFileSize={5 * 1024 * 1024 * 1024}
 *       onUploadComplete={(files) => console.log('Uploaded:', files)}
 *       onClose={() => setShowUpload(false)}
 *     />
 *   );
 * }
 * ```
 *
 * @example File Search
 * ```tsx
 * import { FileSearch } from '@caddy/files';
 *
 * function SearchPage() {
 *   return (
 *     <FileSearch
 *       tenantId="tenant-123"
 *       onFileSelect={(file) => setSelectedFile(file)}
 *       onFileOpen={(file) => openFile(file)}
 *     />
 *   );
 * }
 * ```
 *
 * @example Storage Management
 * ```tsx
 * import { FileStorage } from '@caddy/files';
 *
 * function StoragePage() {
 *   return (
 *     <FileStorage
 *       tenantId="tenant-123"
 *       onUpgrade={() => navigate('/upgrade')}
 *     />
 *   );
 * }
 * ```
 *
 * @example Cloud Integration
 * ```tsx
 * import { FileCloud } from '@caddy/files';
 *
 * function CloudSettings() {
 *   return (
 *     <FileCloud
 *       tenantId="tenant-123"
 *       onClose={() => setShowCloud(false)}
 *     />
 *   );
 * }
 * ```
 */
