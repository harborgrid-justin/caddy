export var FileFormat;
(function (FileFormat) {
    FileFormat["AUTO_DETECT"] = "auto";
    FileFormat["DXF"] = "dxf";
    FileFormat["DWG"] = "dwg";
    FileFormat["STEP"] = "step";
    FileFormat["IGES"] = "iges";
    FileFormat["STL"] = "stl";
    FileFormat["OBJ"] = "obj";
    FileFormat["GLTF"] = "gltf";
    FileFormat["CADDY_BINARY"] = "cdy";
    FileFormat["CADDY_JSON"] = "cdyj";
    FileFormat["SVG"] = "svg";
    FileFormat["PDF"] = "pdf";
    FileFormat["PNG"] = "png";
    FileFormat["JPEG"] = "jpeg";
})(FileFormat || (FileFormat = {}));
export var PaperSize;
(function (PaperSize) {
    PaperSize["A0"] = "A0";
    PaperSize["A1"] = "A1";
    PaperSize["A2"] = "A2";
    PaperSize["A3"] = "A3";
    PaperSize["A4"] = "A4";
    PaperSize["LETTER"] = "Letter";
    PaperSize["LEGAL"] = "Legal";
    PaperSize["TABLOID"] = "Tabloid";
    PaperSize["CUSTOM"] = "Custom";
})(PaperSize || (PaperSize = {}));
export var IOEventType;
(function (IOEventType) {
    IOEventType["IMPORT_START"] = "import:start";
    IOEventType["IMPORT_PROGRESS"] = "import:progress";
    IOEventType["IMPORT_COMPLETE"] = "import:complete";
    IOEventType["IMPORT_ERROR"] = "import:error";
    IOEventType["EXPORT_START"] = "export:start";
    IOEventType["EXPORT_PROGRESS"] = "export:progress";
    IOEventType["EXPORT_COMPLETE"] = "export:complete";
    IOEventType["EXPORT_ERROR"] = "export:error";
    IOEventType["BATCH_START"] = "batch:start";
    IOEventType["BATCH_PROGRESS"] = "batch:progress";
    IOEventType["BATCH_COMPLETE"] = "batch:complete";
    IOEventType["VALIDATION_START"] = "validation:start";
    IOEventType["VALIDATION_COMPLETE"] = "validation:complete";
})(IOEventType || (IOEventType = {}));
export const DEFAULT_EXPORT_OPTIONS = {
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
export const FORMAT_CAPABILITIES = {
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
//# sourceMappingURL=types.js.map