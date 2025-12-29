/**
 * Authentication Middleware
 *
 * Route protection and permission checking middleware for React Router
 */

import type { Permission } from '../../../bindings/typescript/src/auth';

export interface AuthMiddlewareConfig {
  requireAuth?: boolean;
  requireMfa?: boolean;
  requiredRoles?: string[];
  requiredPermissions?: Permission[];
  redirectTo?: string;
}

export interface AuthState {
  isAuthenticated: boolean;
  user: any;
  hasRole: (role: string) => boolean;
  hasPermission: (resource: string, action: string) => boolean;
  mfaStatus: any;
}

/**
 * Check if user can access a route based on middleware config
 */
export function canAccessRoute(
  authState: AuthState,
  config: AuthMiddlewareConfig
): { allowed: boolean; reason?: string } {
  // Check authentication requirement
  if (config.requireAuth && !authState.isAuthenticated) {
    return { allowed: false, reason: 'Authentication required' };
  }

  // Check MFA requirement
  if (config.requireMfa && authState.mfaStatus && !authState.mfaStatus.totpEnabled) {
    return { allowed: false, reason: 'Multi-factor authentication required' };
  }

  // Check required roles
  if (config.requiredRoles && config.requiredRoles.length > 0) {
    const hasRequiredRole = config.requiredRoles.some(role =>
      authState.hasRole(role)
    );

    if (!hasRequiredRole) {
      return {
        allowed: false,
        reason: `Required role: ${config.requiredRoles.join(' or ')}`
      };
    }
  }

  // Check required permissions
  if (config.requiredPermissions && config.requiredPermissions.length > 0) {
    const hasAllPermissions = config.requiredPermissions.every(perm =>
      authState.hasPermission(perm.resource, perm.action)
    );

    if (!hasAllPermissions) {
      return { allowed: false, reason: 'Insufficient permissions' };
    }
  }

  return { allowed: true };
}

/**
 * Higher-order component for protecting routes
 *
 * Usage:
 * ```tsx
 * const ProtectedPage = withAuth(MyPage, {
 *   requireAuth: true,
 *   requiredRoles: ['admin'],
 * });
 * ```
 */
export function withAuth<P extends object>(
  Component: React.ComponentType<P>,
  config: AuthMiddlewareConfig = {}
) {
  return function AuthProtectedComponent(props: P) {
    const React = require('react');
    const { useAuth } = require('./AuthProvider');
    const { useNavigate } = require('react-router-dom');

    const auth = useAuth();
    const navigate = useNavigate();

    React.useEffect(() => {
      const authState: AuthState = {
        isAuthenticated: auth.isAuthenticated,
        user: auth.user,
        hasRole: auth.hasRole,
        hasPermission: auth.hasPermission,
        mfaStatus: auth.mfaStatus,
      };

      const { allowed, reason } = canAccessRoute(authState, config);

      if (!allowed) {
        console.warn(`Access denied: ${reason}`);
        navigate(config.redirectTo || '/login');
      }
    }, [auth, navigate]);

    // Show loading state while checking authentication
    if (config.requireAuth && auth.isLoading) {
      return React.createElement('div', {
        className: 'flex items-center justify-center min-h-screen'
      }, 'Loading...');
    }

    const authState: AuthState = {
      isAuthenticated: auth.isAuthenticated,
      user: auth.user,
      hasRole: auth.hasRole,
      hasPermission: auth.hasPermission,
      mfaStatus: auth.mfaStatus,
    };

    const { allowed } = canAccessRoute(authState, config);

    if (!allowed) {
      return null;
    }

    return React.createElement(Component, props);
  };
}

/**
 * React Router loader function for route protection
 *
 * Usage in React Router v6:
 * ```tsx
 * {
 *   path: '/admin',
 *   element: <AdminPage />,
 *   loader: createAuthLoader({ requiredRoles: ['admin'] }),
 * }
 * ```
 */
export function createAuthLoader(config: AuthMiddlewareConfig = {}) {
  return async ({ request }: { request: Request }) => {
    // Get auth state from somewhere (e.g., session storage, context)
    const sessionData = localStorage.getItem('caddy_session');

    if (!sessionData && config.requireAuth) {
      throw new Response('Unauthorized', { status: 401 });
    }

    // Additional checks would go here based on config
    // This is a simplified version

    return null;
  };
}

/**
 * Permission checking utility
 */
export class PermissionChecker {
  constructor(private userPermissions: Permission[]) {}

  /**
   * Check if user has a specific permission
   */
  has(resource: string, action: string, scope?: string): boolean {
    return this.userPermissions.some(perm => {
      const resourceMatch = perm.resource === '*' || perm.resource === resource;
      const actionMatch = perm.action === '*' || perm.action === action;
      const scopeMatch = !scope || !perm.scope || perm.scope === '*' || perm.scope === scope;

      return resourceMatch && actionMatch && scopeMatch;
    });
  }

  /**
   * Check if user has any of the given permissions
   */
  hasAny(permissions: Permission[]): boolean {
    return permissions.some(perm =>
      this.has(perm.resource, perm.action, perm.scope)
    );
  }

  /**
   * Check if user has all of the given permissions
   */
  hasAll(permissions: Permission[]): boolean {
    return permissions.every(perm =>
      this.has(perm.resource, perm.action, perm.scope)
    );
  }

  /**
   * Filter items based on permission
   */
  filter<T>(
    items: T[],
    getPermission: (item: T) => Permission
  ): T[] {
    return items.filter(item => {
      const perm = getPermission(item);
      return this.has(perm.resource, perm.action, perm.scope);
    });
  }
}

/**
 * Role checking utility
 */
export class RoleChecker {
  constructor(
    private userRoles: string[],
    private roleHierarchy: Map<string, string[]> = new Map()
  ) {}

  /**
   * Check if user has a specific role
   */
  has(role: string): boolean {
    if (this.userRoles.includes(role)) {
      return true;
    }

    // Check role hierarchy
    return this.userRoles.some(userRole => {
      const parentRoles = this.roleHierarchy.get(userRole) || [];
      return parentRoles.includes(role);
    });
  }

  /**
   * Check if user has any of the given roles
   */
  hasAny(roles: string[]): boolean {
    return roles.some(role => this.has(role));
  }

  /**
   * Check if user has all of the given roles
   */
  hasAll(roles: string[]): boolean {
    return roles.every(role => this.has(role));
  }

  /**
   * Check if user has admin role
   */
  isAdmin(): boolean {
    return this.has('admin') || this.has('administrator');
  }
}

/**
 * Session token management utilities
 */
export class TokenManager {
  private static ACCESS_TOKEN_KEY = 'caddy_access_token';
  private static REFRESH_TOKEN_KEY = 'caddy_refresh_token';

  /**
   * Store tokens
   */
  static setTokens(accessToken: string, refreshToken: string): void {
    sessionStorage.setItem(this.ACCESS_TOKEN_KEY, accessToken);
    localStorage.setItem(this.REFRESH_TOKEN_KEY, refreshToken);
  }

  /**
   * Get access token
   */
  static getAccessToken(): string | null {
    return sessionStorage.getItem(this.ACCESS_TOKEN_KEY);
  }

  /**
   * Get refresh token
   */
  static getRefreshToken(): string | null {
    return localStorage.getItem(this.REFRESH_TOKEN_KEY);
  }

  /**
   * Clear tokens
   */
  static clearTokens(): void {
    sessionStorage.removeItem(this.ACCESS_TOKEN_KEY);
    localStorage.removeItem(this.REFRESH_TOKEN_KEY);
  }

  /**
   * Check if token is expired
   */
  static isTokenExpired(token: string): boolean {
    try {
      const payload = JSON.parse(atob(token.split('.')[1]));
      return Date.now() >= payload.exp * 1000;
    } catch {
      return true;
    }
  }
}
