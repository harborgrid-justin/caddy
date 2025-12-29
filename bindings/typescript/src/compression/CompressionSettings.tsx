/**
 * CompressionSettings - Settings UI for compression options
 *
 * React component for configuring compression settings
 */

import React, { useState, useEffect } from 'react';
import {
  CompressionSettings as ICompressionSettings,
  CompressionAlgorithm,
  CompressionLevel,
  formatFileSize,
} from './types';
import { compressionService } from './CompressionService';

export interface CompressionSettingsProps {
  /** Callback when settings change */
  onChange?: (settings: ICompressionSettings) => void;
  /** Custom CSS class */
  className?: string;
}

/**
 * CompressionSettings component
 */
export const CompressionSettings: React.FC<CompressionSettingsProps> = ({
  onChange,
  className = '',
}) => {
  const [settings, setSettings] = useState<ICompressionSettings>(
    compressionService.getSettings()
  );

  // Update service when settings change
  useEffect(() => {
    compressionService.updateSettings(settings);
    if (onChange) {
      onChange(settings);
    }
  }, [settings, onChange]);

  const handleAlgorithmChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setSettings({
      ...settings,
      defaultAlgorithm: e.target.value as CompressionAlgorithm,
    });
  };

  const handleLevelChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setSettings({
      ...settings,
      defaultLevel: parseInt(e.target.value) as CompressionLevel,
    });
  };

  const handleAutoDetectionChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setSettings({
      ...settings,
      enableAutoDetection: e.target.checked,
    });
  };

  const handleParallelChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setSettings({
      ...settings,
      useParallelForLargeFiles: e.target.checked,
    });
  };

  const handleThresholdChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setSettings({
      ...settings,
      largeFileThreshold: parseInt(e.target.value) * 1024 * 1024, // Convert MB to bytes
    });
  };

  const handleStatisticsChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setSettings({
      ...settings,
      trackStatistics: e.target.checked,
    });
  };

  const handleResetDefaults = () => {
    const defaultSettings: ICompressionSettings = {
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

  return (
    <div className={`compression-settings ${className}`}>
      <div className="settings-section">
        <h3 className="settings-title">Compression Settings</h3>

        {/* Algorithm Selection */}
        <div className="setting-group">
          <label htmlFor="algorithm-select" className="setting-label">
            Default Algorithm
          </label>
          <select
            id="algorithm-select"
            value={settings.defaultAlgorithm}
            onChange={handleAlgorithmChange}
            className="setting-select"
            disabled={settings.enableAutoDetection}
          >
            <option value={CompressionAlgorithm.Adaptive}>
              Adaptive (Auto-select best)
            </option>
            <option value={CompressionAlgorithm.LZ4Custom}>
              LZ4 Custom (Fast, general-purpose)
            </option>
            <option value={CompressionAlgorithm.Delta}>
              Delta Encoding (Versioned data)
            </option>
            <option value={CompressionAlgorithm.Mesh}>
              Mesh Compression (3D geometry)
            </option>
            <option value={CompressionAlgorithm.Dictionary}>
              Dictionary (Text-heavy data)
            </option>
            <option value={CompressionAlgorithm.Parallel}>
              Parallel (Large files)
            </option>
          </select>
          <p className="setting-description">
            Choose the compression algorithm to use by default
          </p>
        </div>

        {/* Compression Level */}
        <div className="setting-group">
          <label htmlFor="level-select" className="setting-label">
            Compression Level
          </label>
          <select
            id="level-select"
            value={settings.defaultLevel}
            onChange={handleLevelChange}
            className="setting-select"
          >
            <option value={CompressionLevel.Fastest}>Fastest (Level 1)</option>
            <option value={CompressionLevel.Fast}>Fast (Level 3)</option>
            <option value={CompressionLevel.Balanced}>Balanced (Level 5)</option>
            <option value={CompressionLevel.Best}>Best (Level 7)</option>
            <option value={CompressionLevel.Maximum}>Maximum (Level 9)</option>
          </select>
          <p className="setting-description">
            Higher levels provide better compression but take longer
          </p>
        </div>

        {/* Auto Detection */}
        <div className="setting-group">
          <label className="setting-checkbox-label">
            <input
              type="checkbox"
              checked={settings.enableAutoDetection}
              onChange={handleAutoDetectionChange}
              className="setting-checkbox"
            />
            <span>Enable Auto-Detection</span>
          </label>
          <p className="setting-description">
            Automatically select the best algorithm for each file
          </p>
        </div>

        {/* Parallel Compression */}
        <div className="setting-group">
          <label className="setting-checkbox-label">
            <input
              type="checkbox"
              checked={settings.useParallelForLargeFiles}
              onChange={handleParallelChange}
              className="setting-checkbox"
            />
            <span>Use Parallel Compression for Large Files</span>
          </label>
          <p className="setting-description">
            Utilize multiple CPU cores for faster compression of large files
          </p>
        </div>

        {/* Large File Threshold */}
        {settings.useParallelForLargeFiles && (
          <div className="setting-group">
            <label htmlFor="threshold-input" className="setting-label">
              Large File Threshold
            </label>
            <div className="setting-input-group">
              <input
                id="threshold-input"
                type="number"
                min="1"
                max="1000"
                value={settings.largeFileThreshold / (1024 * 1024)}
                onChange={handleThresholdChange}
                className="setting-input"
              />
              <span className="setting-unit">MB</span>
            </div>
            <p className="setting-description">
              Files larger than this will use parallel compression
              (Current: {formatFileSize(settings.largeFileThreshold)})
            </p>
          </div>
        )}

        {/* Statistics Tracking */}
        <div className="setting-group">
          <label className="setting-checkbox-label">
            <input
              type="checkbox"
              checked={settings.trackStatistics}
              onChange={handleStatisticsChange}
              className="setting-checkbox"
            />
            <span>Track Compression Statistics</span>
          </label>
          <p className="setting-description">
            Monitor compression performance and algorithm usage
          </p>
        </div>

        {/* Reset Button */}
        <div className="setting-actions">
          <button
            onClick={handleResetDefaults}
            className="setting-button setting-button-secondary"
          >
            Reset to Defaults
          </button>
        </div>
      </div>

      <style>{`
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
      `}</style>
    </div>
  );
};

export default CompressionSettings;
