export declare enum FileFormat {
    AUTO_DETECT = "auto",
    DXF = "dxf",
    DWG = "dwg",
    STEP = "step",
    IGES = "iges",
    STL = "stl",
    OBJ = "obj",
    GLTF = "gltf",
    CADDY_BINARY = "cdy",
    CADDY_JSON = "cdyj",
    SVG = "svg",
    PDF = "pdf",
    PNG = "png",
    JPEG = "jpeg"
}
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
export interface LayerExportSettings {
    exportAllLayers?: boolean;
    selectedLayers?: string[];
    exportVisible?: boolean;
    flattenLayers?: boolean;
}
export interface StlExportOptions extends ExportOptions {
    binaryFormat: boolean;
    precision: number;
    calculateNormals?: boolean;
}
export interface ObjExportOptions extends ExportOptions {
    includeMaterials: boolean;
    includeNormals: boolean;
    includeTextureCoords: boolean;
}
export interface GltfExportOptions extends ExportOptions {
    binaryFormat: boolean;
    embedTextures: boolean;
    embedBuffers: boolean;
    pbrMaterials: boolean;
}
export interface PdfExportOptions extends ExportOptions {
    paperSize: PaperSize;
    orientation: 'portrait' | 'landscape';
    margins: number;
    scale: number;
    includeLayers: boolean;
    colorMode: 'color' | 'grayscale';
}
export declare enum PaperSize {
    A0 = "A0",
    A1 = "A1",
    A2 = "A2",
    A3 = "A3",
    A4 = "A4",
    LETTER = "Letter",
    LEGAL = "Legal",
    TABLOID = "Tabloid",
    CUSTOM = "Custom"
}
export interface ImportResult {
    success: boolean;
    fileName: string;
    format: FileFormat;
    entityCount: number;
    layerCount: number;
    warnings: string[];
    errors: string[];
    duration: number;
    fileSize: number;
    preview?: PreviewData;
}
export interface ExportResult {
    success: boolean;
    fileName: string;
    format: FileFormat;
    fileSize: number;
    duration: number;
    warnings: string[];
    errors: string[];
}
export interface PreviewData {
    thumbnail?: string;
    boundingBox: BoundingBox;
    metadata: Record<string, any>;
}
export interface BoundingBox {
    min: Vector3;
    max: Vector3;
}
export interface Vector3 {
    x: number;
    y: number;
    z: number;
}
export interface BatchConversionJob {
    id: string;
    inputFiles: File[];
    outputFormat: FileFormat;
    options: ExportOptions;
    parallel: boolean;
    maxThreads?: number;
}
export interface BatchConversionResult {
    jobId: string;
    totalFiles: number;
    successful: number;
    failed: number;
    results: ConversionFileResult[];
    totalDuration: number;
    averageDuration: number;
}
export interface ConversionFileResult {
    inputFile: string;
    outputFile?: string;
    success: boolean;
    error?: string;
    warnings: string[];
    duration: number;
}
export interface ValidationIssue {
    severity: 'info' | 'warning' | 'error' | 'critical';
    message: string;
    entityId?: string;
    repairable: boolean;
    location?: string;
}
export interface ValidationResult {
    valid: boolean;
    issues: ValidationIssue[];
    repairableCount: number;
}
export interface FileStats {
    fileSize: number;
    entityCount: number;
    layerCount: number;
    blockCount: number;
    operationTime: number;
}
export type ProgressCallback = (current: number, total: number, status: string) => void;
export declare enum IOEventType {
    IMPORT_START = "import:start",
    IMPORT_PROGRESS = "import:progress",
    IMPORT_COMPLETE = "import:complete",
    IMPORT_ERROR = "import:error",
    EXPORT_START = "export:start",
    EXPORT_PROGRESS = "export:progress",
    EXPORT_COMPLETE = "export:complete",
    EXPORT_ERROR = "export:error",
    BATCH_START = "batch:start",
    BATCH_PROGRESS = "batch:progress",
    BATCH_COMPLETE = "batch:complete",
    VALIDATION_START = "validation:start",
    VALIDATION_COMPLETE = "validation:complete"
}
export interface IOEvent {
    type: IOEventType;
    payload: any;
    timestamp: number;
}
export interface FormatSettingsRegistry {
    [FileFormat.STL]: StlExportOptions;
    [FileFormat.OBJ]: ObjExportOptions;
    [FileFormat.GLTF]: GltfExportOptions;
    [FileFormat.PDF]: PdfExportOptions;
}
export declare const DEFAULT_EXPORT_OPTIONS: Partial<Record<FileFormat, ExportOptions>>;
export declare const FORMAT_CAPABILITIES: Record<FileFormat, Partial<FormatInfo>>;
//# sourceMappingURL=types.d.ts.map