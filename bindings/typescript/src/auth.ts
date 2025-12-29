/**
 * CADDY Enterprise Authentication TypeScript Bindings
 *
 * TypeScript bindings for the enterprise authentication system including:
 * - OAuth 2.0 / OpenID Connect
 * - SAML 2.0 SSO
 * - JWT token management
 * - RBAC (Role-Based Access Control)
 * - Multi-Factor Authentication (MFA)
 * - Session management
 */

// ============================================================================
// OAuth 2.0 / OIDC Types
// ============================================================================

export interface OAuth2Config {
  providerName: string;
  clientId: string;
  clientSecret?: string;
  authorizationEndpoint: string;
  tokenEndpoint: string;
  userinfoEndpoint?: string;
  redirectUri: string;
  scopes: string[];
  usePkce: boolean;
}

export interface OAuth2Token {
  accessToken: string;
  tokenType: string;
  expiresIn?: number;
  refreshToken?: string;
  idToken?: string;
  scope?: string;
}

export interface IDTokenClaims {
  iss: string;
  sub: string;
  aud: string;
  exp: number;
  iat: number;
  email?: string;
  emailVerified?: boolean;
  name?: string;
  preferredUsername?: string;
  [key: string]: any;
}

export interface UserInfo {
  sub: string;
  email?: string;
  emailVerified?: boolean;
  name?: string;
  givenName?: string;
  familyName?: string;
  picture?: string;
  locale?: string;
  [key: string]: any;
}

// ============================================================================
// SAML 2.0 Types
// ============================================================================

export interface SamlConfig {
  entityId: string;
  acsUrl: string;
  sloUrl?: string;
  idpEntityId: string;
  idpSsoUrl: string;
  idpCertificate: string;
  requireSignedAssertions: boolean;
  requireSignedResponses: boolean;
}

export interface SamlUser {
  nameId: string;
  email?: string;
  name?: string;
  givenName?: string;
  familyName?: string;
  attributes: Record<string, string[]>;
  sessionIndex?: string;
}

export type NameIDFormat =
  | "EmailAddress"
  | "Persistent"
  | "Transient"
  | "Unspecified";

// ============================================================================
// JWT Types
// ============================================================================

export interface JwtConfig {
  secret: string;
  algorithm: JwtAlgorithm;
  accessTokenTtl: number;
  refreshTokenTtl: number;
  issuer: string;
  audience: string;
  enableFingerprinting: boolean;
  enableTokenRotation: boolean;
}

export type JwtAlgorithm =
  | "HS256" | "HS384" | "HS512"
  | "RS256" | "RS384" | "RS512"
  | "ES256" | "ES384";

export interface TokenClaims {
  sub: string;
  exp: number;
  iat: number;
  nbf: number;
  iss: string;
  aud: string;
  jti: string;
  type: "access" | "refresh";
  username?: string;
  email?: string;
  roles?: string[];
  permissions?: string[];
  fingerprint?: string;
  sessionId?: string;
  ip?: string;
  userAgent?: string;
  [key: string]: any;
}

export interface TokenPair {
  accessToken: string;
  refreshToken: string;
  tokenType: string;
  expiresIn: number;
  refreshExpiresIn: number;
}

export interface ClientInfo {
  ipAddress: string;
  userAgent: string;
}

// ============================================================================
// RBAC Types
// ============================================================================

export interface Permission {
  resource: string;
  action: string;
  scope?: string;
}

export interface Role {
  id: string;
  name: string;
  description: string;
  permissions: Permission[];
  parents: string[];
  constraints: RoleConstraint[];
  isSystem: boolean;
}

export type RoleConstraint =
  | { type: "TimeWindow"; startHour: number; endHour: number }
  | { type: "DateRange"; start: number; end: number }
  | { type: "IpWhitelist"; allowedIps: string[] }
  | { type: "Location"; allowedLocations: string[] }
  | { type: "MfaRequired" }
  | { type: "MaxSessions"; limit: number };

export interface AccessContext {
  currentHour?: number;
  ipAddress?: string;
  location?: string;
  mfaVerified: boolean;
  activeSessions: number;
  metadata: Record<string, string>;
}

export interface Delegation {
  id: string;
  delegator: string;
  delegate: string;
  permissions: Permission[];
  expiresAt: number;
  createdAt: number;
  active: boolean;
}

// ============================================================================
// MFA Types
// ============================================================================

export interface TotpConfig {
  secret: string;
  issuer: string;
  accountName: string;
  timeStep: number;
  digits: number;
  algorithm: TotpAlgorithm;
  enabled: boolean;
}

export type TotpAlgorithm = "SHA1" | "SHA256" | "SHA512";

export interface WebAuthnConfig {
  rpId: string;
  rpName: string;
  origin: string;
  timeout: number;
}

export interface WebAuthnCredential {
  id: Uint8Array;
  publicKey: Uint8Array;
  signCount: number;
  userHandle: Uint8Array;
  name: string;
  createdAt: number;
  lastUsed?: number;
}

export interface MfaStatus {
  totpEnabled: boolean;
  webauthnCredentials: number;
  recoveryCodes: number;
  isLockedOut: boolean;
}

// ============================================================================
// User & Session Types
// ============================================================================

export interface User {
  id: string;
  username: string;
  email: string;
  roles: string[];
  status: UserStatus;
  mfaEnabled: boolean;
  createdAt: number;
  lastLogin?: number;
}

export type UserStatus = "Active" | "Inactive" | "Locked" | "Suspended";

export interface Session {
  id: string;
  userId: string;
  accessToken: string;
  refreshToken: string;
  expiresAt: number;
  ipAddress?: string;
  userAgent?: string;
  createdAt: number;
}

// ============================================================================
// API Key Types
// ============================================================================

export interface ApiKey {
  id: string;
  keyHash: string;
  userId: string;
  name: string;
  scopes: string[];
  createdAt: number;
  lastUsed?: number;
  expiresAt?: number;
  active: boolean;
}

// ============================================================================
// Authentication Client Interface
// ============================================================================

export interface AuthClient {
  // OAuth 2.0 / OIDC
  oauth2Login(config: OAuth2Config): Promise<string>;
  oauth2Callback(code: string, state: string): Promise<OAuth2Token>;
  oauth2Refresh(refreshToken: string): Promise<OAuth2Token>;
  getUserInfo(accessToken: string): Promise<UserInfo>;

  // SAML 2.0
  samlLogin(config: SamlConfig): Promise<string>;
  samlCallback(samlResponse: string): Promise<SamlUser>;

  // JWT
  createTokenPair(
    userId: string,
    username?: string,
    email?: string,
    roles?: string[],
    clientInfo?: ClientInfo
  ): Promise<TokenPair>;
  verifyAccessToken(token: string): Promise<TokenClaims>;
  refreshTokens(refreshToken: string, clientInfo?: ClientInfo): Promise<TokenPair>;
  revokeToken(jti: string): Promise<void>;

  // RBAC
  checkPermission(userId: string, permission: Permission, context: AccessContext): Promise<boolean>;
  assignRole(userId: string, roleId: string): Promise<void>;
  removeRole(userId: string, roleId: string): Promise<void>;
  createDelegation(
    delegator: string,
    delegate: string,
    permissions: Permission[],
    durationSeconds: number
  ): Promise<string>;

  // MFA
  enableTotp(userId: string, issuer: string, accountName: string): Promise<TotpConfig>;
  confirmTotp(userId: string, code: string): Promise<string[]>;
  verifyTotp(userId: string, code: string): Promise<void>;
  verifyRecoveryCode(userId: string, code: string): Promise<void>;
  getMfaStatus(userId: string): Promise<MfaStatus>;

  // WebAuthn
  startWebAuthnRegistration(userId: string): Promise<Uint8Array>;
  completeWebAuthnRegistration(
    userId: string,
    credentialId: Uint8Array,
    publicKey: Uint8Array,
    name: string
  ): Promise<void>;
  startWebAuthnAuthentication(userId: string): Promise<Uint8Array>;
  completeWebAuthnAuthentication(
    userId: string,
    credentialId: Uint8Array
  ): Promise<void>;

  // Session management
  login(username: string, password: string, clientInfo?: ClientInfo): Promise<Session>;
  logout(sessionId: string): Promise<void>;
  validateSession(sessionId: string): Promise<boolean>;

  // API Keys
  createApiKey(userId: string, name: string, scopes: string[]): Promise<{ apiKey: ApiKey; key: string }>;
  verifyApiKey(key: string): Promise<ApiKey | null>;
  revokeApiKey(keyId: string): Promise<void>;
}

// ============================================================================
// Error Types
// ============================================================================

export class AuthError extends Error {
  constructor(
    message: string,
    public code: string,
    public statusCode?: number
  ) {
    super(message);
    this.name = "AuthError";
  }
}

export class OAuth2Error extends AuthError {
  constructor(message: string, statusCode?: number) {
    super(message, "OAUTH2_ERROR", statusCode);
    this.name = "OAuth2Error";
  }
}

export class SamlError extends AuthError {
  constructor(message: string, statusCode?: number) {
    super(message, "SAML_ERROR", statusCode);
    this.name = "SamlError";
  }
}

export class JwtError extends AuthError {
  constructor(message: string, statusCode?: number) {
    super(message, "JWT_ERROR", statusCode);
    this.name = "JwtError";
  }
}

export class MfaError extends AuthError {
  constructor(message: string, statusCode?: number) {
    super(message, "MFA_ERROR", statusCode);
    this.name = "MfaError";
  }
}

export class RbacError extends AuthError {
  constructor(message: string, statusCode?: number) {
    super(message, "RBAC_ERROR", statusCode);
    this.name = "RbacError";
  }
}

// ============================================================================
// Utility Functions
// ============================================================================

export function parseJwt(token: string): TokenClaims | null {
  try {
    const base64Url = token.split('.')[1];
    const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
    const jsonPayload = decodeURIComponent(
      atob(base64)
        .split('')
        .map(c => '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2))
        .join('')
    );
    return JSON.parse(jsonPayload);
  } catch (error) {
    return null;
  }
}

export function isTokenExpired(token: string): boolean {
  const claims = parseJwt(token);
  if (!claims || !claims.exp) return true;

  return Date.now() >= claims.exp * 1000;
}

export function getTokenExpirationTime(token: string): Date | null {
  const claims = parseJwt(token);
  if (!claims || !claims.exp) return null;

  return new Date(claims.exp * 1000);
}

export function formatTotpCode(code: string): string {
  return code.replace(/\D/g, '').slice(0, 6);
}

export function formatRecoveryCode(code: string): string {
  const cleaned = code.replace(/[^A-Z0-9]/g, '').toUpperCase();
  if (cleaned.length >= 4) {
    return `${cleaned.slice(0, 4)}-${cleaned.slice(4, 8)}`;
  }
  return cleaned;
}

export function permissionToString(permission: Permission): string {
  if (permission.scope) {
    return `${permission.resource}:${permission.action}:${permission.scope}`;
  }
  return `${permission.resource}:${permission.action}`;
}

export function permissionFromString(str: string): Permission | null {
  const parts = str.split(':');
  if (parts.length === 2) {
    return {
      resource: parts[0],
      action: parts[1],
    };
  } else if (parts.length === 3) {
    return {
      resource: parts[0],
      action: parts[1],
      scope: parts[2],
    };
  }
  return null;
}
