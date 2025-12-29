/**
 * CADDY Enterprise Dashboard Layout Component v0.4.0
 *
 * Main dashboard layout with responsive grid system, theme support,
 * and accessibility features. Provides the foundation for all dashboard views.
 */

import React, { useState, useEffect, useCallback, createContext, useContext } from 'react';
import type {
  DashboardConfig,
  DashboardLayout as LayoutConfig,
  WidgetConfig,
  ThemeMode,
  AccessibilityConfig,
  ErrorState,
  LoadingState,
} from './types';

/**
 * Dashboard context for state management
 */
interface DashboardContextValue {
  config: DashboardConfig;
  theme: ThemeMode;
  setTheme: (theme: ThemeMode) => void;
  filters: any;
  setFilters: (filters: any) => void;
  isLoading: boolean;
  error: ErrorState | null;
  refreshData: () => void;
  accessibility: AccessibilityConfig;
}

const DashboardContext = createContext<DashboardContextValue | null>(null);

/**
 * Hook to access dashboard context
 */
export const useDashboard = (): DashboardContextValue => {
  const context = useContext(DashboardContext);
  if (!context) {
    throw new Error('useDashboard must be used within DashboardLayout');
  }
  return context;
};

/**
 * Dashboard layout props
 */
export interface DashboardLayoutProps {
  /** Dashboard configuration */
  config: DashboardConfig;
  /** Layout configuration */
  layout: LayoutConfig;
  /** Child components */
  children?: React.ReactNode;
  /** Header component */
  header?: React.ReactNode;
  /** Sidebar component */
  sidebar?: React.ReactNode;
  /** Footer component */
  footer?: React.ReactNode;
  /** On layout change callback */
  onLayoutChange?: (layout: LayoutConfig) => void;
  /** On theme change callback */
  onThemeChange?: (theme: ThemeMode) => void;
  /** Custom class name */
  className?: string;
  /** Accessibility configuration */
  accessibility?: AccessibilityConfig;
}

/**
 * Main dashboard layout component
 */
export const DashboardLayout: React.FC<DashboardLayoutProps> = ({
  config,
  layout,
  children,
  header,
  sidebar,
  footer,
  onLayoutChange,
  onThemeChange,
  className = '',
  accessibility = {},
}) => {
  const [theme, setThemeState] = useState<ThemeMode>(config.theme || 'light');
  const [filters, setFilters] = useState<any>({});
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<ErrorState | null>(null);
  const [isSidebarOpen, setIsSidebarOpen] = useState<boolean>(true);
  const [screenSize, setScreenSize] = useState<'xs' | 'sm' | 'md' | 'lg' | 'xl'>('lg');

  // Accessibility settings
  const accessibilityConfig: AccessibilityConfig = {
    highContrast: false,
    reducedMotion: false,
    screenReaderOptimized: true,
    keyboardNavigation: true,
    focusIndicators: true,
    ariaLabels: true,
    fontSizeMultiplier: 1,
    ...accessibility,
  };

  /**
   * Handle theme changes
   */
  const setTheme = useCallback(
    (newTheme: ThemeMode) => {
      setThemeState(newTheme);
      if (onThemeChange) {
        onThemeChange(newTheme);
      }

      // Apply theme to document
      document.documentElement.setAttribute('data-theme', newTheme);

      // Store preference
      localStorage.setItem('dashboard-theme', newTheme);
    },
    [onThemeChange]
  );

  /**
   * Refresh all dashboard data
   */
  const refreshData = useCallback(() => {
    setIsLoading(true);
    setError(null);

    // Emit refresh event
    const event = new CustomEvent('dashboard:refresh', { detail: { config, filters } });
    window.dispatchEvent(event);

    // Simulate async refresh
    setTimeout(() => {
      setIsLoading(false);
    }, 500);
  }, [config, filters]);

  /**
   * Handle window resize for responsive breakpoints
   */
  useEffect(() => {
    const handleResize = () => {
      const width = window.innerWidth;
      if (width < 576) setScreenSize('xs');
      else if (width < 768) setScreenSize('sm');
      else if (width < 992) setScreenSize('md');
      else if (width < 1200) setScreenSize('lg');
      else setScreenSize('xl');
    };

    handleResize();
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  /**
   * Initialize theme from config or localStorage
   */
  useEffect(() => {
    const savedTheme = localStorage.getItem('dashboard-theme') as ThemeMode;
    if (savedTheme && ['light', 'dark', 'auto'].includes(savedTheme)) {
      setTheme(savedTheme);
    } else if (config.theme) {
      setTheme(config.theme);
    }
  }, [config.theme, setTheme]);

  /**
   * Handle keyboard navigation
   */
  useEffect(() => {
    if (!accessibilityConfig.keyboardNavigation) return;

    const handleKeyPress = (e: KeyboardEvent) => {
      // Toggle sidebar with Ctrl+B
      if (e.ctrlKey && e.key === 'b') {
        e.preventDefault();
        setIsSidebarOpen((prev) => !prev);
      }

      // Refresh with Ctrl+R (prevent browser refresh)
      if (e.ctrlKey && e.key === 'r') {
        e.preventDefault();
        refreshData();
      }

      // Toggle theme with Ctrl+T
      if (e.ctrlKey && e.key === 't') {
        e.preventDefault();
        setTheme(theme === 'light' ? 'dark' : 'light');
      }
    };

    window.addEventListener('keydown', handleKeyPress);
    return () => window.removeEventListener('keydown', handleKeyPress);
  }, [accessibilityConfig.keyboardNavigation, theme, setTheme, refreshData]);

  /**
   * Apply reduced motion preference
   */
  useEffect(() => {
    if (accessibilityConfig.reducedMotion) {
      document.documentElement.style.setProperty('--animation-duration', '0ms');
    } else {
      document.documentElement.style.setProperty('--animation-duration', '200ms');
    }
  }, [accessibilityConfig.reducedMotion]);

  /**
   * Apply font size multiplier
   */
  useEffect(() => {
    const multiplier = accessibilityConfig.fontSizeMultiplier || 1;
    document.documentElement.style.setProperty('--font-size-base', `${16 * multiplier}px`);
  }, [accessibilityConfig.fontSizeMultiplier]);

  /**
   * Get grid columns based on screen size
   */
  const getColumns = (): number => {
    return layout.columns[screenSize] || 12;
  };

  /**
   * Context value
   */
  const contextValue: DashboardContextValue = {
    config,
    theme,
    setTheme,
    filters,
    setFilters,
    isLoading,
    error,
    refreshData,
    accessibility: accessibilityConfig,
  };

  return (
    <DashboardContext.Provider value={contextValue}>
      <div
        className={`dashboard-layout ${className} theme-${theme} ${
          accessibilityConfig.highContrast ? 'high-contrast' : ''
        }`}
        data-theme={theme}
        data-screen-size={screenSize}
        style={{
          ...styles.layout,
          ...(accessibilityConfig.reducedMotion && { transition: 'none' }),
        }}
      >
        {/* Skip to main content link for accessibility */}
        {accessibilityConfig.screenReaderOptimized && (
          <a href="#main-content" style={styles.skipLink}>
            Skip to main content
          </a>
        )}

        {/* Header */}
        {header && (
          <header
            className="dashboard-header"
            style={styles.header}
            role="banner"
            aria-label="Dashboard header"
          >
            {header}
          </header>
        )}

        <div className="dashboard-body" style={styles.body}>
          {/* Sidebar */}
          {sidebar && (
            <>
              {/* Mobile sidebar overlay */}
              {isSidebarOpen && (screenSize === 'xs' || screenSize === 'sm') && (
                <div
                  className="sidebar-overlay"
                  style={styles.overlay}
                  onClick={() => setIsSidebarOpen(false)}
                  role="presentation"
                  aria-hidden="true"
                />
              )}

              {/* Sidebar */}
              <aside
                className={`dashboard-sidebar ${isSidebarOpen ? 'open' : 'closed'}`}
                style={{
                  ...styles.sidebar,
                  ...(isSidebarOpen ? styles.sidebarOpen : styles.sidebarClosed),
                  ...(screenSize === 'xs' || screenSize === 'sm'
                    ? styles.sidebarMobile
                    : {}),
                }}
                role="navigation"
                aria-label="Dashboard navigation"
                aria-hidden={!isSidebarOpen}
              >
                {sidebar}
              </aside>
            </>
          )}

          {/* Main content */}
          <main
            id="main-content"
            className="dashboard-main"
            style={{
              ...styles.main,
              ...(isSidebarOpen && sidebar ? styles.mainWithSidebar : {}),
            }}
            role="main"
            aria-label="Dashboard content"
          >
            {/* Loading overlay */}
            {isLoading && (
              <div
                className="loading-overlay"
                style={styles.loadingOverlay}
                role="status"
                aria-live="polite"
                aria-busy="true"
              >
                <div style={styles.loadingSpinner} aria-label="Loading">
                  <div className="spinner" />
                </div>
                <p style={styles.loadingText}>Loading dashboard data...</p>
              </div>
            )}

            {/* Error display */}
            {error && error.hasError && (
              <div
                className="error-banner"
                style={styles.errorBanner}
                role="alert"
                aria-live="assertive"
              >
                <div style={styles.errorContent}>
                  <strong>Error:</strong> {error.message || 'An error occurred'}
                  {error.retry && (
                    <button
                      onClick={error.retry}
                      style={styles.retryButton}
                      aria-label="Retry failed operation"
                    >
                      Retry
                    </button>
                  )}
                </div>
              </div>
            )}

            {/* Dashboard grid */}
            <div
              className="dashboard-grid"
              style={{
                ...styles.grid,
                gridTemplateColumns: `repeat(${getColumns()}, 1fr)`,
                gap: `${layout.gap || 16}px`,
              }}
            >
              {children}
            </div>
          </main>
        </div>

        {/* Footer */}
        {footer && (
          <footer
            className="dashboard-footer"
            style={styles.footer}
            role="contentinfo"
            aria-label="Dashboard footer"
          >
            {footer}
          </footer>
        )}

        {/* Sidebar toggle button for mobile */}
        {sidebar && (screenSize === 'xs' || screenSize === 'sm') && (
          <button
            className="sidebar-toggle"
            style={styles.sidebarToggle}
            onClick={() => setIsSidebarOpen(!isSidebarOpen)}
            aria-label={isSidebarOpen ? 'Close sidebar' : 'Open sidebar'}
            aria-expanded={isSidebarOpen}
            aria-controls="dashboard-sidebar"
          >
            <span style={styles.hamburger}>☰</span>
          </button>
        )}
      </div>
    </DashboardContext.Provider>
  );
};

/**
 * Component styles
 */
const styles: Record<string, React.CSSProperties> = {
  layout: {
    display: 'flex',
    flexDirection: 'column',
    minHeight: '100vh',
    backgroundColor: 'var(--color-background, #f5f5f5)',
    color: 'var(--color-text, #333)',
    fontFamily: 'var(--font-family, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif)',
    fontSize: 'var(--font-size-base, 16px)',
    transition: 'background-color var(--animation-duration, 200ms), color var(--animation-duration, 200ms)',
  },
  skipLink: {
    position: 'absolute',
    top: -40,
    left: 0,
    backgroundColor: '#000',
    color: '#fff',
    padding: '8px 16px',
    textDecoration: 'none',
    zIndex: 10000,
  },
  header: {
    backgroundColor: 'var(--color-surface, #fff)',
    borderBottom: '1px solid var(--color-border, #e0e0e0)',
    padding: '16px 24px',
    zIndex: 100,
  },
  body: {
    display: 'flex',
    flex: 1,
    overflow: 'hidden',
    position: 'relative',
  },
  sidebar: {
    backgroundColor: 'var(--color-surface, #fff)',
    borderRight: '1px solid var(--color-border, #e0e0e0)',
    overflowY: 'auto',
    transition: 'transform var(--animation-duration, 200ms), width var(--animation-duration, 200ms)',
    zIndex: 90,
  },
  sidebarOpen: {
    width: 280,
    transform: 'translateX(0)',
  },
  sidebarClosed: {
    width: 0,
    transform: 'translateX(-100%)',
  },
  sidebarMobile: {
    position: 'fixed',
    top: 0,
    left: 0,
    bottom: 0,
    width: 280,
    boxShadow: '2px 0 8px rgba(0, 0, 0, 0.1)',
  },
  overlay: {
    position: 'fixed',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    zIndex: 80,
  },
  main: {
    flex: 1,
    overflowY: 'auto',
    overflowX: 'hidden',
    padding: '24px',
    transition: 'margin-left var(--animation-duration, 200ms)',
  },
  mainWithSidebar: {
    marginLeft: 0,
  },
  grid: {
    display: 'grid',
    width: '100%',
    transition: 'gap var(--animation-duration, 200ms)',
  },
  footer: {
    backgroundColor: 'var(--color-surface, #fff)',
    borderTop: '1px solid var(--color-border, #e0e0e0)',
    padding: '16px 24px',
    textAlign: 'center',
    fontSize: '14px',
    color: 'var(--color-text-secondary, #666)',
  },
  loadingOverlay: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(255, 255, 255, 0.9)',
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 1000,
  },
  loadingSpinner: {
    width: 48,
    height: 48,
    border: '4px solid var(--color-border, #e0e0e0)',
    borderTop: '4px solid var(--color-primary, #1976d2)',
    borderRadius: '50%',
    animation: 'spin 1s linear infinite',
  },
  loadingText: {
    marginTop: 16,
    fontSize: 16,
    color: 'var(--color-text, #333)',
  },
  errorBanner: {
    backgroundColor: '#ffebee',
    border: '1px solid #ef5350',
    borderRadius: 4,
    padding: 16,
    marginBottom: 24,
  },
  errorContent: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    color: '#c62828',
    fontSize: 14,
  },
  retryButton: {
    backgroundColor: '#ef5350',
    color: '#fff',
    border: 'none',
    borderRadius: 4,
    padding: '8px 16px',
    cursor: 'pointer',
    fontSize: 14,
    fontWeight: 500,
    transition: 'background-color 200ms',
  },
  sidebarToggle: {
    position: 'fixed',
    bottom: 24,
    left: 24,
    width: 56,
    height: 56,
    borderRadius: '50%',
    backgroundColor: 'var(--color-primary, #1976d2)',
    color: '#fff',
    border: 'none',
    cursor: 'pointer',
    boxShadow: '0 4px 8px rgba(0, 0, 0, 0.2)',
    zIndex: 1001,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontSize: 24,
    transition: 'transform var(--animation-duration, 200ms)',
  },
  hamburger: {
    display: 'block',
    lineHeight: 1,
  },
};

/**
 * Grid item wrapper component
 */
export interface GridItemProps {
  /** Widget configuration */
  widget: WidgetConfig;
  /** Child content */
  children: React.ReactNode;
  /** Custom class name */
  className?: string;
  /** Custom styles */
  style?: React.CSSProperties;
}

export const GridItem: React.FC<GridItemProps> = ({
  widget,
  children,
  className = '',
  style = {},
}) => {
  const { accessibility } = useDashboard();

  return (
    <div
      className={`grid-item ${className}`}
      style={{
        ...gridItemStyles.container,
        gridColumn: `span ${widget.span.cols}`,
        gridRow: `span ${widget.span.rows}`,
        ...widget.style,
        ...style,
      }}
      role="region"
      aria-label={widget.title}
      tabIndex={accessibility.keyboardNavigation ? 0 : undefined}
    >
      {children}
    </div>
  );
};

const gridItemStyles: Record<string, React.CSSProperties> = {
  container: {
    backgroundColor: 'var(--color-surface, #fff)',
    borderRadius: 8,
    border: '1px solid var(--color-border, #e0e0e0)',
    padding: 24,
    transition: 'box-shadow var(--animation-duration, 200ms)',
    overflow: 'hidden',
  },
};

export default DashboardLayout;
