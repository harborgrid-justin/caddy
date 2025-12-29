import React, { useState, useEffect } from 'react';
import { CompressionAlgorithm, CompressionLevel, formatFileSize, } from './types';
import { compressionService } from './CompressionService';
export const CompressionSettings = ({ onChange, className = '', }) => {
    const [settings, setSettings] = useState(compressionService.getSettings());
    useEffect(() => {
        compressionService.updateSettings(settings);
        if (onChange) {
            onChange(settings);
        }
    }, [settings, onChange]);
    const handleAlgorithmChange = (e) => {
        setSettings({
            ...settings,
            defaultAlgorithm: e.target.value,
        });
    };
    const handleLevelChange = (e) => {
        setSettings({
            ...settings,
            defaultLevel: parseInt(e.target.value),
        });
    };
    const handleAutoDetectionChange = (e) => {
        setSettings({
            ...settings,
            enableAutoDetection: e.target.checked,
        });
    };
    const handleParallelChange = (e) => {
        setSettings({
            ...settings,
            useParallelForLargeFiles: e.target.checked,
        });
    };
    const handleThresholdChange = (e) => {
        setSettings({
            ...settings,
            largeFileThreshold: parseInt(e.target.value) * 1024 * 1024,
        });
    };
    const handleStatisticsChange = (e) => {
        setSettings({
            ...settings,
            trackStatistics: e.target.checked,
        });
    };
    const handleResetDefaults = () => {
        const defaultSettings = {
            defaultAlgorithm: CompressionAlgorithm.Adaptive,
            defaultLevel: CompressionLevel.Balanced,
            enableAutoDetection: true,
            useParallelForLargeFiles: true,
            largeFileThreshold: 10 * 1024 * 1024,
            trackStatistics: true,
            algorithmSettings: {},
        };
        setSettings(defaultSettings);
    };
    return (React.createElement("div", { className: `compression-settings ${className}` },
        React.createElement("div", { className: "settings-section" },
            React.createElement("h3", { className: "settings-title" }, "Compression Settings"),
            React.createElement("div", { className: "setting-group" },
                React.createElement("label", { htmlFor: "algorithm-select", className: "setting-label" }, "Default Algorithm"),
                React.createElement("select", { id: "algorithm-select", value: settings.defaultAlgorithm, onChange: handleAlgorithmChange, className: "setting-select", disabled: settings.enableAutoDetection },
                    React.createElement("option", { value: CompressionAlgorithm.Adaptive }, "Adaptive (Auto-select best)"),
                    React.createElement("option", { value: CompressionAlgorithm.LZ4Custom }, "LZ4 Custom (Fast, general-purpose)"),
                    React.createElement("option", { value: CompressionAlgorithm.Delta }, "Delta Encoding (Versioned data)"),
                    React.createElement("option", { value: CompressionAlgorithm.Mesh }, "Mesh Compression (3D geometry)"),
                    React.createElement("option", { value: CompressionAlgorithm.Dictionary }, "Dictionary (Text-heavy data)"),
                    React.createElement("option", { value: CompressionAlgorithm.Parallel }, "Parallel (Large files)")),
                React.createElement("p", { className: "setting-description" }, "Choose the compression algorithm to use by default")),
            React.createElement("div", { className: "setting-group" },
                React.createElement("label", { htmlFor: "level-select", className: "setting-label" }, "Compression Level"),
                React.createElement("select", { id: "level-select", value: settings.defaultLevel, onChange: handleLevelChange, className: "setting-select" },
                    React.createElement("option", { value: CompressionLevel.Fastest }, "Fastest (Level 1)"),
                    React.createElement("option", { value: CompressionLevel.Fast }, "Fast (Level 3)"),
                    React.createElement("option", { value: CompressionLevel.Balanced }, "Balanced (Level 5)"),
                    React.createElement("option", { value: CompressionLevel.Best }, "Best (Level 7)"),
                    React.createElement("option", { value: CompressionLevel.Maximum }, "Maximum (Level 9)")),
                React.createElement("p", { className: "setting-description" }, "Higher levels provide better compression but take longer")),
            React.createElement("div", { className: "setting-group" },
                React.createElement("label", { className: "setting-checkbox-label" },
                    React.createElement("input", { type: "checkbox", checked: settings.enableAutoDetection, onChange: handleAutoDetectionChange, className: "setting-checkbox" }),
                    React.createElement("span", null, "Enable Auto-Detection")),
                React.createElement("p", { className: "setting-description" }, "Automatically select the best algorithm for each file")),
            React.createElement("div", { className: "setting-group" },
                React.createElement("label", { className: "setting-checkbox-label" },
                    React.createElement("input", { type: "checkbox", checked: settings.useParallelForLargeFiles, onChange: handleParallelChange, className: "setting-checkbox" }),
                    React.createElement("span", null, "Use Parallel Compression for Large Files")),
                React.createElement("p", { className: "setting-description" }, "Utilize multiple CPU cores for faster compression of large files")),
            settings.useParallelForLargeFiles && (React.createElement("div", { className: "setting-group" },
                React.createElement("label", { htmlFor: "threshold-input", className: "setting-label" }, "Large File Threshold"),
                React.createElement("div", { className: "setting-input-group" },
                    React.createElement("input", { id: "threshold-input", type: "number", min: "1", max: "1000", value: settings.largeFileThreshold / (1024 * 1024), onChange: handleThresholdChange, className: "setting-input" }),
                    React.createElement("span", { className: "setting-unit" }, "MB")),
                React.createElement("p", { className: "setting-description" },
                    "Files larger than this will use parallel compression (Current: ",
                    formatFileSize(settings.largeFileThreshold),
                    ")"))),
            React.createElement("div", { className: "setting-group" },
                React.createElement("label", { className: "setting-checkbox-label" },
                    React.createElement("input", { type: "checkbox", checked: settings.trackStatistics, onChange: handleStatisticsChange, className: "setting-checkbox" }),
                    React.createElement("span", null, "Track Compression Statistics")),
                React.createElement("p", { className: "setting-description" }, "Monitor compression performance and algorithm usage")),
            React.createElement("div", { className: "setting-actions" },
                React.createElement("button", { onClick: handleResetDefaults, className: "setting-button setting-button-secondary" }, "Reset to Defaults"))),
        React.createElement("style", null, `
        .compression-settings {
          max-width: 600px;
          margin: 0 auto;
          padding: 20px;
          background: #ffffff;
          border-radius: 8px;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        }

        .settings-section {
          display: flex;
          flex-direction: column;
          gap: 24px;
        }

        .settings-title {
          margin: 0;
          font-size: 24px;
          font-weight: 600;
          color: #1a1a1a;
        }

        .setting-group {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .setting-label {
          font-size: 14px;
          font-weight: 500;
          color: #333;
        }

        .setting-select,
        .setting-input {
          padding: 8px 12px;
          font-size: 14px;
          border: 1px solid #d1d5db;
          border-radius: 4px;
          background: #ffffff;
          transition: border-color 0.2s;
        }

        .setting-select:hover,
        .setting-input:hover {
          border-color: #9ca3af;
        }

        .setting-select:focus,
        .setting-input:focus {
          outline: none;
          border-color: #3b82f6;
          box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
        }

        .setting-select:disabled {
          background: #f3f4f6;
          cursor: not-allowed;
        }

        .setting-input-group {
          display: flex;
          align-items: center;
          gap: 8px;
        }

        .setting-input {
          flex: 1;
        }

        .setting-unit {
          font-size: 14px;
          color: #6b7280;
        }

        .setting-checkbox-label {
          display: flex;
          align-items: center;
          gap: 8px;
          font-size: 14px;
          font-weight: 500;
          color: #333;
          cursor: pointer;
        }

        .setting-checkbox {
          width: 18px;
          height: 18px;
          cursor: pointer;
        }

        .setting-description {
          margin: 0;
          font-size: 13px;
          color: #6b7280;
          line-height: 1.5;
        }

        .setting-actions {
          display: flex;
          justify-content: flex-end;
          padding-top: 8px;
        }

        .setting-button {
          padding: 8px 16px;
          font-size: 14px;
          font-weight: 500;
          border: none;
          border-radius: 4px;
          cursor: pointer;
          transition: background-color 0.2s, transform 0.1s;
        }

        .setting-button:hover {
          transform: translateY(-1px);
        }

        .setting-button:active {
          transform: translateY(0);
        }

        .setting-button-secondary {
          background: #f3f4f6;
          color: #374151;
        }

        .setting-button-secondary:hover {
          background: #e5e7eb;
        }

        @media (max-width: 640px) {
          .compression-settings {
            padding: 16px;
          }

          .settings-title {
            font-size: 20px;
          }
        }
      `)));
};
export default CompressionSettings;
//# sourceMappingURL=CompressionSettings.js.map