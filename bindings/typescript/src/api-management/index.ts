/**
 * CADDY API Management
 *
 * Comprehensive API management portal with documentation, testing,
 * monitoring, and versioning capabilities.
 *
 * @module api-management
 */

// Core Components
export { APIPortal } from './APIPortal';
export { APIExplorer } from './APIExplorer';
export { APIDocumentation } from './APIDocumentation';
export { APIEndpoints } from './APIEndpoints';
export { APIKeys } from './APIKeys';
export { APIRateLimits } from './APIRateLimits';
export { APIAnalytics } from './APIAnalytics';
export { APIWebhooks } from './APIWebhooks';
export { APIMocking } from './APIMocking';
export { APIVersioning } from './APIVersioning';
export { APITesting } from './APITesting';

// Type Exports
export type {
  // Core API Types
  APIEndpoint,
  HTTPMethod,
  APIParameter,
  RequestBody,
  APIResponse,
  MediaType,
  Header,
  Link,
  Example,

  // OpenAPI/Swagger Types
  OpenAPISpec,
  Info,
  Contact,
  License,
  Server,
  ServerVariable,
  PathItem,
  Operation,
  Components,
  SecurityRequirement,
  SecurityScheme,
  OAuthFlows,
  OAuthFlow,
  Tag,
  ExternalDocumentation,

  // JSON Schema Types
  JSONSchema,
  Discriminator,
  XML,

  // API Key Management
  APIKey,
  APIKeyUsage,

  // Rate Limiting
  RateLimit,
  RateLimitType,
  RateLimitScope,
  RateLimitAction,
  RateLimitStatus,

  // API Analytics
  APIMetrics,
  EndpointMetrics,
  ErrorMetrics,
  TimeSeriesData,
  APIAnalytics as APIAnalyticsData,

  // Webhooks
  Webhook,
  RetryPolicy,
  WebhookFilter,
  WebhookDelivery,
  WebhookEvent,

  // API Testing
  APITestRequest,
  APITestResponse,
  AuthConfig,
  TestCase,
  Assertion,
  TestStep,
  TestSuite,
  TestResult,

  // Mock Server
  MockEndpoint,
  MockResponse,
  MockCondition,
  MockServer,

  // API Versioning
  APIVersion,
  VersionStatus,
  ChangelogEntry,
  VersionMigration,

  // Code Generation
  CodeGenerationConfig,
  CodeLanguage,
  GeneratedCode,

  // API Portal
  APIPortalConfig,
  APICollection,

  // Request/Response Types
  RequestLog,

  // Error Types
  APIError,
  RateLimitError,
  ValidationError,
  ValidationIssue,
} from './types';

// Default Export
export { APIPortal as default } from './APIPortal';

/**
 * API Management Portal
 *
 * The CADDY API Management Portal provides a comprehensive suite of tools
 * for managing, documenting, testing, and monitoring APIs.
 *
 * @example
 * ```tsx
 * import { APIPortal } from '@caddy/enterprise-sdk/api-management';
 *
 * function App() {
 *   return (
 *     <APIPortal
 *       projectId="my-project"
 *       config={{
 *         title: "My API Portal",
 *         enableAnalytics: true,
 *         enableTesting: true,
 *         enableMocking: true,
 *       }}
 *     />
 *   );
 * }
 * ```
 *
 * @example
 * ```tsx
 * import { APIExplorer, APIDocumentation } from '@caddy/enterprise-sdk/api-management';
 *
 * function APIPage() {
 *   return (
 *     <div>
 *       <APIDocumentation
 *         spec={openAPISpec}
 *         showTryItOut={true}
 *       />
 *       <APIExplorer
 *         endpoints={endpoints}
 *         enableCodeGen={true}
 *       />
 *     </div>
 *   );
 * }
 * ```
 *
 * @example
 * ```tsx
 * import { APIKeys, APIRateLimits } from '@caddy/enterprise-sdk/api-management';
 *
 * function SecurityPage() {
 *   return (
 *     <div>
 *       <APIKeys
 *         userId="user-123"
 *         onKeyCreate={async (name, scopes) => {
 *           const key = await api.createKey(name, scopes);
 *           return key;
 *         }}
 *       />
 *       <APIRateLimits
 *         onRateLimitCreate={async (limit) => {
 *           await api.createRateLimit(limit);
 *         }}
 *       />
 *     </div>
 *   );
 * }
 * ```
 *
 * @example
 * ```tsx
 * import { APIAnalytics, APITesting } from '@caddy/enterprise-sdk/api-management';
 *
 * function MonitoringPage() {
 *   return (
 *     <div>
 *       <APIAnalytics
 *         projectId="my-project"
 *         refreshInterval={30000}
 *       />
 *       <APITesting
 *         onRunTest={async (testId) => {
 *           return await api.runTest(testId);
 *         }}
 *       />
 *     </div>
 *   );
 * }
 * ```
 */
