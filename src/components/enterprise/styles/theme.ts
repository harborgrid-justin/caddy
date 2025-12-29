/**
 * Enterprise Theme System for CADDY
 * Supports dark/light modes with CAD-specific theming
 */

import { colors, spacing, typography, borderRadius, shadows, zIndex, transitions } from './tokens';

export type ThemeMode = 'light' | 'dark';

export interface Theme {
  mode: ThemeMode;
  colors: {
    background: {
      primary: string;
      secondary: string;
      tertiary: string;
      elevated: string;
      overlay: string;
    };
    text: {
      primary: string;
      secondary: string;
      tertiary: string;
      disabled: string;
      inverse: string;
    };
    border: {
      primary: string;
      secondary: string;
      focus: string;
      error: string;
    };
    interactive: {
      primary: string;
      primaryHover: string;
      primaryActive: string;
      primaryDisabled: string;
      secondary: string;
      secondaryHover: string;
      danger: string;
      dangerHover: string;
    };
    status: {
      success: string;
      warning: string;
      error: string;
      info: string;
    };
    cad: typeof colors.cad;
  };
  spacing: typeof spacing;
  typography: typeof typography;
  borderRadius: typeof borderRadius;
  shadows: typeof shadows;
  zIndex: typeof zIndex;
  transitions: typeof transitions;
}

const lightTheme: Theme = {
  mode: 'light',
  colors: {
    background: {
      primary: colors.neutral[0],
      secondary: colors.neutral[50],
      tertiary: colors.neutral[100],
      elevated: colors.neutral[0],
      overlay: 'rgba(0, 0, 0, 0.5)',
    },
    text: {
      primary: colors.neutral[900],
      secondary: colors.neutral[700],
      tertiary: colors.neutral[600],
      disabled: colors.neutral[400],
      inverse: colors.neutral[0],
    },
    border: {
      primary: colors.neutral[300],
      secondary: colors.neutral[200],
      focus: colors.primary[500],
      error: colors.error.main,
    },
    interactive: {
      primary: colors.primary[600],
      primaryHover: colors.primary[700],
      primaryActive: colors.primary[800],
      primaryDisabled: colors.neutral[300],
      secondary: colors.neutral[200],
      secondaryHover: colors.neutral[300],
      danger: colors.error.main,
      dangerHover: colors.error.dark,
    },
    status: {
      success: colors.success.main,
      warning: colors.warning.main,
      error: colors.error.main,
      info: colors.info.main,
    },
    cad: colors.cad,
  },
  spacing,
  typography,
  borderRadius,
  shadows,
  zIndex,
  transitions,
};

const darkTheme: Theme = {
  mode: 'dark',
  colors: {
    background: {
      primary: colors.neutral[900],
      secondary: colors.neutral[800],
      tertiary: colors.neutral[700],
      elevated: colors.neutral[800],
      overlay: 'rgba(0, 0, 0, 0.7)',
    },
    text: {
      primary: colors.neutral[50],
      secondary: colors.neutral[300],
      tertiary: colors.neutral[400],
      disabled: colors.neutral[600],
      inverse: colors.neutral[900],
    },
    border: {
      primary: colors.neutral[700],
      secondary: colors.neutral[800],
      focus: colors.primary[400],
      error: colors.error.light,
    },
    interactive: {
      primary: colors.primary[500],
      primaryHover: colors.primary[400],
      primaryActive: colors.primary[300],
      primaryDisabled: colors.neutral[700],
      secondary: colors.neutral[700],
      secondaryHover: colors.neutral[600],
      danger: colors.error.light,
      dangerHover: colors.error.main,
    },
    status: {
      success: colors.success.light,
      warning: colors.warning.light,
      error: colors.error.light,
      info: colors.info.light,
    },
    cad: {
      ...colors.cad,
      grid: '#4a4a4a',
    },
  },
  spacing,
  typography,
  borderRadius,
  shadows: {
    ...shadows,
    sm: '0 1px 2px 0 rgba(0, 0, 0, 0.3)',
    base: '0 1px 3px 0 rgba(0, 0, 0, 0.4), 0 1px 2px 0 rgba(0, 0, 0, 0.24)',
    md: '0 4px 6px -1px rgba(0, 0, 0, 0.4), 0 2px 4px -1px rgba(0, 0, 0, 0.24)',
    lg: '0 10px 15px -3px rgba(0, 0, 0, 0.4), 0 4px 6px -2px rgba(0, 0, 0, 0.2)',
    xl: '0 20px 25px -5px rgba(0, 0, 0, 0.4), 0 10px 10px -5px rgba(0, 0, 0, 0.16)',
    '2xl': '0 25px 50px -12px rgba(0, 0, 0, 0.5)',
  },
  zIndex,
  transitions,
};

export const themes = {
  light: lightTheme,
  dark: darkTheme,
} as const;

// Theme context and hook for React
import { createContext, useContext, useState, useCallback, ReactNode } from 'react';

interface ThemeContextValue {
  theme: Theme;
  mode: ThemeMode;
  toggleTheme: () => void;
  setTheme: (mode: ThemeMode) => void;
}

const ThemeContext = createContext<ThemeContextValue | undefined>(undefined);

export interface ThemeProviderProps {
  children: ReactNode;
  defaultMode?: ThemeMode;
}

export function ThemeProvider({ children, defaultMode = 'dark' }: ThemeProviderProps) {
  const [mode, setMode] = useState<ThemeMode>(defaultMode);

  const toggleTheme = useCallback(() => {
    setMode((prev) => (prev === 'light' ? 'dark' : 'light'));
  }, []);

  const setTheme = useCallback((newMode: ThemeMode) => {
    setMode(newMode);
  }, []);

  const value: ThemeContextValue = {
    theme: themes[mode],
    mode,
    toggleTheme,
    setTheme,
  };

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}

export function useTheme(): ThemeContextValue {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return context;
}

// CSS-in-JS helper
export function createStyles<T extends Record<string, React.CSSProperties>>(
  styles: T | ((theme: Theme) => T)
): (theme: Theme) => T {
  if (typeof styles === 'function') {
    return styles;
  }
  return () => styles;
}

// Utility function to get contrasting text color
export function getContrastText(background: string, theme: Theme): string {
  // Simple contrast calculation - in production, use a proper contrast ratio algorithm
  const rgb = background.match(/\d+/g);
  if (!rgb) return theme.colors.text.primary;

  const [r, g, b] = rgb.map(Number);
  const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255;

  return luminance > 0.5 ? theme.colors.text.primary : theme.colors.text.inverse;
}
