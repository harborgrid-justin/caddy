export type ReportType = 'table' | 'chart' | 'pivot' | 'dashboard' | 'custom';
export type ChartType = 'line' | 'bar' | 'pie' | 'scatter' | 'area' | 'heatmap' | 'gauge' | 'funnel' | 'waterfall';
export type AggregationType = 'sum' | 'avg' | 'count' | 'min' | 'max' | 'distinct' | 'median' | 'percentile';
export type FilterOperator = 'eq' | 'ne' | 'gt' | 'gte' | 'lt' | 'lte' | 'in' | 'nin' | 'contains' | 'startsWith' | 'endsWith' | 'between' | 'isNull' | 'isNotNull';
export type DataType = 'string' | 'number' | 'date' | 'boolean' | 'json' | 'array';
export type ExportFormat = 'pdf' | 'excel' | 'csv' | 'json' | 'powerpoint' | 'html';
export type ScheduleFrequency = 'once' | 'hourly' | 'daily' | 'weekly' | 'monthly' | 'quarterly' | 'yearly' | 'cron';
export type DistributionChannel = 'email' | 'slack' | 'teams' | 'webhook' | 's3' | 'ftp' | 'sftp';
export type ReportStatus = 'draft' | 'published' | 'archived' | 'scheduled' | 'running' | 'completed' | 'failed';
export type PermissionLevel = 'view' | 'edit' | 'execute' | 'admin';
export interface DataSource {
    id: string;
    name: string;
    type: 'database' | 'api' | 'file' | 'custom';
    connectionString?: string;
    apiEndpoint?: string;
    authentication?: {
        type: 'none' | 'basic' | 'bearer' | 'oauth2' | 'apiKey';
        credentials?: Record<string, string>;
    };
    schema?: DataSourceSchema;
    cacheConfig?: {
        enabled: boolean;
        ttl: number;
        strategy: 'memory' | 'redis' | 'file';
    };
    metadata?: Record<string, any>;
}
export interface DataSourceSchema {
    tables: Table[];
    relationships: Relationship[];
}
export interface Table {
    name: string;
    displayName?: string;
    fields: Field[];
    primaryKey?: string;
    description?: string;
}
export interface Field {
    name: string;
    displayName?: string;
    dataType: DataType;
    nullable?: boolean;
    description?: string;
    format?: string;
    defaultAggregation?: AggregationType;
    categories?: string[];
}
export interface Relationship {
    name: string;
    sourceTable: string;
    sourceField: string;
    targetTable: string;
    targetField: string;
    type: 'one-to-one' | 'one-to-many' | 'many-to-many';
}
export interface ReportDefinition {
    id: string;
    name: string;
    description?: string;
    type: ReportType;
    version: number;
    status: ReportStatus;
    dataSource: DataSource;
    query: ReportQuery;
    layout: ReportLayout;
    parameters?: ReportParameter[];
    permissions: ReportPermission[];
    metadata: {
        createdBy: string;
        createdAt: Date;
        updatedBy: string;
        updatedAt: Date;
        tags?: string[];
        category?: string;
        folder?: string;
    };
    schedule?: ReportSchedule;
    distribution?: ReportDistribution;
    exportConfig?: ExportConfig;
}
export interface ReportQuery {
    select: SelectField[];
    from: string;
    joins?: Join[];
    where?: FilterGroup;
    groupBy?: string[];
    having?: FilterGroup;
    orderBy?: OrderBy[];
    limit?: number;
    offset?: number;
    rawSql?: string;
}
export interface SelectField {
    field: string;
    alias?: string;
    aggregation?: AggregationType;
    format?: string;
    calculation?: {
        expression: string;
        fields: string[];
    };
}
export interface Join {
    type: 'inner' | 'left' | 'right' | 'full' | 'cross';
    table: string;
    alias?: string;
    on: {
        left: string;
        operator: FilterOperator;
        right: string;
    };
}
export interface FilterGroup {
    operator: 'and' | 'or';
    filters: (Filter | FilterGroup)[];
}
export interface Filter {
    field: string;
    operator: FilterOperator;
    value: any;
    valueType?: DataType;
    caseSensitive?: boolean;
}
export interface OrderBy {
    field: string;
    direction: 'asc' | 'desc';
    nullsFirst?: boolean;
}
export interface ReportLayout {
    type: 'grid' | 'flex' | 'absolute';
    sections: ReportSection[];
    theme?: ReportTheme;
    pageSize?: {
        width: number;
        height: number;
        orientation: 'portrait' | 'landscape';
    };
    margins?: {
        top: number;
        right: number;
        bottom: number;
        left: number;
    };
}
export interface ReportSection {
    id: string;
    type: 'header' | 'footer' | 'body' | 'chart' | 'table' | 'text' | 'image' | 'spacer';
    position: {
        x: number;
        y: number;
        width: number;
        height: number;
    };
    content?: any;
    style?: CSSProperties;
    config?: any;
    visibility?: {
        condition?: string;
        breakAt?: 'page' | 'section' | 'never';
    };
}
export interface ReportTheme {
    name: string;
    colors: {
        primary: string;
        secondary: string;
        background: string;
        text: string;
        border: string;
        chart: string[];
    };
    fonts: {
        title: FontConfig;
        header: FontConfig;
        body: FontConfig;
    };
    spacing: {
        section: number;
        element: number;
    };
}
export interface FontConfig {
    family: string;
    size: number;
    weight: number;
    color: string;
}
export interface ChartConfig {
    type: ChartType;
    dataMapping: {
        xAxis: string[];
        yAxis: string[];
        series?: string;
        value?: string;
    };
    options: {
        title?: string;
        subtitle?: string;
        legend?: {
            show: boolean;
            position: 'top' | 'bottom' | 'left' | 'right';
        };
        tooltip?: {
            enabled: boolean;
            format?: string;
        };
        axis?: {
            x?: AxisConfig;
            y?: AxisConfig;
        };
        colors?: string[];
        stacked?: boolean;
        smooth?: boolean;
        animation?: boolean;
    };
    drillDown?: DrillDownConfig;
}
export interface AxisConfig {
    label?: string;
    type?: 'category' | 'value' | 'time' | 'log';
    min?: number;
    max?: number;
    format?: string;
    grid?: boolean;
}
export interface DrillDownConfig {
    enabled: boolean;
    levels: DrillDownLevel[];
}
export interface DrillDownLevel {
    field: string;
    reportId?: string;
    filters?: Filter[];
}
export interface ReportParameter {
    name: string;
    displayName: string;
    type: DataType;
    required: boolean;
    defaultValue?: any;
    allowedValues?: any[];
    validation?: {
        min?: number;
        max?: number;
        pattern?: string;
        message?: string;
    };
    dependsOn?: string[];
    cascading?: {
        parentParameter: string;
        query: string;
    };
    ui?: {
        inputType: 'text' | 'number' | 'date' | 'select' | 'multiselect' | 'daterange' | 'checkbox';
        placeholder?: string;
        helpText?: string;
    };
}
export interface ReportSchedule {
    id: string;
    enabled: boolean;
    frequency: ScheduleFrequency;
    cronExpression?: string;
    startDate: Date;
    endDate?: Date;
    timezone: string;
    parameters?: Record<string, any>;
    conditions?: {
        dataAvailable?: boolean;
        minimumRows?: number;
        customCondition?: string;
    };
    retryPolicy?: {
        maxAttempts: number;
        retryInterval: number;
        backoffMultiplier: number;
    };
    notifications?: {
        onSuccess?: boolean;
        onFailure?: boolean;
        recipients: string[];
    };
}
export interface ReportDistribution {
    channels: DistributionConfig[];
    attachmentFormat?: ExportFormat;
    compression?: boolean;
    encryption?: {
        enabled: boolean;
        algorithm: 'aes-256' | 'rsa-2048';
        password?: string;
    };
}
export interface DistributionConfig {
    type: DistributionChannel;
    enabled: boolean;
    config: EmailConfig | SlackConfig | TeamsConfig | WebhookConfig | StorageConfig;
}
export interface EmailConfig {
    from: string;
    to: string[];
    cc?: string[];
    bcc?: string[];
    subject: string;
    body: string;
    bodyFormat: 'text' | 'html';
    smtpServer?: string;
    attachReport: boolean;
}
export interface SlackConfig {
    webhookUrl: string;
    channel: string;
    username?: string;
    iconEmoji?: string;
    message: string;
    attachReport: boolean;
}
export interface TeamsConfig {
    webhookUrl: string;
    title: string;
    message: string;
    themeColor?: string;
    attachReport: boolean;
}
export interface WebhookConfig {
    url: string;
    method: 'GET' | 'POST' | 'PUT';
    headers?: Record<string, string>;
    payload?: Record<string, any>;
    includeReportData: boolean;
}
export interface StorageConfig {
    type: 's3' | 'azure' | 'gcs' | 'ftp' | 'sftp';
    bucket?: string;
    path: string;
    credentials?: Record<string, string>;
    publicAccess?: boolean;
}
export interface ExportConfig {
    format: ExportFormat;
    options: PdfOptions | ExcelOptions | CsvOptions | PowerPointOptions;
    fileName?: string;
    watermark?: {
        text: string;
        opacity: number;
        position: 'center' | 'corner';
    };
}
export interface PdfOptions {
    pageSize: 'A4' | 'Letter' | 'Legal' | 'A3' | 'Tabloid';
    orientation: 'portrait' | 'landscape';
    margins: {
        top: number;
        right: number;
        bottom: number;
        left: number;
    };
    includeTableOfContents: boolean;
    includePageNumbers: boolean;
    headerTemplate?: string;
    footerTemplate?: string;
    compression: boolean;
}
export interface ExcelOptions {
    sheetName: string;
    includeCharts: boolean;
    includeFormatting: boolean;
    autoFilterHeaders: boolean;
    freezeHeader: boolean;
    columnWidths?: number[];
    password?: string;
}
export interface CsvOptions {
    delimiter: ',' | ';' | '\t' | '|';
    quote: '"' | "'";
    encoding: 'utf-8' | 'utf-16' | 'iso-8859-1';
    includeHeader: boolean;
    lineEnding: '\n' | '\r\n';
}
export interface PowerPointOptions {
    templateId?: string;
    slideLayout: 'title' | 'content' | 'titleAndContent' | 'blank';
    includeCharts: boolean;
    includeData: boolean;
    theme?: string;
}
export interface ReportPermission {
    userId?: string;
    groupId?: string;
    level: PermissionLevel;
    conditions?: {
        timeRestriction?: {
            start: string;
            end: string;
        };
        ipRestriction?: string[];
        dataFilters?: Filter[];
    };
}
export interface ReportExecution {
    id: string;
    reportId: string;
    status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
    startTime: Date;
    endTime?: Date;
    duration?: number;
    parameters?: Record<string, any>;
    resultMetadata?: {
        rowCount: number;
        columnCount: number;
        dataSize: number;
    };
    error?: {
        code: string;
        message: string;
        stack?: string;
    };
    executedBy: string;
    executionMode: 'interactive' | 'scheduled' | 'api';
}
export interface ReportTemplate {
    id: string;
    name: string;
    description: string;
    category: string;
    thumbnail?: string;
    definition: Partial<ReportDefinition>;
    requiredDataSources: string[];
    tags: string[];
    popularity: number;
    metadata: {
        author: string;
        version: string;
        createdAt: Date;
        updatedAt: Date;
    };
}
export interface ReportVersion {
    version: number;
    reportId: string;
    definition: ReportDefinition;
    createdBy: string;
    createdAt: Date;
    comment?: string;
    changeLog?: VersionChange[];
}
export interface VersionChange {
    field: string;
    oldValue: any;
    newValue: any;
    timestamp: Date;
}
export interface ReportData {
    columns: ColumnMetadata[];
    rows: any[][];
    totalRows: number;
    executionTime: number;
    generatedAt: Date;
    parameters?: Record<string, any>;
}
export interface ColumnMetadata {
    name: string;
    displayName: string;
    dataType: DataType;
    format?: string;
    aggregation?: AggregationType;
}
export interface CSSProperties {
    [key: string]: string | number | undefined;
}
export interface ReportBuilderState {
    definition: ReportDefinition;
    selectedSection?: string;
    previewMode: boolean;
    isDirty: boolean;
    history: {
        past: ReportDefinition[];
        future: ReportDefinition[];
    };
}
export interface ValidationResult {
    valid: boolean;
    errors: ValidationError[];
    warnings: ValidationWarning[];
}
export interface ValidationError {
    field: string;
    message: string;
    code: string;
}
export interface ValidationWarning {
    field: string;
    message: string;
    code: string;
}
//# sourceMappingURL=types.d.ts.map