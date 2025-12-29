/**
 * Authentication Context Provider
 *
 * Provides authentication state and methods throughout the application
 */

import React, { createContext, useContext, useState, useEffect, useCallback } from 'react';
import type { User, Session, TokenPair, MfaStatus } from '../../../bindings/typescript/src/auth';

interface AuthContextType {
  user: User | null;
  session: Session | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  mfaStatus: MfaStatus | null;

  // Authentication methods
  login: (username: string, password: string) => Promise<void>;
  loginWithOAuth: (provider: string) => Promise<void>;
  loginWithSAML: (provider: string) => Promise<void>;
  logout: () => Promise<void>;

  // MFA methods
  verifyMfa: (code: string) => Promise<void>;
  setupMfa: () => Promise<{ qrCode: string; secret: string; recoveryCodes: string[] }>;

  // Token management
  refreshTokens: () => Promise<void>;

  // Permission checking
  hasPermission: (resource: string, action: string) => boolean;
  hasRole: (role: string) => boolean;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within AuthProvider');
  }
  return context;
};

interface AuthProviderProps {
  children: React.ReactNode;
  apiBaseUrl?: string;
}

export const AuthProvider: React.FC<AuthProviderProps> = ({
  children,
  apiBaseUrl = '/api'
}) => {
  const [user, setUser] = useState<User | null>(null);
  const [session, setSession] = useState<Session | null>(null);
  const [mfaStatus, setMfaStatus] = useState<MfaStatus | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [mfaPending, setMfaPending] = useState(false);

  // Load session from storage on mount
  useEffect(() => {
    const loadSession = async () => {
      try {
        const storedSession = localStorage.getItem('caddy_session');
        if (storedSession) {
          const parsedSession = JSON.parse(storedSession) as Session;

          // Validate session
          const response = await fetch(`${apiBaseUrl}/auth/validate`, {
            headers: {
              'Authorization': `Bearer ${parsedSession.accessToken}`
            }
          });

          if (response.ok) {
            const userData = await response.json();
            setUser(userData.user);
            setSession(parsedSession);

            // Load MFA status
            const mfaResponse = await fetch(`${apiBaseUrl}/auth/mfa/status`, {
              headers: {
                'Authorization': `Bearer ${parsedSession.accessToken}`
              }
            });
            if (mfaResponse.ok) {
              setMfaStatus(await mfaResponse.json());
            }
          } else {
            // Session invalid, clear it
            localStorage.removeItem('caddy_session');
          }
        }
      } catch (error) {
        console.error('Failed to load session:', error);
        localStorage.removeItem('caddy_session');
      } finally {
        setIsLoading(false);
      }
    };

    loadSession();
  }, [apiBaseUrl]);

  // Auto-refresh tokens before expiration
  useEffect(() => {
    if (!session) return;

    const expiresAt = session.expiresAt * 1000;
    const now = Date.now();
    const timeUntilExpiry = expiresAt - now;

    // Refresh 5 minutes before expiration
    const refreshTime = Math.max(0, timeUntilExpiry - 5 * 60 * 1000);

    const timer = setTimeout(async () => {
      try {
        await refreshTokens();
      } catch (error) {
        console.error('Auto-refresh failed:', error);
        await logout();
      }
    }, refreshTime);

    return () => clearTimeout(timer);
  }, [session]);

  const login = useCallback(async (username: string, password: string) => {
    try {
      const response = await fetch(`${apiBaseUrl}/auth/login`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ username, password }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.message || 'Login failed');
      }

      const data = await response.json();

      // Check if MFA is required
      if (data.mfaRequired) {
        setMfaPending(true);
        // Store temporary token for MFA verification
        sessionStorage.setItem('mfa_temp_token', data.tempToken);
        return;
      }

      // Set user and session
      setUser(data.user);
      setSession(data.session);
      setMfaStatus(data.mfaStatus);

      // Store session
      localStorage.setItem('caddy_session', JSON.stringify(data.session));
    } catch (error) {
      console.error('Login error:', error);
      throw error;
    }
  }, [apiBaseUrl]);

  const loginWithOAuth = useCallback(async (provider: string) => {
    try {
      // Redirect to OAuth provider
      const response = await fetch(`${apiBaseUrl}/auth/oauth/${provider}/authorize`);
      const data = await response.json();

      // Redirect to authorization URL
      window.location.href = data.authorizationUrl;
    } catch (error) {
      console.error('OAuth login error:', error);
      throw error;
    }
  }, [apiBaseUrl]);

  const loginWithSAML = useCallback(async (provider: string) => {
    try {
      // Redirect to SAML provider
      const response = await fetch(`${apiBaseUrl}/auth/saml/${provider}/login`);
      const data = await response.json();

      // Redirect to SSO URL
      window.location.href = data.ssoUrl;
    } catch (error) {
      console.error('SAML login error:', error);
      throw error;
    }
  }, [apiBaseUrl]);

  const verifyMfa = useCallback(async (code: string) => {
    try {
      const tempToken = sessionStorage.getItem('mfa_temp_token');
      if (!tempToken) {
        throw new Error('No MFA session found');
      }

      const response = await fetch(`${apiBaseUrl}/auth/mfa/verify`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${tempToken}`
        },
        body: JSON.stringify({ code }),
      });

      if (!response.ok) {
        throw new Error('MFA verification failed');
      }

      const data = await response.json();

      setUser(data.user);
      setSession(data.session);
      setMfaStatus(data.mfaStatus);
      setMfaPending(false);

      // Store session and clear temp token
      localStorage.setItem('caddy_session', JSON.stringify(data.session));
      sessionStorage.removeItem('mfa_temp_token');
    } catch (error) {
      console.error('MFA verification error:', error);
      throw error;
    }
  }, [apiBaseUrl]);

  const setupMfa = useCallback(async () => {
    if (!session) throw new Error('Not authenticated');

    try {
      const response = await fetch(`${apiBaseUrl}/auth/mfa/setup`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${session.accessToken}`
        },
      });

      if (!response.ok) {
        throw new Error('MFA setup failed');
      }

      return await response.json();
    } catch (error) {
      console.error('MFA setup error:', error);
      throw error;
    }
  }, [apiBaseUrl, session]);

  const logout = useCallback(async () => {
    try {
      if (session) {
        await fetch(`${apiBaseUrl}/auth/logout`, {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${session.accessToken}`
          },
        });
      }
    } catch (error) {
      console.error('Logout error:', error);
    } finally {
      setUser(null);
      setSession(null);
      setMfaStatus(null);
      localStorage.removeItem('caddy_session');
      sessionStorage.removeItem('mfa_temp_token');
    }
  }, [apiBaseUrl, session]);

  const refreshTokens = useCallback(async () => {
    if (!session?.refreshToken) {
      throw new Error('No refresh token available');
    }

    try {
      const response = await fetch(`${apiBaseUrl}/auth/refresh`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ refreshToken: session.refreshToken }),
      });

      if (!response.ok) {
        throw new Error('Token refresh failed');
      }

      const tokenPair: TokenPair = await response.json();

      const newSession: Session = {
        ...session,
        accessToken: tokenPair.accessToken,
        refreshToken: tokenPair.refreshToken,
        expiresAt: Math.floor(Date.now() / 1000) + tokenPair.expiresIn,
      };

      setSession(newSession);
      localStorage.setItem('caddy_session', JSON.stringify(newSession));
    } catch (error) {
      console.error('Token refresh error:', error);
      throw error;
    }
  }, [apiBaseUrl, session]);

  const hasPermission = useCallback((resource: string, action: string) => {
    if (!user) return false;

    // Admin has all permissions
    if (user.roles.includes('admin')) return true;

    // Check user's roles for permission
    // This is a simplified check - in production, fetch from backend
    return false;
  }, [user]);

  const hasRole = useCallback((role: string) => {
    if (!user) return false;
    return user.roles.includes(role);
  }, [user]);

  const value: AuthContextType = {
    user,
    session,
    isAuthenticated: !!user && !mfaPending,
    isLoading,
    mfaStatus,
    login,
    loginWithOAuth,
    loginWithSAML,
    logout,
    verifyMfa,
    setupMfa,
    refreshTokens,
    hasPermission,
    hasRole,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};

export default AuthProvider;
