# CADDY Enterprise TypeScript SDK

Official TypeScript/JavaScript SDK for CADDY Enterprise Edition v0.2.0.

## Installation

```bash
npm install @caddy/enterprise-sdk
```

## Quick Start

```typescript
import {
  EnterpriseSDK,
  CacheClient,
  TracingClient,
  TenantManager,
  RateLimitClient,
  RealtimeClient
} from '@caddy/enterprise-sdk';

// Initialize the SDK
const sdk = new EnterpriseSDK({
  apiUrl: 'https://api.caddy-cad.com',
  apiToken: 'your-api-token',
  licenseKey: 'your-license-key',
});

// Validate license
const isValid = await sdk.validateLicense();
console.log('License valid:', isValid);
```

## Features

### Distributed Caching

```typescript
const cache = new CacheClient({
  apiUrl: 'https://api.caddy-cad.com',
  token: 'your-token',
  defaultTtl: 3600,
});

// Set a value
await cache.set('user:123', { name: 'John Doe' }, {
  ttl: 7200,
  tags: ['users', 'premium'],
});

// Get a value
const user = await cache.get('user:123');

// Invalidate by tag
await cache.invalidateByTag('users');
```

### Distributed Tracing

```typescript
const tracing = new TracingClient({
  apiUrl: 'https://api.caddy-cad.com',
  serviceName: 'my-cad-service',
  samplingRate: 1.0,
});

// Trace a function
const result = await tracing.traced('processDrawing', async (span) => {
  await span.addEvent('Loading CAD file');
  // Your processing logic
  return processedData;
});
```

### Multi-Tenancy

```typescript
const tenantManager = new TenantManager({
  apiUrl: 'https://api.caddy-cad.com',
  token: 'your-token',
});

// Create a tenant
const tenant = await tenantManager.createTenant('Acme Corp', {
  maxStorage: 1024 * 1024 * 1024, // 1GB
  maxConcurrentUsers: 50,
});

// Execute within tenant context
await tenantManager.withTenant(tenant.id, async () => {
  // All operations here are scoped to this tenant
  const usage = await tenantManager.getTenantUsage(tenant.id);
  console.log('Storage used:', usage.storageUsed);
});
```

### Rate Limiting

```typescript
const rateLimit = new RateLimitClient({
  apiUrl: 'https://api.caddy-cad.com',
  token: 'your-token',
});

// Check rate limit
const result = await rateLimit.check('user:123', {
  type: 'user',
  cost: 1,
});

if (result.allowed) {
  // Process request
} else {
  console.log(`Rate limited. Retry after ${result.retryAfter}s`);
}

// Execute with automatic retry
await rateLimit.rateLimited('user:123', async () => {
  // Your rate-limited operation
}, { retryOnLimit: true });
```

### Real-Time Collaboration

```typescript
const realtime = new RealtimeClient({
  wsUrl: 'wss://api.caddy-cad.com/ws',
  token: 'your-token',
  autoReconnect: true,
});

// Connect
await realtime.connect();

// Join a session
const session = await realtime.joinSession('doc-123', 'John Doe');

// Listen for updates
realtime.on('document:update', (update) => {
  console.log('Document updated:', update);
  // Apply update to local state
});

// Send updates
realtime.sendUpdate({
  documentId: 'doc-123',
  userId: 'user-123',
  type: 'insert',
  data: { /* update data */ },
  vectorClock: { 'user-123': 1 },
});

// Update presence (cursor position)
realtime.updatePresence({
  cursor: { x: 100, y: 200 },
});
```

## API Reference

### EnterpriseSDK

Main SDK class for enterprise features.

- `constructor(config: EnterpriseConfig)` - Initialize the SDK
- `validateLicense(): Promise<boolean>` - Validate enterprise license
- `getFeatureStatus(): Promise<Record<string, boolean>>` - Get enabled features

### CacheClient

Distributed caching with multi-tier support.

- `get<T>(key: string): Promise<T | null>` - Get cached value
- `set<T>(key: string, value: T, options?): Promise<void>` - Set cached value
- `delete(key: string): Promise<boolean>` - Delete cached value
- `invalidateByTag(tag: string): Promise<number>` - Invalidate by tag
- `lock(key: string, ttl?: number): Promise<string>` - Acquire distributed lock

### TracingClient

Distributed tracing and observability.

- `startSpan(name: string, options?): Promise<Span>` - Start a new span
- `endSpan(spanId: string, options?): Promise<void>` - End a span
- `traced<T>(name: string, fn: Function): Promise<T>` - Execute traced function

### TenantManager

Multi-tenant isolation and management.

- `createTenant(name: string, quotas?, settings?): Promise<Tenant>` - Create tenant
- `getTenant(id: string): Promise<Tenant>` - Get tenant details
- `getTenantUsage(id: string): Promise<TenantUsage>` - Get usage statistics
- `withTenant<T>(id: string, fn: Function): Promise<T>` - Execute in tenant context

### RateLimitClient

Rate limiting and throttling.

- `check(identifier: string, options?): Promise<RateLimitResult>` - Check rate limit
- `createQuota(...): Promise<QuotaInfo>` - Create rate limit quota
- `rateLimited<T>(identifier: string, fn: Function): Promise<T>` - Execute rate-limited

### RealtimeClient

Real-time collaboration with WebSockets.

- `connect(): Promise<void>` - Connect to real-time server
- `joinSession(documentId: string, userName: string): Promise<Session>` - Join session
- `sendUpdate(update): void` - Send document update
- `updatePresence(presence): void` - Update cursor/selection
- Event emitters: `connected`, `document:update`, `participant:joined`, etc.

## Configuration

All clients accept configuration objects with the following common properties:

```typescript
interface ClientConfig {
  apiUrl: string;      // Base API URL
  token?: string;      // Authentication token
  // Client-specific options...
}
```

## TypeScript Support

This SDK is written in TypeScript and provides full type definitions for all APIs.

```typescript
import { CacheEntry, Span, Tenant, RateLimitResult } from '@caddy/enterprise-sdk';

// Type-safe operations
const entry: CacheEntry<User> = await cache.get('user:123');
```

## Error Handling

All async methods can throw errors. Use try-catch for error handling:

```typescript
try {
  await cache.set('key', 'value');
} catch (error) {
  console.error('Cache operation failed:', error);
}
```

## License

Commercial license required. See LICENSE-ENTERPRISE.txt for details.

## Support

For enterprise support: enterprise@caddy-cad.com

## Version Compatibility

- SDK Version: 0.2.0
- Compatible with CADDY Enterprise: 0.2.0+
- Node.js: 16.0.0+
