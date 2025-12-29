/**
 * React hooks for 3D engine operations
 *
 * Provides convenient hooks for accessing and manipulating 3D engine state
 */

import { useContext, useCallback, useMemo } from 'react';
import { Engine3DContext } from './Engine3DProvider';
import type {
  ModelFeature,
  Material,
  Constraint,
  BooleanOp,
  TopologyOperation,
  Point3,
  HalfEdgeMesh,
  MassProperties,
  HealingReport,
  SimplificationSettings,
} from './types';

/**
 * Main hook for accessing 3D engine state and operations
 */
export function use3DEngine() {
  const context = useContext(Engine3DContext);

  if (!context) {
    throw new Error('use3DEngine must be used within an Engine3DProvider');
  }

  return context;
}

/**
 * Hook for feature management
 */
export function useFeatures() {
  const { state, dispatch } = use3DEngine();

  const addFeature = useCallback(
    (feature: ModelFeature) => {
      dispatch({ type: 'ADD_FEATURE', payload: feature });
    },
    [dispatch]
  );

  const updateFeature = useCallback(
    (id: string, updates: Partial<ModelFeature>) => {
      dispatch({ type: 'UPDATE_FEATURE', payload: { id, updates } });
    },
    [dispatch]
  );

  const deleteFeature = useCallback(
    (id: string) => {
      dispatch({ type: 'DELETE_FEATURE', payload: id });
    },
    [dispatch]
  );

  const getFeature = useCallback(
    (id: string): ModelFeature | undefined => {
      const findFeature = (features: ModelFeature[]): ModelFeature | undefined => {
        for (const feature of features) {
          if (feature.id === id) return feature;
          if (feature.children) {
            const found = findFeature(feature.children);
            if (found) return found;
          }
        }
        return undefined;
      };
      return findFeature(state.features);
    },
    [state.features]
  );

  return {
    features: state.features,
    selectedFeatures: state.features.filter((f) =>
      state.selectedFeatureIds.includes(f.id)
    ),
    addFeature,
    updateFeature,
    deleteFeature,
    getFeature,
  };
}

/**
 * Hook for material management
 */
export function useMaterials() {
  const { state, dispatch } = use3DEngine();

  const addMaterial = useCallback(
    (material: Material) => {
      dispatch({ type: 'ADD_MATERIAL', payload: material });
    },
    [dispatch]
  );

  const updateMaterial = useCallback(
    (id: string, updates: Partial<Material>) => {
      dispatch({ type: 'UPDATE_MATERIAL', payload: { id, updates } });
    },
    [dispatch]
  );

  const deleteMaterial = useCallback(
    (id: string) => {
      dispatch({ type: 'DELETE_MATERIAL', payload: id });
    },
    [dispatch]
  );

  const getMaterial = useCallback(
    (id: string): Material | undefined => {
      return state.materials.find((m) => m.id === id);
    },
    [state.materials]
  );

  return {
    materials: state.materials,
    addMaterial,
    updateMaterial,
    deleteMaterial,
    getMaterial,
  };
}

/**
 * Hook for constraint management
 */
export function useConstraints() {
  const { state, dispatch } = use3DEngine();

  const addConstraint = useCallback(
    (constraint: Constraint) => {
      dispatch({ type: 'ADD_CONSTRAINT', payload: constraint });
    },
    [dispatch]
  );

  const updateConstraint = useCallback(
    (id: string, updates: Partial<Constraint>) => {
      dispatch({ type: 'UPDATE_CONSTRAINT', payload: { id, updates } });
    },
    [dispatch]
  );

  const deleteConstraint = useCallback(
    (id: string) => {
      dispatch({ type: 'DELETE_CONSTRAINT', payload: id });
    },
    [dispatch]
  );

  const solveConstraints = useCallback(async () => {
    // Call Rust backend to solve constraints
    // This would invoke the constraint solver
    console.log('Solving constraints...');
  }, []);

  const unsatisfiedConstraints = useMemo(() => {
    return state.constraints.filter((c) => c.enabled && !c.satisfied);
  }, [state.constraints]);

  return {
    constraints: state.constraints,
    unsatisfiedConstraints,
    addConstraint,
    updateConstraint,
    deleteConstraint,
    solveConstraints,
  };
}

/**
 * Hook for topology operations
 */
export function useTopology() {
  const { dispatch } = use3DEngine();

  const extrude = useCallback(
    async (profile: Point3[], direction: [number, number, number]) => {
      // Call Rust backend for extrude operation
      console.log('Extruding profile...', profile, direction);
      // Returns HalfEdgeMesh from Rust
      return null as unknown as HalfEdgeMesh;
    },
    []
  );

  const revolve = useCallback(
    async (profile: Point3[], axis: Point3, angle: number, segments: number) => {
      console.log('Revolving profile...', profile, axis, angle, segments);
      return null as unknown as HalfEdgeMesh;
    },
    []
  );

  const loft = useCallback(async (profiles: Point3[][], closed: boolean) => {
    console.log('Lofting profiles...', profiles, closed);
    return null as unknown as HalfEdgeMesh;
  }, []);

  return {
    extrude,
    revolve,
    loft,
  };
}

/**
 * Hook for boolean operations
 */
export function useBoolean() {
  const booleanOperation = useCallback(
    async (meshA: HalfEdgeMesh, meshB: HalfEdgeMesh, operation: BooleanOp) => {
      console.log('Performing boolean operation...', operation);
      // Call Rust backend
      return null as unknown as HalfEdgeMesh;
    },
    []
  );

  return {
    union: (meshA: HalfEdgeMesh, meshB: HalfEdgeMesh) =>
      booleanOperation(meshA, meshB, BooleanOp.Union),
    intersection: (meshA: HalfEdgeMesh, meshB: HalfEdgeMesh) =>
      booleanOperation(meshA, meshB, BooleanOp.Intersection),
    difference: (meshA: HalfEdgeMesh, meshB: HalfEdgeMesh) =>
      booleanOperation(meshA, meshB, BooleanOp.Difference),
  };
}

/**
 * Hook for mesh analysis
 */
export function useAnalysis() {
  const computeMassProperties = useCallback(
    async (mesh: HalfEdgeMesh, density = 1.0): Promise<MassProperties> => {
      console.log('Computing mass properties...', mesh, density);
      // Call Rust backend
      return {
        volume: 0,
        surfaceArea: 0,
        centerOfMass: { x: 0, y: 0, z: 0 },
        inertiaTensor: [],
        principalMoments: [0, 0, 0],
        boundingBox: {
          min: { x: 0, y: 0, z: 0 },
          max: { x: 0, y: 0, z: 0 },
        },
      };
    },
    []
  );

  const computeCurvature = useCallback(async (mesh: HalfEdgeMesh, vertexId: number) => {
    console.log('Computing curvature...', mesh, vertexId);
    return {
      gaussian: 0,
      mean: 0,
    };
  }, []);

  return {
    computeMassProperties,
    computeCurvature,
  };
}

/**
 * Hook for mesh operations
 */
export function useMesh() {
  const healMesh = useCallback(async (mesh: HalfEdgeMesh): Promise<HealingReport> => {
    console.log('Healing mesh...', mesh);
    return {
      mergedVertices: 0,
      removedDegenerateFaces: 0,
      flippedNormals: 0,
      filledHoles: 0,
      removedIsolatedVertices: 0,
      hasChanges: false,
    };
  }, []);

  const simplifyMesh = useCallback(
    async (mesh: HalfEdgeMesh, settings: SimplificationSettings) => {
      console.log('Simplifying mesh...', mesh, settings);
      return mesh;
    },
    []
  );

  const tessellateMesh = useCallback(async (mesh: HalfEdgeMesh, maxError: number) => {
    console.log('Tessellating mesh...', mesh, maxError);
    return mesh;
  }, []);

  return {
    healMesh,
    simplifyMesh,
    tessellateMesh,
  };
}

/**
 * Hook for selection management
 */
export function useSelection() {
  const { state, dispatch } = use3DEngine();

  const selectFeatures = useCallback(
    (ids: string[]) => {
      dispatch({ type: 'SELECT_FEATURES', payload: ids });
    },
    [dispatch]
  );

  const clearSelection = useCallback(() => {
    dispatch({ type: 'SELECT_FEATURES', payload: [] });
  }, [dispatch]);

  const toggleSelection = useCallback(
    (id: string) => {
      const currentSelection = state.selectedFeatureIds;
      const newSelection = currentSelection.includes(id)
        ? currentSelection.filter((fid) => fid !== id)
        : [...currentSelection, id];
      dispatch({ type: 'SELECT_FEATURES', payload: newSelection });
    },
    [state.selectedFeatureIds, dispatch]
  );

  return {
    selectedFeatureIds: state.selectedFeatureIds,
    selectFeatures,
    clearSelection,
    toggleSelection,
    hasSelection: state.selectedFeatureIds.length > 0,
  };
}

/**
 * Hook for operation preview
 */
export function useOperationPreview() {
  const { state, dispatch } = use3DEngine();

  const setActiveOperation = useCallback(
    (operation: typeof state.activeOperation) => {
      dispatch({ type: 'SET_ACTIVE_OPERATION', payload: operation });
    },
    [dispatch]
  );

  const clearOperation = useCallback(() => {
    dispatch({ type: 'SET_ACTIVE_OPERATION', payload: undefined });
  }, [dispatch]);

  return {
    activeOperation: state.activeOperation,
    setActiveOperation,
    clearOperation,
    hasActiveOperation: !!state.activeOperation,
  };
}
