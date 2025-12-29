/**
 * 3D Engine React Context Provider
 *
 * Manages global 3D engine state and provides it to child components
 */

import React, { createContext, useReducer, useCallback, ReactNode, Dispatch } from 'react';
import type { Engine3DState, Engine3DAction, TessellationSettings, SimplificationSettings } from './types';

/**
 * Context value type
 */
interface Engine3DContextValue {
  state: Engine3DState;
  dispatch: Dispatch<Engine3DAction>;
}

/**
 * Create context with undefined default (will throw if used outside provider)
 */
export const Engine3DContext = createContext<Engine3DContextValue | undefined>(undefined);

/**
 * Default tessellation settings
 */
const defaultTessellationSettings: TessellationSettings = {
  maxChordError: 0.01,
  maxAngleDeviation: 0.1,
  minEdgeLength: 0.001,
  maxEdgeLength: 1.0,
};

/**
 * Default simplification settings
 */
const defaultSimplificationSettings: SimplificationSettings = {
  reductionRatio: 0.5,
  preserveBoundaries: true,
  preserveSharpFeatures: true,
  sharpAngleThreshold: 0.5,
  maxError: Infinity,
};

/**
 * Initial state
 */
const initialState: Engine3DState = {
  features: [],
  selectedFeatureIds: [],
  materials: [
    {
      id: 'default',
      name: 'Default Material',
      color: [0.8, 0.8, 0.8, 1.0],
      metallic: 0.0,
      roughness: 0.5,
    },
  ],
  constraints: [],
  settings: {
    tessellation: defaultTessellationSettings,
    simplification: defaultSimplificationSettings,
  },
};

/**
 * State reducer
 */
function engine3DReducer(state: Engine3DState, action: Engine3DAction): Engine3DState {
  switch (action.type) {
    case 'ADD_FEATURE':
      return {
        ...state,
        features: [...state.features, action.payload],
      };

    case 'UPDATE_FEATURE': {
      const updateFeatureRecursive = (features: typeof state.features): typeof state.features => {
        return features.map((feature) => {
          if (feature.id === action.payload.id) {
            return { ...feature, ...action.payload.updates };
          }
          if (feature.children) {
            return {
              ...feature,
              children: updateFeatureRecursive(feature.children),
            };
          }
          return feature;
        });
      };

      return {
        ...state,
        features: updateFeatureRecursive(state.features),
      };
    }

    case 'DELETE_FEATURE': {
      const deleteFeatureRecursive = (features: typeof state.features): typeof state.features => {
        return features
          .filter((feature) => feature.id !== action.payload)
          .map((feature) => {
            if (feature.children) {
              return {
                ...feature,
                children: deleteFeatureRecursive(feature.children),
              };
            }
            return feature;
          });
      };

      return {
        ...state,
        features: deleteFeatureRecursive(state.features),
        selectedFeatureIds: state.selectedFeatureIds.filter((id) => id !== action.payload),
      };
    }

    case 'SELECT_FEATURES':
      return {
        ...state,
        selectedFeatureIds: action.payload,
      };

    case 'ADD_MATERIAL':
      return {
        ...state,
        materials: [...state.materials, action.payload],
      };

    case 'UPDATE_MATERIAL':
      return {
        ...state,
        materials: state.materials.map((material) =>
          material.id === action.payload.id
            ? { ...material, ...action.payload.updates }
            : material
        ),
      };

    case 'DELETE_MATERIAL':
      return {
        ...state,
        materials: state.materials.filter((material) => material.id !== action.payload),
      };

    case 'ADD_CONSTRAINT':
      return {
        ...state,
        constraints: [...state.constraints, action.payload],
      };

    case 'UPDATE_CONSTRAINT':
      return {
        ...state,
        constraints: state.constraints.map((constraint) =>
          constraint.id === action.payload.id
            ? { ...constraint, ...action.payload.updates }
            : constraint
        ),
      };

    case 'DELETE_CONSTRAINT':
      return {
        ...state,
        constraints: state.constraints.filter((constraint) => constraint.id !== action.payload),
      };

    case 'SET_ACTIVE_OPERATION':
      return {
        ...state,
        activeOperation: action.payload,
      };

    case 'UPDATE_SETTINGS':
      return {
        ...state,
        settings: {
          ...state.settings,
          ...action.payload,
        },
      };

    default:
      return state;
  }
}

/**
 * Provider props
 */
interface Engine3DProviderProps {
  children: ReactNode;
  initialState?: Partial<Engine3DState>;
}

/**
 * 3D Engine Provider Component
 */
export function Engine3DProvider({ children, initialState: userInitialState }: Engine3DProviderProps) {
  const [state, dispatch] = useReducer(
    engine3DReducer,
    userInitialState ? { ...initialState, ...userInitialState } : initialState
  );

  const contextValue: Engine3DContextValue = {
    state,
    dispatch,
  };

  return <Engine3DContext.Provider value={contextValue}>{children}</Engine3DContext.Provider>;
}

/**
 * HOC to inject 3D engine context
 */
export function withEngine3D<P extends object>(Component: React.ComponentType<P & Engine3DContextValue>) {
  return function WithEngine3DComponent(props: P) {
    return (
      <Engine3DContext.Consumer>
        {(context) => {
          if (!context) {
            throw new Error('Component must be used within Engine3DProvider');
          }
          return <Component {...props} {...context} />;
        }}
      </Engine3DContext.Consumer>
    );
  };
}
