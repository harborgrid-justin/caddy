/**
 * CADDY v0.4.0 - Reporting System Module
 * $650M Platform - Production Ready
 *
 * Comprehensive enterprise reporting system with report builder, viewer,
 * scheduling, distribution, and multi-format export capabilities.
 *
 * @module reporting
 */

// Type exports
export type {
  // Core types
  ReportType,
  ChartType,
  AggregationType,
  FilterOperator,
  DataType,
  ExportFormat,
  ScheduleFrequency,
  DistributionChannel,
  ReportStatus,
  PermissionLevel,

  // Data source types
  DataSource,
  DataSourceSchema,
  Table,
  Field,
  Relationship,

  // Report definition types
  ReportDefinition,
  ReportQuery,
  SelectField,
  Join,
  FilterGroup,
  Filter,
  OrderBy,

  // Layout types
  ReportLayout,
  ReportSection,
  ReportTheme,
  FontConfig,
  CSSProperties,

  // Chart types
  ChartConfig,
  AxisConfig,
  DrillDownConfig,
  DrillDownLevel,

  // Parameter types
  ReportParameter,

  // Schedule types
  ReportSchedule,

  // Distribution types
  ReportDistribution as ReportDistributionData,
  DistributionConfig,
  EmailConfig,
  SlackConfig,
  TeamsConfig,
  WebhookConfig,
  StorageConfig,

  // Export types
  ExportConfig,
  PdfOptions,
  ExcelOptions,
  CsvOptions,
  PowerPointOptions,

  // Permission types
  ReportPermission,

  // Execution types
  ReportExecution,

  // Template types
  ReportTemplate,

  // Version types
  ReportVersion,
  VersionChange,

  // Data types
  ReportData,
  ColumnMetadata,

  // State types
  ReportBuilderState,

  // Validation types
  ValidationResult,
  ValidationError,
  ValidationWarning,
} from './types';

// Component exports
import { ReportBuilder as RB } from './ReportBuilder';
import { ReportViewer as RV } from './ReportViewer';
import { ReportDataSource as RDS } from './ReportDataSource';
import { ReportFields as RF } from './ReportFields';
import { ReportFilters as RFilt } from './ReportFilters';
import { ReportCharts as RC } from './ReportCharts';
import { ReportScheduler as RS } from './ReportScheduler';
import { ReportDistributionComponent as RD } from './ReportDistribution';
import { ReportTemplates as RT } from './ReportTemplates';
import { ReportExport as RE } from './ReportExport';
import { ReportDashboard as RDash } from './ReportDashboard';

export { RB as ReportBuilder };
export type { ReportBuilderProps } from './ReportBuilder';

export { RV as ReportViewer };
export type { ReportViewerProps } from './ReportViewer';

export { RDS as ReportDataSource };
export type { ReportDataSourceProps } from './ReportDataSource';

export { RF as ReportFields };
export type { ReportFieldsProps } from './ReportFields';

export { RFilt as ReportFilters };
export type { ReportFiltersProps } from './ReportFilters';

export { RC as ReportCharts };
export type { ReportChartsProps } from './ReportCharts';

export { RS as ReportScheduler };
export type { ReportSchedulerProps } from './ReportScheduler';

export { RD as ReportDistribution };
export type { ReportDistributionProps } from './ReportDistribution';

export { RT as ReportTemplates };
export type { ReportTemplatesProps } from './ReportTemplates';

export { RE as ReportExport };
export type { ReportExportProps } from './ReportExport';

export { RDash as ReportDashboard };
export type { ReportDashboardProps } from './ReportDashboard';

// Default export for convenience
export default {
  ReportBuilder: RB,
  ReportViewer: RV,
  ReportDataSource: RDS,
  ReportFields: RF,
  ReportFilters: RFilt,
  ReportCharts: RC,
  ReportScheduler: RS,
  ReportDistribution: RD,
  ReportTemplates: RT,
  ReportExport: RE,
  ReportDashboard: RDash,
};

/**
 * @example
 * // Import specific components
 * import { ReportBuilder, ReportViewer } from '@caddy/reporting';
 *
 * @example
 * // Import all components
 * import Reporting from '@caddy/reporting';
 * const { ReportBuilder, ReportViewer } = Reporting;
 *
 * @example
 * // Import types
 * import type { ReportDefinition, ReportData } from '@caddy/reporting';
 */
