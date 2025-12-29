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
export type NameIDFormat = "EmailAddress" | "Persistent" | "Transient" | "Unspecified";
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
export type JwtAlgorithm = "HS256" | "HS384" | "HS512" | "RS256" | "RS384" | "RS512" | "ES256" | "ES384";
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
export type RoleConstraint = {
    type: "TimeWindow";
    startHour: number;
    endHour: number;
} | {
    type: "DateRange";
    start: number;
    end: number;
} | {
    type: "IpWhitelist";
    allowedIps: string[];
} | {
    type: "Location";
    allowedLocations: string[];
} | {
    type: "MfaRequired";
} | {
    type: "MaxSessions";
    limit: number;
};
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
export interface AuthClient {
    oauth2Login(config: OAuth2Config): Promise<string>;
    oauth2Callback(code: string, state: string): Promise<OAuth2Token>;
    oauth2Refresh(refreshToken: string): Promise<OAuth2Token>;
    getUserInfo(accessToken: string): Promise<UserInfo>;
    samlLogin(config: SamlConfig): Promise<string>;
    samlCallback(samlResponse: string): Promise<SamlUser>;
    createTokenPair(userId: string, username?: string, email?: string, roles?: string[], clientInfo?: ClientInfo): Promise<TokenPair>;
    verifyAccessToken(token: string): Promise<TokenClaims>;
    refreshTokens(refreshToken: string, clientInfo?: ClientInfo): Promise<TokenPair>;
    revokeToken(jti: string): Promise<void>;
    checkPermission(userId: string, permission: Permission, context: AccessContext): Promise<boolean>;
    assignRole(userId: string, roleId: string): Promise<void>;
    removeRole(userId: string, roleId: string): Promise<void>;
    createDelegation(delegator: string, delegate: string, permissions: Permission[], durationSeconds: number): Promise<string>;
    enableTotp(userId: string, issuer: string, accountName: string): Promise<TotpConfig>;
    confirmTotp(userId: string, code: string): Promise<string[]>;
    verifyTotp(userId: string, code: string): Promise<void>;
    verifyRecoveryCode(userId: string, code: string): Promise<void>;
    getMfaStatus(userId: string): Promise<MfaStatus>;
    startWebAuthnRegistration(userId: string): Promise<Uint8Array>;
    completeWebAuthnRegistration(userId: string, credentialId: Uint8Array, publicKey: Uint8Array, name: string): Promise<void>;
    startWebAuthnAuthentication(userId: string): Promise<Uint8Array>;
    completeWebAuthnAuthentication(userId: string, credentialId: Uint8Array): Promise<void>;
    login(username: string, password: string, clientInfo?: ClientInfo): Promise<Session>;
    logout(sessionId: string): Promise<void>;
    validateSession(sessionId: string): Promise<boolean>;
    createApiKey(userId: string, name: string, scopes: string[]): Promise<{
        apiKey: ApiKey;
        key: string;
    }>;
    verifyApiKey(key: string): Promise<ApiKey | null>;
    revokeApiKey(keyId: string): Promise<void>;
}
export declare class AuthError extends Error {
    code: string;
    statusCode?: number | undefined;
    constructor(message: string, code: string, statusCode?: number | undefined);
}
export declare class OAuth2Error extends AuthError {
    constructor(message: string, statusCode?: number);
}
export declare class SamlError extends AuthError {
    constructor(message: string, statusCode?: number);
}
export declare class JwtError extends AuthError {
    constructor(message: string, statusCode?: number);
}
export declare class MfaError extends AuthError {
    constructor(message: string, statusCode?: number);
}
export declare class RbacError extends AuthError {
    constructor(message: string, statusCode?: number);
}
export declare function parseJwt(token: string): TokenClaims | null;
export declare function isTokenExpired(token: string): boolean;
export declare function getTokenExpirationTime(token: string): Date | null;
export declare function formatTotpCode(code: string): string;
export declare function formatRecoveryCode(code: string): string;
export declare function permissionToString(permission: Permission): string;
export declare function permissionFromString(str: string): Permission | null;
//# sourceMappingURL=auth.d.ts.map