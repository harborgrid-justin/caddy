/**
 * CADDY v0.4.0 - Report Export Component
 * $650M Platform - Production Ready
 *
 * Multi-format export with PDF, Excel, CSV, PowerPoint support,
 * advanced formatting options, and watermarking capabilities.
 */

import React, { useState, useCallback } from 'react';
import {
  ExportConfig,
  ExportFormat,
  PdfOptions,
  ExcelOptions,
  CsvOptions,
  PowerPointOptions,
  ReportData,
} from './types';

export interface ReportExportProps {
  reportData?: ReportData;
  onExport: (config: ExportConfig) => Promise<void>;
  showPreview?: boolean;
}

export const ReportExport: React.FC<ReportExportProps> = ({
  reportData,
  onExport,
  showPreview = true,
}) => {
  const [format, setFormat] = useState<ExportFormat>('pdf');
  const [config, setConfig] = useState<ExportConfig>(createDefaultConfig('pdf'));
  const [exporting, setExporting] = useState(false);
  const [progress, setProgress] = useState(0);

  const handleFormatChange = useCallback((newFormat: ExportFormat) => {
    setFormat(newFormat);
    setConfig(createDefaultConfig(newFormat));
  }, []);

  const updateOptions = useCallback(
    (updates: Partial<PdfOptions | ExcelOptions | CsvOptions | PowerPointOptions>) => {
      setConfig((prev) => ({
        ...prev,
        options: { ...prev.options, ...updates },
      }));
    },
    []
  );

  const updateConfig = useCallback((updates: Partial<ExportConfig>) => {
    setConfig((prev) => ({ ...prev, ...updates }));
  }, []);

  const handleExport = useCallback(async () => {
    setExporting(true);
    setProgress(0);

    try {
      // Simulate progress
      const progressInterval = setInterval(() => {
        setProgress((prev) => Math.min(prev + 10, 90));
      }, 200);

      await onExport(config);

      clearInterval(progressInterval);
      setProgress(100);

      setTimeout(() => {
        setExporting(false);
        setProgress(0);
      }, 1000);
    } catch (error) {
      console.error('Export failed:', error);
      alert('Export failed. Please try again.');
      setExporting(false);
      setProgress(0);
    }
  }, [config, onExport]);

  const renderPdfOptions = () => {
    const options = config.options as PdfOptions;

    return (
      <div style={styles.optionsPanel}>
        <div style={styles.formGroup}>
          <label style={styles.label}>Page Size</label>
          <select
            value={options.pageSize}
            onChange={(e) =>
              updateOptions({ pageSize: e.target.value as PdfOptions['pageSize'] })
            }
            style={styles.select}
          >
            <option value="A4">A4</option>
            <option value="Letter">Letter</option>
            <option value="Legal">Legal</option>
            <option value="A3">A3</option>
            <option value="Tabloid">Tabloid</option>
          </select>
        </div>

        <div style={styles.formGroup}>
          <label style={styles.label}>Orientation</label>
          <select
            value={options.orientation}
            onChange={(e) =>
              updateOptions({ orientation: e.target.value as 'portrait' | 'landscape' })
            }
            style={styles.select}
          >
            <option value="portrait">Portrait</option>
            <option value="landscape">Landscape</option>
          </select>
        </div>

        <div style={styles.formGroup}>
          <h4 style={styles.subsectionTitle}>Margins (mm)</h4>
          <div style={styles.marginsGrid}>
            <div>
              <label style={styles.smallLabel}>Top</label>
              <input
                type="number"
                value={options.margins.top}
                onChange={(e) =>
                  updateOptions({
                    margins: { ...options.margins, top: Number(e.target.value) },
                  })
                }
                style={styles.smallInput}
              />
            </div>
            <div>
              <label style={styles.smallLabel}>Right</label>
              <input
                type="number"
                value={options.margins.right}
                onChange={(e) =>
                  updateOptions({
                    margins: { ...options.margins, right: Number(e.target.value) },
                  })
                }
                style={styles.smallInput}
              />
            </div>
            <div>
              <label style={styles.smallLabel}>Bottom</label>
              <input
                type="number"
                value={options.margins.bottom}
                onChange={(e) =>
                  updateOptions({
                    margins: { ...options.margins, bottom: Number(e.target.value) },
                  })
                }
                style={styles.smallInput}
              />
            </div>
            <div>
              <label style={styles.smallLabel}>Left</label>
              <input
                type="number"
                value={options.margins.left}
                onChange={(e) =>
                  updateOptions({
                    margins: { ...options.margins, left: Number(e.target.value) },
                  })
                }
                style={styles.smallInput}
              />
            </div>
          </div>
        </div>

        <div style={styles.formGroup}>
          <label style={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={options.includeTableOfContents}
              onChange={(e) => updateOptions({ includeTableOfContents: e.target.checked })}
            />
            <span>Include table of contents</span>
          </label>
        </div>

        <div style={styles.formGroup}>
          <label style={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={options.includePageNumbers}
              onChange={(e) => updateOptions({ includePageNumbers: e.target.checked })}
            />
            <span>Include page numbers</span>
          </label>
        </div>

        <div style={styles.formGroup}>
          <label style={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={options.compression}
              onChange={(e) => updateOptions({ compression: e.target.checked })}
            />
            <span>Enable compression</span>
          </label>
        </div>
      </div>
    );
  };

  const renderExcelOptions = () => {
    const options = config.options as ExcelOptions;

    return (
      <div style={styles.optionsPanel}>
        <div style={styles.formGroup}>
          <label style={styles.label}>Sheet Name</label>
          <input
            type="text"
            value={options.sheetName}
            onChange={(e) => updateOptions({ sheetName: e.target.value })}
            style={styles.input}
            placeholder="Sheet1"
          />
        </div>

        <div style={styles.formGroup}>
          <label style={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={options.includeCharts}
              onChange={(e) => updateOptions({ includeCharts: e.target.checked })}
            />
            <span>Include charts</span>
          </label>
        </div>

        <div style={styles.formGroup}>
          <label style={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={options.includeFormatting}
              onChange={(e) => updateOptions({ includeFormatting: e.target.checked })}
            />
            <span>Include formatting</span>
          </label>
        </div>

        <div style={styles.formGroup}>
          <label style={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={options.autoFilterHeaders}
              onChange={(e) => updateOptions({ autoFilterHeaders: e.target.checked })}
            />
            <span>Auto-filter headers</span>
          </label>
        </div>

        <div style={styles.formGroup}>
          <label style={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={options.freezeHeader}
              onChange={(e) => updateOptions({ freezeHeader: e.target.checked })}
            />
            <span>Freeze header row</span>
          </label>
        </div>

        <div style={styles.formGroup}>
          <label style={styles.label}>Password (optional)</label>
          <input
            type="password"
            value={options.password || ''}
            onChange={(e) => updateOptions({ password: e.target.value })}
            style={styles.input}
            placeholder="Leave empty for no password"
          />
        </div>
      </div>
    );
  };

  const renderCsvOptions = () => {
    const options = config.options as CsvOptions;

    return (
      <div style={styles.optionsPanel}>
        <div style={styles.formGroup}>
          <label style={styles.label}>Delimiter</label>
          <select
            value={options.delimiter}
            onChange={(e) =>
              updateOptions({ delimiter: e.target.value as CsvOptions['delimiter'] })
            }
            style={styles.select}
          >
            <option value=",">Comma (,)</option>
            <option value=";">Semicolon (;)</option>
            <option value="\t">Tab</option>
            <option value="|">Pipe (|)</option>
          </select>
        </div>

        <div style={styles.formGroup}>
          <label style={styles.label}>Quote Character</label>
          <select
            value={options.quote}
            onChange={(e) => updateOptions({ quote: e.target.value as '"' | "'" })}
            style={styles.select}
          >
            <option value='"'>Double Quote (")</option>
            <option value="'">Single Quote (')</option>
          </select>
        </div>

        <div style={styles.formGroup}>
          <label style={styles.label}>Encoding</label>
          <select
            value={options.encoding}
            onChange={(e) =>
              updateOptions({ encoding: e.target.value as CsvOptions['encoding'] })
            }
            style={styles.select}
          >
            <option value="utf-8">UTF-8</option>
            <option value="utf-16">UTF-16</option>
            <option value="iso-8859-1">ISO-8859-1</option>
          </select>
        </div>

        <div style={styles.formGroup}>
          <label style={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={options.includeHeader}
              onChange={(e) => updateOptions({ includeHeader: e.target.checked })}
            />
            <span>Include header row</span>
          </label>
        </div>

        <div style={styles.formGroup}>
          <label style={styles.label}>Line Ending</label>
          <select
            value={options.lineEnding}
            onChange={(e) => updateOptions({ lineEnding: e.target.value as '\n' | '\r\n' })}
            style={styles.select}
          >
            <option value="\n">LF (\n) - Unix/Mac</option>
            <option value="\r\n">CRLF (\r\n) - Windows</option>
          </select>
        </div>
      </div>
    );
  };

  const renderPowerPointOptions = () => {
    const options = config.options as PowerPointOptions;

    return (
      <div style={styles.optionsPanel}>
        <div style={styles.formGroup}>
          <label style={styles.label}>Slide Layout</label>
          <select
            value={options.slideLayout}
            onChange={(e) =>
              updateOptions({
                slideLayout: e.target.value as PowerPointOptions['slideLayout'],
              })
            }
            style={styles.select}
          >
            <option value="title">Title Slide</option>
            <option value="content">Content</option>
            <option value="titleAndContent">Title and Content</option>
            <option value="blank">Blank</option>
          </select>
        </div>

        <div style={styles.formGroup}>
          <label style={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={options.includeCharts}
              onChange={(e) => updateOptions({ includeCharts: e.target.checked })}
            />
            <span>Include charts</span>
          </label>
        </div>

        <div style={styles.formGroup}>
          <label style={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={options.includeData}
              onChange={(e) => updateOptions({ includeData: e.target.checked })}
            />
            <span>Include data tables</span>
          </label>
        </div>

        <div style={styles.formGroup}>
          <label style={styles.label}>Theme (optional)</label>
          <input
            type="text"
            value={options.theme || ''}
            onChange={(e) => updateOptions({ theme: e.target.value })}
            style={styles.input}
            placeholder="Default"
          />
        </div>
      </div>
    );
  };

  const renderOptions = () => {
    switch (format) {
      case 'pdf':
        return renderPdfOptions();
      case 'excel':
        return renderExcelOptions();
      case 'csv':
        return renderCsvOptions();
      case 'powerpoint':
        return renderPowerPointOptions();
      default:
        return <div>JSON export has no additional options</div>;
    }
  };

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h3 style={styles.title}>Export Configuration</h3>
      </div>

      <div style={styles.content}>
        {/* Format Selection */}
        <div style={styles.section}>
          <h4 style={styles.sectionTitle}>Export Format</h4>
          <div style={styles.formatGrid}>
            {exportFormats.map((fmt) => (
              <button
                key={fmt.value}
                onClick={() => handleFormatChange(fmt.value)}
                style={{
                  ...styles.formatButton,
                  ...(format === fmt.value ? styles.formatButtonActive : {}),
                }}
              >
                <span style={styles.formatIcon}>{fmt.icon}</span>
                <span style={styles.formatLabel}>{fmt.label}</span>
              </button>
            ))}
          </div>
        </div>

        {/* Format Options */}
        <div style={styles.section}>
          <h4 style={styles.sectionTitle}>{format.toUpperCase()} Options</h4>
          {renderOptions()}
        </div>

        {/* General Options */}
        <div style={styles.section}>
          <h4 style={styles.sectionTitle}>General Options</h4>

          <div style={styles.formGroup}>
            <label style={styles.label}>File Name</label>
            <input
              type="text"
              value={config.fileName || ''}
              onChange={(e) => updateConfig({ fileName: e.target.value })}
              style={styles.input}
              placeholder="report-{{date}}"
            />
          </div>

          <div style={styles.formGroup}>
            <label style={styles.checkboxLabel}>
              <input
                type="checkbox"
                checked={!!config.watermark}
                onChange={(e) =>
                  updateConfig({
                    watermark: e.target.checked
                      ? { text: 'CONFIDENTIAL', opacity: 0.3, position: 'center' }
                      : undefined,
                  })
                }
              />
              <span>Add watermark</span>
            </label>
          </div>

          {config.watermark && (
            <>
              <div style={styles.formGroup}>
                <label style={styles.label}>Watermark Text</label>
                <input
                  type="text"
                  value={config.watermark.text}
                  onChange={(e) =>
                    updateConfig({
                      watermark: { ...config.watermark!, text: e.target.value },
                    })
                  }
                  style={styles.input}
                />
              </div>

              <div style={styles.formGroup}>
                <label style={styles.label}>Opacity</label>
                <input
                  type="range"
                  min="0"
                  max="1"
                  step="0.1"
                  value={config.watermark.opacity}
                  onChange={(e) =>
                    updateConfig({
                      watermark: {
                        ...config.watermark!,
                        opacity: Number(e.target.value),
                      },
                    })
                  }
                  style={styles.slider}
                />
                <span style={styles.sliderValue}>{config.watermark.opacity}</span>
              </div>

              <div style={styles.formGroup}>
                <label style={styles.label}>Position</label>
                <select
                  value={config.watermark.position}
                  onChange={(e) =>
                    updateConfig({
                      watermark: {
                        ...config.watermark!,
                        position: e.target.value as 'center' | 'corner',
                      },
                    })
                  }
                  style={styles.select}
                >
                  <option value="center">Center</option>
                  <option value="corner">Corner</option>
                </select>
              </div>
            </>
          )}
        </div>

        {/* Data Summary */}
        {reportData && (
          <div style={styles.section}>
            <h4 style={styles.sectionTitle}>Export Summary</h4>
            <div style={styles.summary}>
              <div style={styles.summaryItem}>
                <span style={styles.summaryLabel}>Rows:</span>
                <span>{reportData.totalRows.toLocaleString()}</span>
              </div>
              <div style={styles.summaryItem}>
                <span style={styles.summaryLabel}>Columns:</span>
                <span>{reportData.columns.length}</span>
              </div>
              <div style={styles.summaryItem}>
                <span style={styles.summaryLabel}>Format:</span>
                <span>{format.toUpperCase()}</span>
              </div>
              <div style={styles.summaryItem}>
                <span style={styles.summaryLabel}>Estimated Size:</span>
                <span>{estimateFileSize(reportData, format)}</span>
              </div>
            </div>
          </div>
        )}

        {/* Export Button */}
        <div style={styles.exportSection}>
          <button
            onClick={handleExport}
            disabled={exporting}
            style={{
              ...styles.exportButton,
              ...(exporting ? styles.exportButtonDisabled : {}),
            }}
          >
            {exporting ? `Exporting... ${progress}%` : `Export as ${format.toUpperCase()}`}
          </button>

          {exporting && (
            <div style={styles.progressBar}>
              <div style={{ ...styles.progressFill, width: `${progress}%` }} />
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

// Helper functions
function createDefaultConfig(format: ExportFormat): ExportConfig {
  let options: any;

  switch (format) {
    case 'pdf':
      options = {
        pageSize: 'A4',
        orientation: 'portrait',
        margins: { top: 20, right: 20, bottom: 20, left: 20 },
        includeTableOfContents: false,
        includePageNumbers: true,
        compression: true,
      } as PdfOptions;
      break;

    case 'excel':
      options = {
        sheetName: 'Report',
        includeCharts: true,
        includeFormatting: true,
        autoFilterHeaders: true,
        freezeHeader: true,
      } as ExcelOptions;
      break;

    case 'csv':
      options = {
        delimiter: ',',
        quote: '"',
        encoding: 'utf-8',
        includeHeader: true,
        lineEnding: '\n',
      } as CsvOptions;
      break;

    case 'powerpoint':
      options = {
        slideLayout: 'titleAndContent',
        includeCharts: true,
        includeData: true,
      } as PowerPointOptions;
      break;

    default:
      options = {};
  }

  return {
    format,
    options,
  };
}

function estimateFileSize(data: ReportData, format: ExportFormat): string {
  const baseSize = data.totalRows * data.columns.length * 50; // rough estimate

  let multiplier = 1;
  switch (format) {
    case 'pdf':
      multiplier = 2;
      break;
    case 'excel':
      multiplier = 1.5;
      break;
    case 'csv':
      multiplier = 0.5;
      break;
    case 'powerpoint':
      multiplier = 3;
      break;
    case 'json':
      multiplier = 1.2;
      break;
  }

  const sizeInBytes = baseSize * multiplier;
  const sizeInKB = sizeInBytes / 1024;
  const sizeInMB = sizeInKB / 1024;

  if (sizeInMB > 1) {
    return `${sizeInMB.toFixed(2)} MB`;
  } else {
    return `${sizeInKB.toFixed(2)} KB`;
  }
}

// Export formats configuration
const exportFormats: Array<{ value: ExportFormat; label: string; icon: string }> = [
  { value: 'pdf', label: 'PDF', icon: '📄' },
  { value: 'excel', label: 'Excel', icon: '📊' },
  { value: 'csv', label: 'CSV', icon: '📝' },
  { value: 'powerpoint', label: 'PowerPoint', icon: '📽️' },
  { value: 'json', label: 'JSON', icon: '{}' },
];

// Styles
const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    height: '100%',
    backgroundColor: '#ffffff',
    border: '1px solid #e2e8f0',
    borderRadius: '8px',
    fontFamily: 'Inter, system-ui, sans-serif',
    overflow: 'hidden',
  },
  header: {
    padding: '12px 16px',
    borderBottom: '1px solid #e2e8f0',
    backgroundColor: '#f8fafc',
  },
  title: {
    fontSize: '14px',
    fontWeight: 600,
    margin: 0,
    color: '#1e293b',
  },
  content: {
    flex: 1,
    overflow: 'auto',
    padding: '16px',
  },
  section: {
    marginBottom: '24px',
    paddingBottom: '24px',
    borderBottom: '1px solid #e2e8f0',
  },
  sectionTitle: {
    fontSize: '13px',
    fontWeight: 600,
    margin: '0 0 12px 0',
    color: '#1e293b',
  },
  formatGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(120px, 1fr))',
    gap: '8px',
  },
  formatButton: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    gap: '6px',
    padding: '16px',
    border: '1px solid #e2e8f0',
    borderRadius: '6px',
    backgroundColor: '#ffffff',
    cursor: 'pointer',
    transition: 'all 0.2s',
  },
  formatButtonActive: {
    borderColor: '#2563eb',
    backgroundColor: '#eff6ff',
  },
  formatIcon: {
    fontSize: '32px',
  },
  formatLabel: {
    fontSize: '13px',
    fontWeight: 500,
    color: '#475569',
  },
  optionsPanel: {
    display: 'flex',
    flexDirection: 'column',
    gap: '12px',
  },
  formGroup: {
    marginBottom: '12px',
  },
  label: {
    display: 'block',
    fontSize: '12px',
    fontWeight: 500,
    color: '#475569',
    marginBottom: '4px',
  },
  input: {
    width: '100%',
    padding: '6px 8px',
    border: '1px solid #e2e8f0',
    borderRadius: '4px',
    fontSize: '13px',
  },
  select: {
    width: '100%',
    padding: '6px 8px',
    border: '1px solid #e2e8f0',
    borderRadius: '4px',
    fontSize: '13px',
    cursor: 'pointer',
  },
  checkboxLabel: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    fontSize: '13px',
    color: '#475569',
    cursor: 'pointer',
  },
  subsectionTitle: {
    fontSize: '12px',
    fontWeight: 600,
    margin: '0 0 8px 0',
    color: '#64748b',
  },
  marginsGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(4, 1fr)',
    gap: '8px',
  },
  smallLabel: {
    display: 'block',
    fontSize: '11px',
    fontWeight: 500,
    color: '#64748b',
    marginBottom: '2px',
  },
  smallInput: {
    width: '100%',
    padding: '4px 6px',
    border: '1px solid #e2e8f0',
    borderRadius: '4px',
    fontSize: '12px',
  },
  slider: {
    width: 'calc(100% - 50px)',
    marginRight: '8px',
  },
  sliderValue: {
    fontSize: '12px',
    color: '#64748b',
    fontWeight: 500,
  },
  summary: {
    backgroundColor: '#f8fafc',
    border: '1px solid #e2e8f0',
    borderRadius: '6px',
    padding: '12px',
  },
  summaryItem: {
    display: 'flex',
    justifyContent: 'space-between',
    padding: '6px 0',
    fontSize: '13px',
    borderBottom: '1px solid #e2e8f0',
  },
  summaryLabel: {
    fontWeight: 600,
    color: '#475569',
  },
  exportSection: {
    marginTop: '24px',
  },
  exportButton: {
    width: '100%',
    padding: '12px',
    border: 'none',
    borderRadius: '6px',
    backgroundColor: '#2563eb',
    color: '#ffffff',
    fontSize: '14px',
    fontWeight: 600,
    cursor: 'pointer',
    transition: 'all 0.2s',
  },
  exportButtonDisabled: {
    backgroundColor: '#94a3b8',
    cursor: 'not-allowed',
  },
  progressBar: {
    width: '100%',
    height: '4px',
    backgroundColor: '#e2e8f0',
    borderRadius: '2px',
    marginTop: '12px',
    overflow: 'hidden',
  },
  progressFill: {
    height: '100%',
    backgroundColor: '#2563eb',
    transition: 'width 0.3s',
  },
};

export default ReportExport;
