/**
 * CADDY v0.4.0 - Enterprise File Management Types
 * Complete TypeScript type definitions for file management system
 */

export type FileType = 'file' | 'folder';
export type ViewMode = 'grid' | 'list';
export type SortField = 'name' | 'size' | 'modified' | 'created' | 'type';
export type SortDirection = 'asc' | 'desc';
export type SharePermission = 'view' | 'edit' | 'comment' | 'admin';
export type CloudProvider = 's3' | 'gcs' | 'azure' | 'dropbox' | 'onedrive';

export interface FileItem {
  id: string;
  name: string;
  type: FileType;
  size: number;
  mimeType: string;
  path: string;
  parentId: string | null;
  createdAt: Date;
  modifiedAt: Date;
  createdBy: string;
  modifiedBy: string;
  isStarred: boolean;
  isTrashed: boolean;
  trashedAt?: Date;
  permissions: FilePermissions;
  metadata: FileMetadata;
  thumbnail?: string;
  checksum?: string;
  version: number;
  cloudSync?: CloudSyncInfo;
}

export interface FileMetadata {
  width?: number;
  height?: number;
  duration?: number;
  pages?: number;
  encoding?: string;
  author?: string;
  title?: string;
  description?: string;
  tags: string[];
  customFields: Record<string, any>;
}

export interface FilePermissions {
  canView: boolean;
  canEdit: boolean;
  canDelete: boolean;
  canShare: boolean;
  canDownload: boolean;
  inheritedFrom?: string;
}

export interface FolderHierarchy {
  id: string;
  name: string;
  path: string;
  children: FolderHierarchy[];
  fileCount: number;
  folderCount: number;
  totalSize: number;
}

export interface BreadcrumbItem {
  id: string;
  name: string;
  path: string;
}

export interface FileVersion {
  id: string;
  fileId: string;
  version: number;
  size: number;
  checksum: string;
  createdAt: Date;
  createdBy: string;
  comment?: string;
  url: string;
  isCurrent: boolean;
}

export interface UploadTask {
  id: string;
  file: File;
  name: string;
  size: number;
  uploaded: number;
  status: 'pending' | 'uploading' | 'processing' | 'complete' | 'error' | 'paused' | 'cancelled';
  progress: number;
  speed: number;
  remainingTime: number;
  error?: string;
  url?: string;
  chunkSize: number;
  uploadedChunks: number;
  totalChunks: number;
  retryCount: number;
  startTime: number;
  endTime?: number;
  parentId?: string;
}

export interface ShareLink {
  id: string;
  fileId: string;
  token: string;
  url: string;
  permission: SharePermission;
  expiresAt?: Date;
  password?: string;
  maxDownloads?: number;
  downloadCount: number;
  createdAt: Date;
  createdBy: string;
  isActive: boolean;
  allowedEmails?: string[];
  allowedDomains?: string[];
}

export interface FileShare {
  id: string;
  fileId: string;
  sharedWith: ShareRecipient[];
  links: ShareLink[];
  createdAt: Date;
  updatedAt: Date;
}

export interface ShareRecipient {
  id: string;
  type: 'user' | 'group' | 'team';
  email?: string;
  name: string;
  permission: SharePermission;
  addedAt: Date;
  addedBy: string;
  notified: boolean;
}

export interface StorageQuota {
  total: number;
  used: number;
  available: number;
  percentage: number;
  breakdown: StorageBreakdown;
  plan: string;
  limits: QuotaLimits;
}

export interface StorageBreakdown {
  documents: number;
  images: number;
  videos: number;
  audio: number;
  archives: number;
  other: number;
  trash: number;
}

export interface QuotaLimits {
  maxFileSize: number;
  maxTotalStorage: number;
  maxFilesPerFolder: number;
  maxVersionsPerFile: number;
  allowedFileTypes: string[];
  retentionDays: number;
}

export interface CloudSyncInfo {
  provider: CloudProvider;
  providerFileId: string;
  syncedAt: Date;
  syncStatus: 'synced' | 'syncing' | 'error' | 'conflict';
  lastError?: string;
  autoSync: boolean;
}

export interface CloudConfig {
  provider: CloudProvider;
  enabled: boolean;
  credentials: CloudCredentials;
  settings: CloudSettings;
  syncRules: SyncRule[];
}

export interface CloudCredentials {
  accessKey?: string;
  secretKey?: string;
  bucket?: string;
  region?: string;
  endpoint?: string;
  projectId?: string;
  storageAccount?: string;
  container?: string;
  token?: string;
}

export interface CloudSettings {
  autoSync: boolean;
  syncInterval: number;
  conflictResolution: 'local' | 'remote' | 'newest' | 'manual';
  bandwidth: {
    upload: number;
    download: number;
  };
  encryption: boolean;
}

export interface SyncRule {
  id: string;
  pattern: string;
  action: 'sync' | 'ignore' | 'download-only' | 'upload-only';
  enabled: boolean;
}

export interface SearchQuery {
  query: string;
  filters: SearchFilters;
  sort: {
    field: SortField;
    direction: SortDirection;
  };
  pagination: {
    page: number;
    limit: number;
  };
}

export interface SearchFilters {
  type?: FileType;
  mimeTypes?: string[];
  minSize?: number;
  maxSize?: number;
  createdAfter?: Date;
  createdBefore?: Date;
  modifiedAfter?: Date;
  modifiedBefore?: Date;
  createdBy?: string[];
  tags?: string[];
  parentId?: string;
  isStarred?: boolean;
  isTrashed?: boolean;
}

export interface SearchResult {
  items: FileItem[];
  total: number;
  page: number;
  totalPages: number;
  took: number;
  highlights?: Record<string, string[]>;
}

export interface FileOperation {
  id: string;
  type: 'copy' | 'move' | 'delete' | 'rename' | 'restore';
  fileIds: string[];
  status: 'pending' | 'processing' | 'complete' | 'error';
  progress: number;
  total: number;
  completed: number;
  failed: number;
  error?: string;
  destination?: string;
  createdAt: Date;
  completedAt?: Date;
}

export interface RecentFile {
  file: FileItem;
  accessedAt: Date;
  accessType: 'view' | 'edit' | 'download' | 'share';
  accessCount: number;
}

export interface FileActivity {
  id: string;
  fileId: string;
  userId: string;
  userName: string;
  action: 'create' | 'update' | 'delete' | 'rename' | 'move' | 'share' | 'download' | 'restore';
  details: Record<string, any>;
  timestamp: Date;
  ipAddress: string;
  userAgent: string;
}

export interface FileManagerState {
  currentPath: string;
  currentFolderId: string | null;
  files: FileItem[];
  selectedFiles: string[];
  viewMode: ViewMode;
  sortField: SortField;
  sortDirection: SortDirection;
  loading: boolean;
  error: string | null;
  breadcrumbs: BreadcrumbItem[];
  clipboard: {
    items: string[];
    operation: 'copy' | 'cut' | null;
  };
}

export interface FileManagerConfig {
  chunkSize: number;
  maxConcurrentUploads: number;
  maxRetries: number;
  allowedFileTypes: string[];
  maxFileSize: number;
  thumbnailSize: number;
  previewFormats: {
    images: string[];
    videos: string[];
    documents: string[];
    audio: string[];
  };
  enableVersioning: boolean;
  enableCloudSync: boolean;
  enableFullTextSearch: boolean;
  trashRetentionDays: number;
}

export interface FileManagerCallbacks {
  onFileSelect?: (file: FileItem) => void;
  onFileOpen?: (file: FileItem) => void;
  onFileUpload?: (files: FileItem[]) => void;
  onFileDelete?: (fileIds: string[]) => void;
  onFolderChange?: (folderId: string | null) => void;
  onError?: (error: Error) => void;
}

export interface PreviewConfig {
  maxSize: number;
  supportedTypes: string[];
  enableAnnotations: boolean;
  enableComments: boolean;
  enableDownload: boolean;
  enablePrint: boolean;
  enableShare: boolean;
}

export interface FilePreviewData {
  file: FileItem;
  url: string;
  type: 'image' | 'video' | 'audio' | 'pdf' | 'document' | 'code' | 'text' | 'unsupported';
  content?: string;
  pages?: number;
  currentPage?: number;
}

export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
}

export interface ValidationError {
  field: string;
  message: string;
  code: string;
}

export interface BulkOperationResult {
  success: string[];
  failed: Array<{
    id: string;
    error: string;
  }>;
  total: number;
  successCount: number;
  failedCount: number;
}
