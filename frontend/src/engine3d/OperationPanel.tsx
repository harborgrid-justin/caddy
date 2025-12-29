/**
 * Operation Panel Component
 *
 * Controls for 3D modeling operations (extrude, revolve, sweep, loft, boolean)
 */

import React, { useState, useCallback } from 'react';
import { useTopology, useBoolean, useOperationPreview } from './use3DEngine';
import type { TopologyOperation, BooleanOp, Point3, Vector3 } from './types';

/**
 * Props for OperationPanel component
 */
interface OperationPanelProps {
  className?: string;
}

/**
 * Extrude operation form
 */
function ExtrudeForm() {
  const [direction, setDirection] = useState<Vector3>({ x: 0, y: 0, z: 1 });
  const [distance, setDistance] = useState(10);
  const [capped, setCapped] = useState(true);
  const [twist, setTwist] = useState(0);
  const [taper, setTaper] = useState(1);

  const { extrude } = useTopology();

  const handleExtrude = useCallback(async () => {
    const normalizedDir: Vector3 = {
      x: direction.x * distance,
      y: direction.y * distance,
      z: direction.z * distance,
    };

    // This would use the selected profile from the viewport
    const profile: Point3[] = [];
    await extrude(profile, [normalizedDir.x, normalizedDir.y, normalizedDir.z]);
  }, [direction, distance, extrude]);

  return (
    <div className="operation-form">
      <h4>Extrude</h4>

      <div className="form-group">
        <label>Direction</label>
        <div className="vector-input">
          <input
            type="number"
            value={direction.x}
            onChange={(e) => setDirection({ ...direction, x: parseFloat(e.target.value) })}
            placeholder="X"
          />
          <input
            type="number"
            value={direction.y}
            onChange={(e) => setDirection({ ...direction, y: parseFloat(e.target.value) })}
            placeholder="Y"
          />
          <input
            type="number"
            value={direction.z}
            onChange={(e) => setDirection({ ...direction, z: parseFloat(e.target.value) })}
            placeholder="Z"
          />
        </div>
      </div>

      <div className="form-group">
        <label>Distance</label>
        <input
          type="number"
          value={distance}
          onChange={(e) => setDistance(parseFloat(e.target.value))}
          step={0.1}
        />
      </div>

      <div className="form-group">
        <label>
          <input type="checkbox" checked={capped} onChange={(e) => setCapped(e.target.checked)} />
          Capped
        </label>
      </div>

      <div className="form-group">
        <label>Twist (rad/unit)</label>
        <input type="number" value={twist} onChange={(e) => setTwist(parseFloat(e.target.value))} step={0.1} />
      </div>

      <div className="form-group">
        <label>Taper</label>
        <input type="number" value={taper} onChange={(e) => setTaper(parseFloat(e.target.value))} step={0.1} />
      </div>

      <button className="primary-button" onClick={handleExtrude}>
        Apply Extrude
      </button>
    </div>
  );
}

/**
 * Revolve operation form
 */
function RevolveForm() {
  const [axisOrigin, setAxisOrigin] = useState<Point3>({ x: 0, y: 0, z: 0 });
  const [angle, setAngle] = useState(360);
  const [segments, setSegments] = useState(32);

  const { revolve } = useTopology();

  const handleRevolve = useCallback(async () => {
    const profile: Point3[] = [];
    await revolve(profile, axisOrigin, (angle * Math.PI) / 180, segments);
  }, [axisOrigin, angle, segments, revolve]);

  return (
    <div className="operation-form">
      <h4>Revolve</h4>

      <div className="form-group">
        <label>Axis Origin</label>
        <div className="vector-input">
          <input
            type="number"
            value={axisOrigin.x}
            onChange={(e) => setAxisOrigin({ ...axisOrigin, x: parseFloat(e.target.value) })}
            placeholder="X"
          />
          <input
            type="number"
            value={axisOrigin.y}
            onChange={(e) => setAxisOrigin({ ...axisOrigin, y: parseFloat(e.target.value) })}
            placeholder="Y"
          />
          <input
            type="number"
            value={axisOrigin.z}
            onChange={(e) => setAxisOrigin({ ...axisOrigin, z: parseFloat(e.target.value) })}
            placeholder="Z"
          />
        </div>
      </div>

      <div className="form-group">
        <label>Angle (degrees)</label>
        <input type="number" value={angle} onChange={(e) => setAngle(parseFloat(e.target.value))} step={1} />
      </div>

      <div className="form-group">
        <label>Segments</label>
        <input type="number" value={segments} onChange={(e) => setSegments(parseInt(e.target.value))} step={1} />
      </div>

      <button className="primary-button" onClick={handleRevolve}>
        Apply Revolve
      </button>
    </div>
  );
}

/**
 * Boolean operations form
 */
function BooleanForm() {
  const [operation, setOperation] = useState<BooleanOp>('union' as BooleanOp);
  const { union, intersection, difference } = useBoolean();

  const handleBoolean = useCallback(async () => {
    // This would use the selected features from the model tree
    console.log(`Performing ${operation} operation`);
  }, [operation]);

  return (
    <div className="operation-form">
      <h4>Boolean Operations</h4>

      <div className="form-group">
        <label>Operation</label>
        <select value={operation} onChange={(e) => setOperation(e.target.value as BooleanOp)}>
          <option value="union">Union</option>
          <option value="intersection">Intersection</option>
          <option value="difference">Difference</option>
        </select>
      </div>

      <div className="info-box">
        <p>Select two or more features in the model tree to perform boolean operations.</p>
      </div>

      <button className="primary-button" onClick={handleBoolean}>
        Apply {operation.charAt(0).toUpperCase() + operation.slice(1)}
      </button>
    </div>
  );
}

/**
 * Main OperationPanel component
 */
export function OperationPanel({ className = '' }: OperationPanelProps) {
  const [activeTab, setActiveTab] = useState<'extrude' | 'revolve' | 'sweep' | 'loft' | 'boolean'>('extrude');

  return (
    <div className={`operation-panel ${className}`}>
      <div className="panel-header">
        <h3>3D Operations</h3>
      </div>

      <div className="panel-tabs">
        <button
          className={`tab ${activeTab === 'extrude' ? 'active' : ''}`}
          onClick={() => setActiveTab('extrude')}
        >
          Extrude
        </button>
        <button
          className={`tab ${activeTab === 'revolve' ? 'active' : ''}`}
          onClick={() => setActiveTab('revolve')}
        >
          Revolve
        </button>
        <button className={`tab ${activeTab === 'sweep' ? 'active' : ''}`} onClick={() => setActiveTab('sweep')}>
          Sweep
        </button>
        <button className={`tab ${activeTab === 'loft' ? 'active' : ''}`} onClick={() => setActiveTab('loft')}>
          Loft
        </button>
        <button
          className={`tab ${activeTab === 'boolean' ? 'active' : ''}`}
          onClick={() => setActiveTab('boolean')}
        >
          Boolean
        </button>
      </div>

      <div className="panel-content">
        {activeTab === 'extrude' && <ExtrudeForm />}
        {activeTab === 'revolve' && <RevolveForm />}
        {activeTab === 'boolean' && <BooleanForm />}
        {(activeTab === 'sweep' || activeTab === 'loft') && (
          <div className="coming-soon">
            <p>{activeTab.charAt(0).toUpperCase() + activeTab.slice(1)} operation coming soon!</p>
          </div>
        )}
      </div>

      <style jsx>{`
        .operation-panel {
          display: flex;
          flex-direction: column;
          height: 100%;
          background: #1e1e1e;
          color: #cccccc;
        }

        .panel-header {
          padding: 12px 16px;
          border-bottom: 1px solid #333;
        }

        .panel-header h3 {
          margin: 0;
          font-size: 14px;
          font-weight: 600;
        }

        .panel-tabs {
          display: flex;
          border-bottom: 1px solid #333;
          overflow-x: auto;
        }

        .tab {
          background: transparent;
          border: none;
          color: #cccccc;
          padding: 10px 16px;
          cursor: pointer;
          font-size: 13px;
          border-bottom: 2px solid transparent;
          transition: all 0.15s;
          white-space: nowrap;
        }

        .tab:hover {
          background: #2a2a2a;
        }

        .tab.active {
          color: #4fc3f7;
          border-bottom-color: #4fc3f7;
        }

        .panel-content {
          flex: 1;
          overflow-y: auto;
          padding: 16px;
        }

        .operation-form h4 {
          margin: 0 0 16px 0;
          font-size: 13px;
          font-weight: 600;
          color: #4fc3f7;
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

        .form-group input[type='number'],
        .form-group select {
          width: 100%;
          background: #2a2a2a;
          border: 1px solid #444;
          color: #cccccc;
          padding: 6px 8px;
          font-size: 13px;
          border-radius: 4px;
        }

        .form-group input[type='checkbox'] {
          margin-right: 8px;
        }

        .vector-input {
          display: grid;
          grid-template-columns: repeat(3, 1fr);
          gap: 8px;
        }

        .primary-button {
          width: 100%;
          background: #4fc3f7;
          color: #000;
          border: none;
          padding: 10px;
          font-size: 13px;
          font-weight: 600;
          border-radius: 4px;
          cursor: pointer;
          transition: background 0.15s;
        }

        .primary-button:hover {
          background: #29b6f6;
        }

        .info-box {
          background: #2a2a2a;
          border: 1px solid #444;
          border-radius: 4px;
          padding: 12px;
          margin-bottom: 16px;
        }

        .info-box p {
          margin: 0;
          font-size: 12px;
          color: #999;
        }

        .coming-soon {
          text-align: center;
          padding: 32px;
          color: #666;
        }
      `}</style>
    </div>
  );
}
