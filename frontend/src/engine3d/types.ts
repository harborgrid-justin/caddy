/**
 * TypeScript type definitions for 3D Engine
 *
 * Enterprise-grade type system for the CADDY 3D modeling engine
 */

/**
 * 3D Point
 */
export interface Point3 {
  x: number;
  y: number;
  z: number;
}

/**
 * 3D Vector
 */
export interface Vector3 {
  x: number;
  y: number;
  z: number;
}

/**
 * 2D Point
 */
export interface Point2 {
  x: number;
  y: number;
}

/**
 * Material properties
 */
export interface Material {
  id: string;
  name: string;
  color: [number, number, number, number]; // RGBA
  metallic: number;
  roughness: number;
  emissive?: [number, number, number];
  transparency?: number;
  texture?: string;
}

/**
 * Mesh vertex
 */
export interface Vertex {
  position: Point3;
  normal: Vector3;
  uv?: Point2;
  color?: [number, number, number, number];
}

/**
 * Mesh face (triangle or quad)
 */
export interface Face {
  vertices: number[]; // Indices into vertex array
  materialId?: string;
  normal?: Vector3;
}

/**
 * Half-edge mesh representation
 */
export interface HalfEdgeMesh {
  vertices: Vertex[];
  faces: Face[];
  edges: Edge[];
  stats: MeshStats;
}

/**
 * Edge in the mesh
 */
export interface Edge {
  vertices: [number, number]; // Vertex indices
  faces: number[]; // Adjacent face indices
  isBoundary: boolean;
  isSharp: boolean;
}

/**
 * Mesh statistics
 */
export interface MeshStats {
  vertices: number;
  faces: number;
  edges: number;
  isManifold: boolean;
  volume?: number;
  surfaceArea?: number;
}

/**
 * NURBS curve
 */
export interface NurbsCurve {
  degree: number;
  controlPoints: Point3[];
  weights: number[];
  knots: number[];
}

/**
 * NURBS surface
 */
export interface NurbsSurface {
  degreeU: number;
  degreeV: number;
  controlPointsGrid: Point3[][];
  weightsGrid: number[][];
  knotsU: number[];
  knotsV: number[];
}

/**
 * Boolean operation type
 */
export enum BooleanOp {
  Union = 'union',
  Intersection = 'intersection',
  Difference = 'difference',
}

/**
 * Extrude operation parameters
 */
export interface ExtrudeParams {
  direction: Vector3;
  capped: boolean;
  twist?: number;
  taper?: number;
}

/**
 * Revolve operation parameters
 */
export interface RevolveParams {
  axisOrigin: Point3;
  axisDirection: Vector3;
  angle: number;
  segments: number;
}

/**
 * Sweep operation parameters
 */
export interface SweepParams {
  path: NurbsCurve;
  samples: number;
  frenetFrame: boolean;
}

/**
 * Loft operation parameters
 */
export interface LoftParams {
  profiles: Point3[][];
  closed: boolean;
}

/**
 * Topology operation type
 */
export enum TopologyOperation {
  Extrude = 'extrude',
  Revolve = 'revolve',
  Sweep = 'sweep',
  Loft = 'loft',
  Shell = 'shell',
}

/**
 * Feature in the model tree
 */
export interface ModelFeature {
  id: string;
  type: TopologyOperation | BooleanOp | 'primitive';
  name: string;
  visible: boolean;
  locked: boolean;
  parameters: ExtrudeParams | RevolveParams | SweepParams | LoftParams | Record<string, unknown>;
  children?: ModelFeature[];
  mesh?: HalfEdgeMesh;
  materialId?: string;
}

/**
 * Constraint type
 */
export enum ConstraintType {
  FixedPoint = 'fixedPoint',
  Distance = 'distance',
  Angle = 'angle',
  Parallel = 'parallel',
  Perpendicular = 'perpendicular',
  Coincident = 'coincident',
  Horizontal = 'horizontal',
  Vertical = 'vertical',
  PointOnLine = 'pointOnLine',
  Tangent = 'tangent',
}

/**
 * Geometric constraint
 */
export interface Constraint {
  id: string;
  type: ConstraintType;
  entityIds: string[];
  value?: number;
  enabled: boolean;
  satisfied: boolean;
  error: number;
}

/**
 * Mass properties
 */
export interface MassProperties {
  volume: number;
  surfaceArea: number;
  centerOfMass: Point3;
  inertiaTensor: number[][];
  principalMoments: [number, number, number];
  boundingBox: {
    min: Point3;
    max: Point3;
  };
}

/**
 * Tessellation settings
 */
export interface TessellationSettings {
  maxChordError: number;
  maxAngleDeviation: number;
  minEdgeLength: number;
  maxEdgeLength: number;
  targetTriangleCount?: number;
}

/**
 * Simplification settings
 */
export interface SimplificationSettings {
  targetTriangleCount?: number;
  reductionRatio: number;
  preserveBoundaries: boolean;
  preserveSharpFeatures: boolean;
  sharpAngleThreshold: number;
  maxError: number;
}

/**
 * Healing report
 */
export interface HealingReport {
  mergedVertices: number;
  removedDegenerateFaces: number;
  flippedNormals: number;
  filledHoles: number;
  removedIsolatedVertices: number;
  hasChanges: boolean;
}

/**
 * 3D Engine state
 */
export interface Engine3DState {
  features: ModelFeature[];
  selectedFeatureIds: string[];
  materials: Material[];
  constraints: Constraint[];
  activeOperation?: {
    type: TopologyOperation | BooleanOp;
    preview?: HalfEdgeMesh;
  };
  settings: {
    tessellation: TessellationSettings;
    simplification: SimplificationSettings;
  };
}

/**
 * 3D Engine actions
 */
export type Engine3DAction =
  | { type: 'ADD_FEATURE'; payload: ModelFeature }
  | { type: 'UPDATE_FEATURE'; payload: { id: string; updates: Partial<ModelFeature> } }
  | { type: 'DELETE_FEATURE'; payload: string }
  | { type: 'SELECT_FEATURES'; payload: string[] }
  | { type: 'ADD_MATERIAL'; payload: Material }
  | { type: 'UPDATE_MATERIAL'; payload: { id: string; updates: Partial<Material> } }
  | { type: 'DELETE_MATERIAL'; payload: string }
  | { type: 'ADD_CONSTRAINT'; payload: Constraint }
  | { type: 'UPDATE_CONSTRAINT'; payload: { id: string; updates: Partial<Constraint> } }
  | { type: 'DELETE_CONSTRAINT'; payload: string }
  | { type: 'SET_ACTIVE_OPERATION'; payload: Engine3DState['activeOperation'] }
  | { type: 'UPDATE_SETTINGS'; payload: Partial<Engine3DState['settings']> };

/**
 * Render quality levels
 */
export enum RenderQuality {
  Low = 'low',
  Medium = 'medium',
  High = 'high',
  Ultra = 'ultra',
}

/**
 * Camera type
 */
export enum CameraType {
  Perspective = 'perspective',
  Orthographic = 'orthographic',
}

/**
 * Camera configuration
 */
export interface CameraConfig {
  type: CameraType;
  position: Point3;
  target: Point3;
  up: Vector3;
  fov?: number; // For perspective
  zoom?: number; // For orthographic
  near: number;
  far: number;
}

/**
 * Viewport configuration
 */
export interface ViewportConfig {
  camera: CameraConfig;
  grid: {
    visible: boolean;
    size: number;
    divisions: number;
  };
  axes: {
    visible: boolean;
    size: number;
  };
  renderQuality: RenderQuality;
  backgroundColor: [number, number, number, number];
}

/**
 * Selection information
 */
export interface SelectionInfo {
  featureId?: string;
  faceIndex?: number;
  edgeIndex?: number;
  vertexIndex?: number;
  point: Point3;
  normal: Vector3;
}

/**
 * Curvature analysis result
 */
export interface CurvatureAnalysis {
  vertexId: number;
  gaussianCurvature: number;
  meanCurvature: number;
  principalCurvatures: [number, number];
  principalDirections: [Vector3, Vector3];
}
