/**
 * CADDY v0.3.0 - API Client
 * Enterprise-grade API communication layer
 */

import type {
  APIResponse,
  AuthResponse,
  ScanResult,
  SyncRequest,
  SyncResponse,
  UserSettings,
} from './types';

// ============================================================================
// Configuration
// ============================================================================

const DEFAULT_API_ENDPOINT = 'https://api.caddy.dev/v1';
const API_TIMEOUT = 30000; // 30 seconds
const MAX_RETRIES = 3;
const RETRY_DELAY = 1000; // 1 second

// ============================================================================
// API Client Class
// ============================================================================

export class CaddyAPI {
  private baseUrl: string;
  private token: string | null = null;
  private retryCount: number = 0;

  constructor(baseUrl: string = DEFAULT_API_ENDPOINT) {
    this.baseUrl = baseUrl;
    this.loadToken();
  }

  // --------------------------------------------------------------------------
  // Authentication
  // --------------------------------------------------------------------------

  async authenticate(email: string, password: string): Promise<AuthResponse> {
    const response = await this.request<AuthResponse>('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ email, password }),
    });

    if (response.success && response.data) {
      this.token = response.data.token;
      await this.saveToken(response.data.token);
      return response.data;
    }

    throw new Error(response.error || 'Authentication failed');
  }

  async authenticateWithApiKey(apiKey: string): Promise<AuthResponse> {
    const response = await this.request<AuthResponse>('/auth/api-key', {
      method: 'POST',
      body: JSON.stringify({ apiKey }),
    });

    if (response.success && response.data) {
      this.token = response.data.token;
      await this.saveToken(response.data.token);
      return response.data;
    }

    throw new Error(response.error || 'API key authentication failed');
  }

  async refreshToken(): Promise<void> {
    const response = await this.request<AuthResponse>('/auth/refresh', {
      method: 'POST',
    });

    if (response.success && response.data) {
      this.token = response.data.token;
      await this.saveToken(response.data.token);
    } else {
      this.logout();
      throw new Error('Token refresh failed');
    }
  }

  logout(): void {
    this.token = null;
    chrome.storage.local.remove(['auth_token']);
  }

  isAuthenticated(): boolean {
    return this.token !== null;
  }

  // --------------------------------------------------------------------------
  // Scan Results
  // --------------------------------------------------------------------------

  async uploadScanResult(result: ScanResult): Promise<void> {
    const response = await this.request('/scans', {
      method: 'POST',
      body: JSON.stringify(result),
    });

    if (!response.success) {
      throw new Error(response.error || 'Failed to upload scan result');
    }
  }

  async getScanResults(options?: {
    limit?: number;
    offset?: number;
    url?: string;
  }): Promise<ScanResult[]> {
    const params = new URLSearchParams();
    if (options?.limit) params.set('limit', options.limit.toString());
    if (options?.offset) params.set('offset', options.offset.toString());
    if (options?.url) params.set('url', options.url);

    const response = await this.request<ScanResult[]>(
      `/scans?${params.toString()}`
    );

    if (response.success && response.data) {
      return response.data;
    }

    throw new Error(response.error || 'Failed to fetch scan results');
  }

  async getScanResult(id: string): Promise<ScanResult> {
    const response = await this.request<ScanResult>(`/scans/${id}`);

    if (response.success && response.data) {
      return response.data;
    }

    throw new Error(response.error || 'Failed to fetch scan result');
  }

  async deleteScanResult(id: string): Promise<void> {
    const response = await this.request(`/scans/${id}`, {
      method: 'DELETE',
    });

    if (!response.success) {
      throw new Error(response.error || 'Failed to delete scan result');
    }
  }

  // --------------------------------------------------------------------------
  // Synchronization
  // --------------------------------------------------------------------------

  async syncData(data: SyncRequest): Promise<SyncResponse> {
    const response = await this.request<SyncResponse>('/sync', {
      method: 'POST',
      body: JSON.stringify(data),
    });

    if (response.success && response.data) {
      return response.data;
    }

    throw new Error(response.error || 'Synchronization failed');
  }

  async getSettings(): Promise<UserSettings> {
    const response = await this.request<UserSettings>('/settings');

    if (response.success && response.data) {
      return response.data;
    }

    throw new Error(response.error || 'Failed to fetch settings');
  }

  async updateSettings(settings: Partial<UserSettings>): Promise<void> {
    const response = await this.request('/settings', {
      method: 'PUT',
      body: JSON.stringify(settings),
    });

    if (!response.success) {
      throw new Error(response.error || 'Failed to update settings');
    }
  }

  // --------------------------------------------------------------------------
  // Export
  // --------------------------------------------------------------------------

  async exportResults(
    resultIds: string[],
    format: string
  ): Promise<Blob> {
    const response = await fetch(`${this.baseUrl}/export`, {
      method: 'POST',
      headers: this.getHeaders(),
      body: JSON.stringify({ resultIds, format }),
    });

    if (!response.ok) {
      throw new Error('Export failed');
    }

    return response.blob();
  }

  // --------------------------------------------------------------------------
  // Analytics
  // --------------------------------------------------------------------------

  async trackEvent(event: string, properties?: Record<string, any>): Promise<void> {
    try {
      await this.request('/analytics/events', {
        method: 'POST',
        body: JSON.stringify({ event, properties, timestamp: Date.now() }),
      });
    } catch (error) {
      // Silently fail analytics
      console.warn('Analytics tracking failed:', error);
    }
  }

  // --------------------------------------------------------------------------
  // Private Methods
  // --------------------------------------------------------------------------

  private async request<T = any>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<APIResponse<T>> {
    const url = `${this.baseUrl}${endpoint}`;
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), API_TIMEOUT);

    try {
      const response = await fetch(url, {
        ...options,
        headers: this.getHeaders(options.headers),
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      if (response.status === 401) {
        // Token expired, try to refresh
        if (this.retryCount < MAX_RETRIES) {
          this.retryCount++;
          await this.refreshToken();
          return this.request<T>(endpoint, options);
        } else {
          this.logout();
          throw new Error('Authentication required');
        }
      }

      this.retryCount = 0;

      const data = await response.json();

      if (!response.ok) {
        return {
          success: false,
          error: data.error || `HTTP ${response.status}`,
          timestamp: Date.now(),
        };
      }

      return {
        success: true,
        data,
        timestamp: Date.now(),
      };
    } catch (error) {
      clearTimeout(timeoutId);

      if (error instanceof Error) {
        if (error.name === 'AbortError') {
          return {
            success: false,
            error: 'Request timeout',
            timestamp: Date.now(),
          };
        }

        return {
          success: false,
          error: error.message,
          timestamp: Date.now(),
        };
      }

      return {
        success: false,
        error: 'Unknown error occurred',
        timestamp: Date.now(),
      };
    }
  }

  private getHeaders(additionalHeaders?: HeadersInit): HeadersInit {
    const headers: HeadersInit = {
      'Content-Type': 'application/json',
      'X-Extension-Version': '0.3.0',
      ...additionalHeaders,
    };

    if (this.token) {
      headers['Authorization'] = `Bearer ${this.token}`;
    }

    return headers;
  }

  private async loadToken(): Promise<void> {
    const result = await chrome.storage.local.get(['auth_token']);
    if (result.auth_token) {
      this.token = result.auth_token;
    }
  }

  private async saveToken(token: string): Promise<void> {
    await chrome.storage.local.set({ auth_token: token });
  }
}

// ============================================================================
// Singleton Instance
// ============================================================================

export const api = new CaddyAPI();

// ============================================================================
// Helper Functions
// ============================================================================

export async function getApiEndpoint(): Promise<string> {
  const result = await chrome.storage.local.get(['api_endpoint']);
  return result.api_endpoint || DEFAULT_API_ENDPOINT;
}

export async function setApiEndpoint(endpoint: string): Promise<void> {
  await chrome.storage.local.set({ api_endpoint: endpoint });
}

export async function isOnline(): Promise<boolean> {
  try {
    const response = await fetch('https://api.caddy.dev/health', {
      method: 'HEAD',
      cache: 'no-cache',
    });
    return response.ok;
  } catch {
    return false;
  }
}
