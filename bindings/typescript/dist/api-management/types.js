export class APIError extends Error {
    constructor(message, code, statusCode, details) {
        super(message);
        this.code = code;
        this.statusCode = statusCode;
        this.details = details;
        this.name = 'APIError';
    }
}
export class RateLimitError extends APIError {
    constructor(message, retryAfter) {
        super(message, 'RATE_LIMIT_EXCEEDED', 429, { retryAfter });
        this.retryAfter = retryAfter;
        this.name = 'RateLimitError';
    }
}
export class ValidationError extends APIError {
    constructor(message, errors) {
        super(message, 'VALIDATION_ERROR', 400, { errors });
        this.errors = errors;
        this.name = 'ValidationError';
    }
}
//# sourceMappingURL=types.js.map