// CADDY - Enterprise CAD System
// TypeScript I/O Type Definitions
// Agent 9 - Import/Export Pipeline Specialist

/**
 * Supported file formats for import/export
 */
export enum FileFormat {
  AUTO_DETECT = 'auto',
  DXF = 'dxf',
  DWG = 'dwg',
  STEP = 'step',
  IGES = 'iges',
  STL = 'stl',
  OBJ = 'obj',
  GLTF = 'gltf',
  CADDY_BINARY = 'cdy',
  CADDY_JSON = 'cdyj',
  SVG = 'svg',
  PDF = 'pdf',
  PNG = 'png',
  JPEG = 'jpeg',
}

/**
 * File format metadata
 */
export interface FormatInfo {
  format: FileFormat;
  name: string;
  extension: string;
  description: string;
  mimeType: string;
  canRead: boolean;
  canWrite: boolean;
  supports3D: boolean;
  supportsMaterials: boolean;
  supportsLayers: boolean;
}

/**
 * Import options
 */
export interface ImportOptions {
  format?: FileFormat;
  validateInput?: boolean;
  repairGeometry?: boolean;
  mergeWithCurrent?: boolean;
  createNewLayer?: boolean;
  layerPrefix?: string;
  scaleFactor?: number;
  unitConversion?: boolean;
  targetUnit?: string;
}

/**
 * Export options
 */
export interface ExportOptions {
  format: FileFormat;
  validateOutput?: boolean;
  includeMetadata?: boolean;
  compression?: boolean;
  compressionLevel?: number;
  precision?: number;
  binaryFormat?: boolean;
  layerSettings?: LayerExportSettings;
}

/**
 * Layer export settings
 */
export interface LayerExportSettings {
  exportAllLayers?: boolean;
  selectedLayers?: string[];
  exportVisible?: boolean;
  flattenLayers?: boolean;
}

/**
 * STL export options
 */
export interface StlExportOptions extends ExportOptions {
  binaryFormat: boolean;
  precision: number;
  calculateNormals?: boolean;
}

/**
 * OBJ export options
 */
export interface ObjExportOptions extends ExportOptions {
  includeMaterials: boolean;
  includeNormals: boolean;
  includeTextureCoords: boolean;
}

/**
 * glTF export options
 */
export interface GltfExportOptions extends ExportOptions {
  binaryFormat: boolean; // GLB vs glTF
  embedTextures: boolean;
  embedBuffers: boolean;
  pbrMaterials: boolean;
}

/**
 * PDF export options
 */
export interface PdfExportOptions extends ExportOptions {
  paperSize: PaperSize;
  orientation: 'portrait' | 'landscape';
  margins: number; // in mm
  scale: number;
  includeLayers: boolean;
  colorMode: 'color' | 'grayscale';
}

/**
 * Paper sizes for PDF export
 */
export enum PaperSize {
  A0 = 'A0',
  A1 = 'A1',
  A2 = 'A2',
  A3 = 'A3',
  A4 = 'A4',
  LETTER = 'Letter',
  LEGAL = 'Legal',
  TABLOID = 'Tabloid',
  CUSTOM = 'Custom',
}

/**
 * Import result
 */
export interface ImportResult {
  success: boolean;
  fileName: string;
  format: FileFormat;
  entityCount: number;
  layerCount: number;
  warnings: string[];
  errors: string[];
  duration: number; // in ms
  fileSize: number; // in bytes
  preview?: PreviewData;
}

/**
 * Export result
 */
export interface ExportResult {
  success: boolean;
  fileName: string;
  format: FileFormat;
  fileSize: number; // in bytes
  duration: number; // in ms
  warnings: string[];
  errors: string[];
}

/**
 * Preview data for imported files
 */
export interface PreviewData {
  thumbnail?: string; // base64 image data
  boundingBox: BoundingBox;
  metadata: Record<string, any>;
}

/**
 * Bounding box
 */
export interface BoundingBox {
  min: Vector3;
  max: Vector3;
}

/**
 * 3D vector
 */
export interface Vector3 {
  x: number;
  y: number;
  z: number;
}

/**
 * Batch conversion job
 */
export interface BatchConversionJob {
  id: string;
  inputFiles: File[];
  outputFormat: FileFormat;
  options: ExportOptions;
  parallel: boolean;
  maxThreads?: number;
}

/**
 * Batch conversion result
 */
export interface BatchConversionResult {
  jobId: string;
  totalFiles: number;
  successful: number;
  failed: number;
  results: ConversionFileResult[];
  totalDuration: number;
  averageDuration: number;
}

/**
 * Single file conversion result
 */
export interface ConversionFileResult {
  inputFile: string;
  outputFile?: string;
  success: boolean;
  error?: string;
  warnings: string[];
  duration: number;
}

/**
 * Validation issue
 */
export interface ValidationIssue {
  severity: 'info' | 'warning' | 'error' | 'critical';
  message: string;
  entityId?: string;
  repairable: boolean;
  location?: string;
}

/**
 * Validation result
 */
export interface ValidationResult {
  valid: boolean;
  issues: ValidationIssue[];
  repairableCount: number;
}

/**
 * File statistics
 */
export interface FileStats {
  fileSize: number;
  entityCount: number;
  layerCount: number;
  blockCount: number;
  operationTime: number;
}

/**
 * Progress callback type
 */
export type ProgressCallback = (current: number, total: number, status: string) => void;

/**
 * Import/Export event types
 */
export enum IOEventType {
  IMPORT_START = 'import:start',
  IMPORT_PROGRESS = 'import:progress',
  IMPORT_COMPLETE = 'import:complete',
  IMPORT_ERROR = 'import:error',
  EXPORT_START = 'export:start',
  EXPORT_PROGRESS = 'export:progress',
  EXPORT_COMPLETE = 'export:complete',
  EXPORT_ERROR = 'export:error',
  BATCH_START = 'batch:start',
  BATCH_PROGRESS = 'batch:progress',
  BATCH_COMPLETE = 'batch:complete',
  VALIDATION_START = 'validation:start',
  VALIDATION_COMPLETE = 'validation:complete',
}

/**
 * I/O event
 */
export interface IOEvent {
  type: IOEventType;
  payload: any;
  timestamp: number;
}

/**
 * Format settings registry
 */
export interface FormatSettingsRegistry {
  [FileFormat.STL]: StlExportOptions;
  [FileFormat.OBJ]: ObjExportOptions;
  [FileFormat.GLTF]: GltfExportOptions;
  [FileFormat.PDF]: PdfExportOptions;
}

/**
 * Default export options for each format
 */
export const DEFAULT_EXPORT_OPTIONS: Partial<Record<FileFormat, ExportOptions>> = {
  [FileFormat.DXF]: {
    format: FileFormat.DXF,
    validateOutput: true,
    includeMetadata: true,
    precision: 6,
  },
  [FileFormat.STL]: {
    format: FileFormat.STL,
    binaryFormat: true,
    precision: 6,
    validateOutput: true,
  },
  [FileFormat.OBJ]: {
    format: FileFormat.OBJ,
    validateOutput: true,
    precision: 6,
  },
  [FileFormat.GLTF]: {
    format: FileFormat.GLTF,
    binaryFormat: false,
    validateOutput: true,
  },
  [FileFormat.PDF]: {
    format: FileFormat.PDF,
    validateOutput: false,
    compression: true,
  },
  [FileFormat.SVG]: {
    format: FileFormat.SVG,
    validateOutput: true,
    precision: 3,
  },
};

/**
 * Format capabilities
 */
export const FORMAT_CAPABILITIES: Record<FileFormat, Partial<FormatInfo>> = {
  [FileFormat.AUTO_DETECT]: {
    name: 'Auto-detect',
    canRead: true,
    canWrite: false,
  },
  [FileFormat.DXF]: {
    name: 'AutoCAD DXF',
    extension: '.dxf',
    mimeType: 'application/dxf',
    canRead: true,
    canWrite: true,
    supports3D: true,
    supportsLayers: true,
  },
  [FileFormat.DWG]: {
    name: 'AutoCAD DWG',
    extension: '.dwg',
    mimeType: 'application/dwg',
    canRead: true,
    canWrite: true,
    supports3D: true,
    supportsLayers: true,
  },
  [FileFormat.STEP]: {
    name: 'STEP/AP214',
    extension: '.step',
    mimeType: 'application/step',
    canRead: true,
    canWrite: true,
    supports3D: true,
  },
  [FileFormat.IGES]: {
    name: 'IGES',
    extension: '.iges',
    mimeType: 'application/iges',
    canRead: true,
    canWrite: true,
    supports3D: true,
  },
  [FileFormat.STL]: {
    name: 'STL',
    extension: '.stl',
    mimeType: 'application/sla',
    canRead: true,
    canWrite: true,
    supports3D: true,
  },
  [FileFormat.OBJ]: {
    name: 'Wavefront OBJ',
    extension: '.obj',
    mimeType: 'model/obj',
    canRead: true,
    canWrite: true,
    supports3D: true,
    supportsMaterials: true,
  },
  [FileFormat.GLTF]: {
    name: 'glTF 2.0',
    extension: '.gltf',
    mimeType: 'model/gltf+json',
    canRead: true,
    canWrite: true,
    supports3D: true,
    supportsMaterials: true,
  },
  [FileFormat.CADDY_BINARY]: {
    name: 'CADDY Binary',
    extension: '.cdy',
    mimeType: 'application/x-caddy',
    canRead: true,
    canWrite: true,
    supports3D: true,
    supportsLayers: true,
  },
  [FileFormat.CADDY_JSON]: {
    name: 'CADDY JSON',
    extension: '.cdyj',
    mimeType: 'application/json',
    canRead: true,
    canWrite: true,
    supports3D: true,
    supportsLayers: true,
  },
  [FileFormat.SVG]: {
    name: 'SVG',
    extension: '.svg',
    mimeType: 'image/svg+xml',
    canRead: true,
    canWrite: true,
    supports3D: false,
  },
  [FileFormat.PDF]: {
    name: 'PDF',
    extension: '.pdf',
    mimeType: 'application/pdf',
    canRead: false,
    canWrite: true,
    supports3D: false,
    supportsLayers: true,
  },
  [FileFormat.PNG]: {
    name: 'PNG',
    extension: '.png',
    mimeType: 'image/png',
    canRead: false,
    canWrite: true,
    supports3D: false,
  },
  [FileFormat.JPEG]: {
    name: 'JPEG',
    extension: '.jpeg',
    mimeType: 'image/jpeg',
    canRead: false,
    canWrite: true,
    supports3D: false,
  },
};
