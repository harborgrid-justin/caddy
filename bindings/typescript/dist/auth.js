export class AuthError extends Error {
    constructor(message, code, statusCode) {
        super(message);
        this.code = code;
        this.statusCode = statusCode;
        this.name = "AuthError";
    }
}
export class OAuth2Error extends AuthError {
    constructor(message, statusCode) {
        super(message, "OAUTH2_ERROR", statusCode);
        this.name = "OAuth2Error";
    }
}
export class SamlError extends AuthError {
    constructor(message, statusCode) {
        super(message, "SAML_ERROR", statusCode);
        this.name = "SamlError";
    }
}
export class JwtError extends AuthError {
    constructor(message, statusCode) {
        super(message, "JWT_ERROR", statusCode);
        this.name = "JwtError";
    }
}
export class MfaError extends AuthError {
    constructor(message, statusCode) {
        super(message, "MFA_ERROR", statusCode);
        this.name = "MfaError";
    }
}
export class RbacError extends AuthError {
    constructor(message, statusCode) {
        super(message, "RBAC_ERROR", statusCode);
        this.name = "RbacError";
    }
}
export function parseJwt(token) {
    try {
        const base64Url = token.split('.')[1];
        const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
        const jsonPayload = decodeURIComponent(atob(base64)
            .split('')
            .map(c => '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2))
            .join(''));
        return JSON.parse(jsonPayload);
    }
    catch (error) {
        return null;
    }
}
export function isTokenExpired(token) {
    const claims = parseJwt(token);
    if (!claims || !claims.exp)
        return true;
    return Date.now() >= claims.exp * 1000;
}
export function getTokenExpirationTime(token) {
    const claims = parseJwt(token);
    if (!claims || !claims.exp)
        return null;
    return new Date(claims.exp * 1000);
}
export function formatTotpCode(code) {
    return code.replace(/\D/g, '').slice(0, 6);
}
export function formatRecoveryCode(code) {
    const cleaned = code.replace(/[^A-Z0-9]/g, '').toUpperCase();
    if (cleaned.length >= 4) {
        return `${cleaned.slice(0, 4)}-${cleaned.slice(4, 8)}`;
    }
    return cleaned;
}
export function permissionToString(permission) {
    if (permission.scope) {
        return `${permission.resource}:${permission.action}:${permission.scope}`;
    }
    return `${permission.resource}:${permission.action}`;
}
export function permissionFromString(str) {
    const parts = str.split(':');
    if (parts.length === 2) {
        return {
            resource: parts[0],
            action: parts[1],
        };
    }
    else if (parts.length === 3) {
        return {
            resource: parts[0],
            action: parts[1],
            scope: parts[2],
        };
    }
    return null;
}
//# sourceMappingURL=auth.js.map