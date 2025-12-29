/**
 * Material Editor Component
 *
 * Manages material properties and appearance settings for 3D models
 */

import React, { useState, useCallback, useEffect } from 'react';
import { useMaterials } from './use3DEngine';
import type { Material } from './types';

/**
 * Props for MaterialEditor component
 */
interface MaterialEditorProps {
  className?: string;
  selectedMaterialId?: string;
  onMaterialSelect?: (materialId: string) => void;
}

/**
 * Color picker component
 */
interface ColorPickerProps {
  color: [number, number, number, number];
  onChange: (color: [number, number, number, number]) => void;
  label: string;
  showAlpha?: boolean;
}

function ColorPicker({ color, onChange, label, showAlpha = true }: ColorPickerProps) {
  const rgbToHex = (rgb: [number, number, number, number]) => {
    const toHex = (n: number) => Math.round(n * 255).toString(16).padStart(2, '0');
    return `#${toHex(rgb[0])}${toHex(rgb[1])}${toHex(rgb[2])}`;
  };

  const hexToRgb = (hex: string): [number, number, number] => {
    const r = parseInt(hex.slice(1, 3), 16) / 255;
    const g = parseInt(hex.slice(3, 5), 16) / 255;
    const b = parseInt(hex.slice(5, 7), 16) / 255;
    return [r, g, b];
  };

  const handleColorChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const [r, g, b] = hexToRgb(e.target.value);
    onChange([r, g, b, color[3]]);
  };

  const handleAlphaChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onChange([color[0], color[1], color[2], parseFloat(e.target.value)]);
  };

  return (
    <div className="color-picker">
      <label>{label}</label>
      <div className="color-controls">
        <input type="color" value={rgbToHex(color)} onChange={handleColorChange} className="color-input" />
        {showAlpha && (
          <div className="alpha-control">
            <label>Alpha</label>
            <input
              type="range"
              min="0"
              max="1"
              step="0.01"
              value={color[3]}
              onChange={handleAlphaChange}
              className="alpha-slider"
            />
            <span className="alpha-value">{(color[3] * 100).toFixed(0)}%</span>
          </div>
        )}
      </div>

      <style jsx>{`
        .color-picker {
          margin-bottom: 16px;
        }

        .color-picker > label {
          display: block;
          margin-bottom: 6px;
          font-size: 12px;
          color: #999;
        }

        .color-controls {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .color-input {
          width: 100%;
          height: 40px;
          border: 1px solid #444;
          border-radius: 4px;
          cursor: pointer;
        }

        .alpha-control {
          display: flex;
          align-items: center;
          gap: 8px;
        }

        .alpha-control label {
          font-size: 11px;
          color: #999;
        }

        .alpha-slider {
          flex: 1;
        }

        .alpha-value {
          font-size: 11px;
          color: #999;
          min-width: 40px;
          text-align: right;
        }
      `}</style>
    </div>
  );
}

/**
 * Material preview sphere
 */
interface MaterialPreviewProps {
  material: Material;
}

function MaterialPreview({ material }: MaterialPreviewProps) {
  return (
    <div className="material-preview">
      <div
        className="preview-sphere"
        style={{
          background: `rgba(${material.color[0] * 255}, ${material.color[1] * 255}, ${material.color[2] * 255}, ${material.color[3]})`,
        }}
      />
      <div className="preview-info">
        <div className="info-row">
          <span>Metallic:</span>
          <span>{(material.metallic * 100).toFixed(0)}%</span>
        </div>
        <div className="info-row">
          <span>Roughness:</span>
          <span>{(material.roughness * 100).toFixed(0)}%</span>
        </div>
      </div>

      <style jsx>{`
        .material-preview {
          background: #2a2a2a;
          border: 1px solid #444;
          border-radius: 8px;
          padding: 16px;
          margin-bottom: 16px;
        }

        .preview-sphere {
          width: 100%;
          height: 120px;
          border-radius: 50%;
          margin-bottom: 12px;
          box-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
        }

        .preview-info {
          font-size: 12px;
        }

        .info-row {
          display: flex;
          justify-content: space-between;
          margin-bottom: 4px;
          color: #999;
        }

        .info-row span:last-child {
          color: #ccc;
          font-weight: 500;
        }
      `}</style>
    </div>
  );
}

/**
 * Main MaterialEditor component
 */
export function MaterialEditor({ className = '', selectedMaterialId, onMaterialSelect }: MaterialEditorProps) {
  const { materials, addMaterial, updateMaterial, deleteMaterial } = useMaterials();
  const [editingId, setEditingId] = useState<string | undefined>(selectedMaterialId);

  const editingMaterial = materials.find((m) => m.id === editingId);

  useEffect(() => {
    if (selectedMaterialId) {
      setEditingId(selectedMaterialId);
    }
  }, [selectedMaterialId]);

  const handleCreateMaterial = useCallback(() => {
    const newMaterial: Material = {
      id: `material-${Date.now()}`,
      name: `Material ${materials.length + 1}`,
      color: [0.8, 0.8, 0.8, 1.0],
      metallic: 0.0,
      roughness: 0.5,
    };
    addMaterial(newMaterial);
    setEditingId(newMaterial.id);
  }, [materials, addMaterial]);

  const handleUpdateColor = useCallback(
    (color: [number, number, number, number]) => {
      if (editingId) {
        updateMaterial(editingId, { color });
      }
    },
    [editingId, updateMaterial]
  );

  const handleUpdateMetallic = useCallback(
    (value: number) => {
      if (editingId) {
        updateMaterial(editingId, { metallic: value });
      }
    },
    [editingId, updateMaterial]
  );

  const handleUpdateRoughness = useCallback(
    (value: number) => {
      if (editingId) {
        updateMaterial(editingId, { roughness: value });
      }
    },
    [editingId, updateMaterial]
  );

  const handleUpdateName = useCallback(
    (name: string) => {
      if (editingId) {
        updateMaterial(editingId, { name });
      }
    },
    [editingId, updateMaterial]
  );

  const handleDelete = useCallback(() => {
    if (editingId && confirm('Delete this material?')) {
      deleteMaterial(editingId);
      setEditingId(materials[0]?.id);
    }
  }, [editingId, materials, deleteMaterial]);

  return (
    <div className={`material-editor ${className}`}>
      <div className="panel-header">
        <h3>Materials</h3>
        <button className="add-button" onClick={handleCreateMaterial} title="Create new material">
          +
        </button>
      </div>

      <div className="materials-sidebar">
        {materials.map((material) => (
          <div
            key={material.id}
            className={`material-item ${editingId === material.id ? 'active' : ''}`}
            onClick={() => {
              setEditingId(material.id);
              onMaterialSelect?.(material.id);
            }}
          >
            <div
              className="material-swatch"
              style={{
                background: `rgba(${material.color[0] * 255}, ${material.color[1] * 255}, ${material.color[2] * 255}, ${material.color[3]})`,
              }}
            />
            <span className="material-name">{material.name}</span>
          </div>
        ))}
      </div>

      {editingMaterial && (
        <div className="editor-content">
          <MaterialPreview material={editingMaterial} />

          <div className="editor-form">
            <div className="form-group">
              <label>Name</label>
              <input
                type="text"
                value={editingMaterial.name}
                onChange={(e) => handleUpdateName(e.target.value)}
                className="text-input"
              />
            </div>

            <ColorPicker label="Base Color" color={editingMaterial.color} onChange={handleUpdateColor} />

            <div className="form-group">
              <label>Metallic</label>
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                value={editingMaterial.metallic}
                onChange={(e) => handleUpdateMetallic(parseFloat(e.target.value))}
                className="slider"
              />
              <span className="slider-value">{(editingMaterial.metallic * 100).toFixed(0)}%</span>
            </div>

            <div className="form-group">
              <label>Roughness</label>
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                value={editingMaterial.roughness}
                onChange={(e) => handleUpdateRoughness(parseFloat(e.target.value))}
                className="slider"
              />
              <span className="slider-value">{(editingMaterial.roughness * 100).toFixed(0)}%</span>
            </div>

            {editingMaterial.id !== 'default' && (
              <button className="delete-button" onClick={handleDelete}>
                Delete Material
              </button>
            )}
          </div>
        </div>
      )}

      <style jsx>{`
        .material-editor {
          display: flex;
          flex-direction: column;
          height: 100%;
          background: #1e1e1e;
          color: #cccccc;
        }

        .panel-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 12px 16px;
          border-bottom: 1px solid #333;
        }

        .panel-header h3 {
          margin: 0;
          font-size: 14px;
          font-weight: 600;
        }

        .add-button {
          background: #4fc3f7;
          color: #000;
          border: none;
          width: 28px;
          height: 28px;
          border-radius: 50%;
          cursor: pointer;
          font-size: 18px;
          font-weight: bold;
          display: flex;
          align-items: center;
          justify-content: center;
        }

        .add-button:hover {
          background: #29b6f6;
        }

        .materials-sidebar {
          border-bottom: 1px solid #333;
          padding: 8px;
          max-height: 150px;
          overflow-y: auto;
        }

        .material-item {
          display: flex;
          align-items: center;
          padding: 8px;
          margin-bottom: 4px;
          background: #2a2a2a;
          border-radius: 4px;
          cursor: pointer;
          transition: background 0.15s;
        }

        .material-item:hover {
          background: #333;
        }

        .material-item.active {
          background: #094771;
        }

        .material-swatch {
          width: 32px;
          height: 32px;
          border-radius: 4px;
          margin-right: 12px;
          border: 1px solid #444;
        }

        .material-name {
          font-size: 13px;
        }

        .editor-content {
          flex: 1;
          overflow-y: auto;
          padding: 16px;
        }

        .editor-form {
          /* Styles for form elements */
        }

        .form-group {
          margin-bottom: 16px;
        }

        .form-group label {
          display: block;
          margin-bottom: 6px;
          font-size: 12px;
          color: #999;
        }

        .text-input {
          width: 100%;
          background: #2a2a2a;
          border: 1px solid #444;
          color: #cccccc;
          padding: 8px;
          font-size: 13px;
          border-radius: 4px;
        }

        .slider {
          width: 100%;
          margin-bottom: 4px;
        }

        .slider-value {
          font-size: 11px;
          color: #999;
        }

        .delete-button {
          width: 100%;
          background: #f44336;
          color: #fff;
          border: none;
          padding: 10px;
          font-size: 13px;
          font-weight: 600;
          border-radius: 4px;
          cursor: pointer;
          margin-top: 16px;
        }

        .delete-button:hover {
          background: #d32f2f;
        }
      `}</style>
    </div>
  );
}
