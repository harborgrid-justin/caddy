export class UserManagementError extends Error {
    constructor(message, code, statusCode, details) {
        super(message);
        this.code = code;
        this.statusCode = statusCode;
        this.details = details;
        this.name = 'UserManagementError';
    }
}
export class PermissionDeniedError extends UserManagementError {
    constructor(message, details) {
        super(message, 'PERMISSION_DENIED', 403, details);
        this.name = 'PermissionDeniedError';
    }
}
export class UserNotFoundError extends UserManagementError {
    constructor(userId) {
        super(`User not found: ${userId}`, 'USER_NOT_FOUND', 404);
        this.name = 'UserNotFoundError';
    }
}
export class RoleNotFoundError extends UserManagementError {
    constructor(roleId) {
        super(`Role not found: ${roleId}`, 'ROLE_NOT_FOUND', 404);
        this.name = 'RoleNotFoundError';
    }
}
export class TeamNotFoundError extends UserManagementError {
    constructor(teamId) {
        super(`Team not found: ${teamId}`, 'TEAM_NOT_FOUND', 404);
        this.name = 'TeamNotFoundError';
    }
}
export class ValidationError extends UserManagementError {
    constructor(message, fields) {
        super(message, 'VALIDATION_ERROR', 400, fields);
        this.fields = fields;
        this.name = 'ValidationError';
    }
}
//# sourceMappingURL=types.js.map