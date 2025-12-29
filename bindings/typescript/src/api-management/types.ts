/**
 * CADDY API Management Types
 *
 * TypeScript type definitions for API management, documentation,
 * testing, monitoring, and versioning.
 */

// ============================================================================
// Core API Types
// ============================================================================

export interface APIEndpoint {
  id: string;
  path: string;
  method: HTTPMethod;
  version: string;
  summary: string;
  description: string;
  tags: string[];
  deprecated: boolean;
  security: SecurityRequirement[];
  parameters: APIParameter[];
  requestBody?: RequestBody;
  responses: Record<string, APIResponse>;
  servers?: Server[];
  operationId: string;
  metadata: Record<string, unknown>;
  createdAt: number;
  updatedAt: number;
}

export type HTTPMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE' | 'HEAD' | 'OPTIONS';

export interface APIParameter {
  name: string;
  in: 'query' | 'header' | 'path' | 'cookie';
  description: string;
  required: boolean;
  deprecated: boolean;
  schema: JSONSchema;
  example?: unknown;
  examples?: Record<string, Example>;
}

export interface RequestBody {
  description: string;
  required: boolean;
  content: Record<string, MediaType>;
}

export interface MediaType {
  schema: JSONSchema;
  example?: unknown;
  examples?: Record<string, Example>;
  encoding?: Record<string, Encoding>;
}

export interface Encoding {
  contentType?: string;
  headers?: Record<string, Header>;
  style?: string;
  explode?: boolean;
}

export interface Header {
  description?: string;
  required?: boolean;
  deprecated?: boolean;
  schema: JSONSchema;
}

export interface APIResponse {
  description: string;
  headers?: Record<string, Header>;
  content?: Record<string, MediaType>;
  links?: Record<string, Link>;
}

export interface Link {
  operationId?: string;
  operationRef?: string;
  parameters?: Record<string, unknown>;
  requestBody?: unknown;
  description?: string;
}

export interface Example {
  summary?: string;
  description?: string;
  value?: unknown;
  externalValue?: string;
}

// ============================================================================
// OpenAPI/Swagger Types
// ============================================================================

export interface OpenAPISpec {
  openapi: string;
  info: Info;
  servers: Server[];
  paths: Record<string, PathItem>;
  components?: Components;
  security?: SecurityRequirement[];
  tags?: Tag[];
  externalDocs?: ExternalDocumentation;
}

export interface Info {
  title: string;
  version: string;
  description?: string;
  termsOfService?: string;
  contact?: Contact;
  license?: License;
}

export interface Contact {
  name?: string;
  url?: string;
  email?: string;
}

export interface License {
  name: string;
  url?: string;
}

export interface Server {
  url: string;
  description?: string;
  variables?: Record<string, ServerVariable>;
}

export interface ServerVariable {
  enum?: string[];
  default: string;
  description?: string;
}

export interface PathItem {
  summary?: string;
  description?: string;
  get?: Operation;
  put?: Operation;
  post?: Operation;
  delete?: Operation;
  options?: Operation;
  head?: Operation;
  patch?: Operation;
  trace?: Operation;
  servers?: Server[];
  parameters?: APIParameter[];
}

export interface Operation {
  tags?: string[];
  summary?: string;
  description?: string;
  operationId?: string;
  parameters?: APIParameter[];
  requestBody?: RequestBody;
  responses: Record<string, APIResponse>;
  deprecated?: boolean;
  security?: SecurityRequirement[];
  servers?: Server[];
}

export interface Components {
  schemas?: Record<string, JSONSchema>;
  responses?: Record<string, APIResponse>;
  parameters?: Record<string, APIParameter>;
  examples?: Record<string, Example>;
  requestBodies?: Record<string, RequestBody>;
  headers?: Record<string, Header>;
  securitySchemes?: Record<string, SecurityScheme>;
  links?: Record<string, Link>;
  callbacks?: Record<string, Callback>;
}

export interface Callback {
  [expression: string]: PathItem;
}

export interface SecurityRequirement {
  [name: string]: string[];
}

export interface SecurityScheme {
  type: 'apiKey' | 'http' | 'oauth2' | 'openIdConnect';
  description?: string;
  name?: string;
  in?: 'query' | 'header' | 'cookie';
  scheme?: string;
  bearerFormat?: string;
  flows?: OAuthFlows;
  openIdConnectUrl?: string;
}

export interface OAuthFlows {
  implicit?: OAuthFlow;
  password?: OAuthFlow;
  clientCredentials?: OAuthFlow;
  authorizationCode?: OAuthFlow;
}

export interface OAuthFlow {
  authorizationUrl?: string;
  tokenUrl?: string;
  refreshUrl?: string;
  scopes: Record<string, string>;
}

export interface Tag {
  name: string;
  description?: string;
  externalDocs?: ExternalDocumentation;
}

export interface ExternalDocumentation {
  description?: string;
  url: string;
}

// ============================================================================
// JSON Schema Types
// ============================================================================

export interface JSONSchema {
  type?: 'string' | 'number' | 'integer' | 'boolean' | 'array' | 'object' | 'null';
  format?: string;
  title?: string;
  description?: string;
  default?: unknown;
  enum?: unknown[];
  const?: unknown;
  multipleOf?: number;
  maximum?: number;
  exclusiveMaximum?: number;
  minimum?: number;
  exclusiveMinimum?: number;
  maxLength?: number;
  minLength?: number;
  pattern?: string;
  maxItems?: number;
  minItems?: number;
  uniqueItems?: boolean;
  maxProperties?: number;
  minProperties?: number;
  required?: string[];
  properties?: Record<string, JSONSchema>;
  additionalProperties?: boolean | JSONSchema;
  items?: JSONSchema | JSONSchema[];
  allOf?: JSONSchema[];
  oneOf?: JSONSchema[];
  anyOf?: JSONSchema[];
  not?: JSONSchema;
  nullable?: boolean;
  discriminator?: Discriminator;
  readOnly?: boolean;
  writeOnly?: boolean;
  xml?: XML;
  externalDocs?: ExternalDocumentation;
  example?: unknown;
  deprecated?: boolean;
}

export interface Discriminator {
  propertyName: string;
  mapping?: Record<string, string>;
}

export interface XML {
  name?: string;
  namespace?: string;
  prefix?: string;
  attribute?: boolean;
  wrapped?: boolean;
}

// ============================================================================
// API Key Management
// ============================================================================

export interface APIKey {
  id: string;
  name: string;
  key: string;
  userId: string;
  scopes: string[];
  rateLimits: RateLimit[];
  environment: 'development' | 'staging' | 'production';
  expiresAt?: number;
  createdAt: number;
  lastUsedAt?: number;
  active: boolean;
  metadata: Record<string, unknown>;
}

export interface APIKeyUsage {
  apiKeyId: string;
  totalRequests: number;
  successfulRequests: number;
  failedRequests: number;
  averageResponseTime: number;
  lastUsed: number;
  topEndpoints: Array<{ endpoint: string; count: number }>;
}

// ============================================================================
// Rate Limiting
// ============================================================================

export interface RateLimit {
  id: string;
  name: string;
  type: RateLimitType;
  limit: number;
  window: number;
  scope: RateLimitScope;
  endpoints?: string[];
  methods?: HTTPMethod[];
  priority: number;
  active: boolean;
  actions: RateLimitAction[];
  metadata: Record<string, unknown>;
}

export type RateLimitType =
  | 'fixed_window'
  | 'sliding_window'
  | 'token_bucket'
  | 'leaky_bucket'
  | 'concurrent';

export type RateLimitScope = 'global' | 'user' | 'api_key' | 'ip' | 'endpoint';

export interface RateLimitAction {
  type: 'throttle' | 'block' | 'notify' | 'upgrade_required';
  config: Record<string, unknown>;
}

export interface RateLimitStatus {
  scope: string;
  limit: number;
  remaining: number;
  resetAt: number;
  retryAfter?: number;
}

// ============================================================================
// API Analytics
// ============================================================================

export interface APIMetrics {
  totalRequests: number;
  successRate: number;
  averageResponseTime: number;
  p50ResponseTime: number;
  p95ResponseTime: number;
  p99ResponseTime: number;
  errorRate: number;
  requestsPerSecond: number;
  bandwidth: number;
  timestamp: number;
}

export interface EndpointMetrics {
  endpointId: string;
  path: string;
  method: HTTPMethod;
  metrics: APIMetrics;
  statusCodes: Record<number, number>;
  errors: ErrorMetrics[];
  topUsers: Array<{ userId: string; count: number }>;
}

export interface ErrorMetrics {
  statusCode: number;
  message: string;
  count: number;
  lastOccurrence: number;
  stack?: string;
}

export interface TimeSeriesData {
  timestamp: number;
  value: number;
  metadata?: Record<string, unknown>;
}

export interface APIAnalytics {
  period: { start: number; end: number };
  overall: APIMetrics;
  byEndpoint: EndpointMetrics[];
  byUser: Record<string, APIMetrics>;
  byRegion: Record<string, APIMetrics>;
  timeSeries: Record<string, TimeSeriesData[]>;
  topErrors: ErrorMetrics[];
}

// ============================================================================
// Webhooks
// ============================================================================

export interface Webhook {
  id: string;
  name: string;
  url: string;
  events: string[];
  secret: string;
  active: boolean;
  retryPolicy: RetryPolicy;
  headers: Record<string, string>;
  filters?: WebhookFilter[];
  createdAt: number;
  updatedAt: number;
}

export interface RetryPolicy {
  maxRetries: number;
  backoffMultiplier: number;
  maxBackoff: number;
  retryOn: number[];
}

export interface WebhookFilter {
  field: string;
  operator: 'equals' | 'contains' | 'starts_with' | 'ends_with' | 'regex';
  value: string;
}

export interface WebhookDelivery {
  id: string;
  webhookId: string;
  event: string;
  payload: unknown;
  status: 'pending' | 'success' | 'failed' | 'retrying';
  attempts: number;
  lastAttemptAt?: number;
  nextRetryAt?: number;
  response?: {
    statusCode: number;
    headers: Record<string, string>;
    body: string;
    duration: number;
  };
  createdAt: number;
}

export interface WebhookEvent {
  type: string;
  timestamp: number;
  data: unknown;
  metadata: Record<string, unknown>;
}

// ============================================================================
// API Testing
// ============================================================================

export interface APITestRequest {
  endpoint: APIEndpoint;
  parameters: Record<string, unknown>;
  headers: Record<string, string>;
  body?: unknown;
  auth?: AuthConfig;
  timeout?: number;
  followRedirects?: boolean;
}

export interface APITestResponse {
  status: number;
  statusText: string;
  headers: Record<string, string>;
  body: unknown;
  duration: number;
  size: number;
  timestamp: number;
}

export interface AuthConfig {
  type: 'none' | 'api_key' | 'bearer' | 'basic' | 'oauth2';
  credentials: Record<string, string>;
}

export interface TestCase {
  id: string;
  name: string;
  description: string;
  request: APITestRequest;
  assertions: Assertion[];
  setup?: TestStep[];
  teardown?: TestStep[];
  createdAt: number;
  updatedAt: number;
}

export interface Assertion {
  type: 'status' | 'header' | 'body' | 'response_time' | 'json_schema';
  operator: 'equals' | 'contains' | 'matches' | 'greater_than' | 'less_than';
  expected: unknown;
  actual?: unknown;
  passed?: boolean;
  message?: string;
}

export interface TestStep {
  type: 'request' | 'script' | 'wait';
  config: Record<string, unknown>;
}

export interface TestSuite {
  id: string;
  name: string;
  description: string;
  tests: TestCase[];
  environment: string;
  variables: Record<string, unknown>;
  createdAt: number;
  updatedAt: number;
}

export interface TestResult {
  testId: string;
  passed: boolean;
  assertions: Assertion[];
  response?: APITestResponse;
  error?: string;
  duration: number;
  timestamp: number;
}

// ============================================================================
// Mock Server
// ============================================================================

export interface MockEndpoint {
  id: string;
  path: string;
  method: HTTPMethod;
  responses: MockResponse[];
  delay?: { min: number; max: number };
  active: boolean;
  createdAt: number;
}

export interface MockResponse {
  id: string;
  name: string;
  statusCode: number;
  headers: Record<string, string>;
  body: unknown;
  weight: number;
  conditions?: MockCondition[];
}

export interface MockCondition {
  type: 'header' | 'query' | 'body' | 'random';
  operator: 'equals' | 'contains' | 'matches';
  field?: string;
  value?: string;
  probability?: number;
}

export interface MockServer {
  id: string;
  name: string;
  baseUrl: string;
  endpoints: MockEndpoint[];
  globalDelay?: number;
  active: boolean;
  createdAt: number;
}

// ============================================================================
// API Versioning
// ============================================================================

export interface APIVersion {
  version: string;
  status: VersionStatus;
  releaseDate: number;
  deprecationDate?: number;
  sunsetDate?: number;
  changelog: ChangelogEntry[];
  endpoints: number;
  breaking: boolean;
  metadata: Record<string, unknown>;
}

export type VersionStatus = 'draft' | 'beta' | 'stable' | 'deprecated' | 'retired';

export interface ChangelogEntry {
  type: 'added' | 'changed' | 'deprecated' | 'removed' | 'fixed' | 'security';
  description: string;
  breaking: boolean;
  endpoint?: string;
  timestamp: number;
}

export interface VersionMigration {
  from: string;
  to: string;
  guide: string;
  automated: boolean;
  script?: string;
  estimatedEffort: 'low' | 'medium' | 'high';
}

// ============================================================================
// Code Generation
// ============================================================================

export interface CodeGenerationConfig {
  language: CodeLanguage;
  framework?: string;
  packageName?: string;
  outputPath?: string;
  options: Record<string, unknown>;
}

export type CodeLanguage =
  | 'typescript'
  | 'javascript'
  | 'python'
  | 'java'
  | 'go'
  | 'rust'
  | 'csharp'
  | 'php'
  | 'ruby'
  | 'curl';

export interface GeneratedCode {
  files: Array<{
    path: string;
    content: string;
    language: string;
  }>;
  dependencies: Record<string, string>;
  instructions: string;
  timestamp: number;
}

// ============================================================================
// API Portal
// ============================================================================

export interface APIPortalConfig {
  title: string;
  description: string;
  logo?: string;
  primaryColor: string;
  enableTryOut: boolean;
  enableCodeGen: boolean;
  enableMocking: boolean;
  supportedLanguages: CodeLanguage[];
  customCSS?: string;
}

export interface APICollection {
  id: string;
  name: string;
  description: string;
  endpoints: string[];
  icon?: string;
  color?: string;
  order: number;
}

// ============================================================================
// Request/Response Types
// ============================================================================

export interface RequestLog {
  id: string;
  timestamp: number;
  method: HTTPMethod;
  path: string;
  statusCode: number;
  duration: number;
  requestHeaders: Record<string, string>;
  requestBody?: unknown;
  responseHeaders: Record<string, string>;
  responseBody?: unknown;
  userId?: string;
  apiKeyId?: string;
  ipAddress: string;
  userAgent: string;
  error?: string;
}

// ============================================================================
// Error Types
// ============================================================================

export class APIError extends Error {
  constructor(
    message: string,
    public code: string,
    public statusCode?: number,
    public details?: Record<string, unknown>
  ) {
    super(message);
    this.name = 'APIError';
  }
}

export class RateLimitError extends APIError {
  constructor(message: string, public retryAfter: number) {
    super(message, 'RATE_LIMIT_EXCEEDED', 429, { retryAfter });
    this.name = 'RateLimitError';
  }
}

export class ValidationError extends APIError {
  constructor(message: string, public errors: ValidationIssue[]) {
    super(message, 'VALIDATION_ERROR', 400, { errors });
    this.name = 'ValidationError';
  }
}

export interface ValidationIssue {
  field: string;
  message: string;
  code: string;
  value?: unknown;
}
