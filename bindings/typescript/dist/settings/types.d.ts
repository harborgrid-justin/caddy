export interface SettingsBase {
    id: string;
    version: number;
    updatedAt: Date;
    updatedBy: string;
}
export interface GeneralSettings extends SettingsBase {
    appName: string;
    description: string;
    timezone: string;
    locale: string;
    dateFormat: string;
    timeFormat: '12h' | '24h';
    currency: string;
    branding: BrandingSettings;
    maintenance: MaintenanceSettings;
}
export interface BrandingSettings {
    logo: string;
    favicon: string;
    primaryColor: string;
    secondaryColor: string;
    accentColor: string;
    customCSS?: string;
    emailTemplate?: string;
}
export interface MaintenanceSettings {
    enabled: boolean;
    message: string;
    allowedIPs: string[];
    startTime?: Date;
    endTime?: Date;
}
export interface SecuritySettings extends SettingsBase {
    passwordPolicy: PasswordPolicy;
    twoFactorAuth: TwoFactorAuthSettings;
    sso: SSOSettings;
    sessionManagement: SessionManagement;
    ipWhitelist: IPWhitelistSettings;
    auditLog: AuditLogSettings;
}
export interface PasswordPolicy {
    minLength: number;
    maxLength: number;
    requireUppercase: boolean;
    requireLowercase: boolean;
    requireNumbers: boolean;
    requireSpecialChars: boolean;
    preventReuse: number;
    expirationDays: number;
    maxAttempts: number;
    lockoutDuration: number;
}
export interface TwoFactorAuthSettings {
    enabled: boolean;
    required: boolean;
    methods: ('totp' | 'sms' | 'email' | 'hardware')[];
    gracePeriod: number;
    trustedDeviceDuration: number;
}
export interface SSOSettings {
    enabled: boolean;
    providers: SSOProvider[];
    allowLocalAuth: boolean;
    autoProvision: boolean;
    defaultRole: string;
}
export interface SSOProvider {
    id: string;
    type: 'saml' | 'oauth2' | 'oidc' | 'ldap';
    name: string;
    enabled: boolean;
    clientId: string;
    clientSecret: string;
    issuer?: string;
    authorizationURL?: string;
    tokenURL?: string;
    userInfoURL?: string;
    certificateURL?: string;
    attributeMapping: Record<string, string>;
}
export interface SessionManagement {
    timeout: number;
    absoluteTimeout: number;
    extendOnActivity: boolean;
    maxConcurrentSessions: number;
    enforceIPBinding: boolean;
    secureOnly: boolean;
}
export interface IPWhitelistSettings {
    enabled: boolean;
    allowedRanges: string[];
    bypassRoles: string[];
}
export interface AuditLogSettings {
    enabled: boolean;
    retentionDays: number;
    logAuthEvents: boolean;
    logDataChanges: boolean;
    logApiCalls: boolean;
    exportFormat: 'json' | 'csv' | 'syslog';
}
export interface NotificationSettings extends SettingsBase {
    email: EmailNotificationSettings;
    sms: SMSNotificationSettings;
    push: PushNotificationSettings;
    inApp: InAppNotificationSettings;
    channels: NotificationChannel[];
}
export interface EmailNotificationSettings {
    enabled: boolean;
    provider: 'smtp' | 'sendgrid' | 'ses' | 'mailgun';
    host?: string;
    port?: number;
    username?: string;
    password?: string;
    apiKey?: string;
    fromAddress: string;
    fromName: string;
    replyTo?: string;
    useTLS: boolean;
    templates: Record<string, EmailTemplate>;
}
export interface EmailTemplate {
    subject: string;
    body: string;
    htmlBody?: string;
    attachments?: string[];
}
export interface SMSNotificationSettings {
    enabled: boolean;
    provider: 'twilio' | 'sns' | 'nexmo';
    accountSid?: string;
    authToken?: string;
    apiKey?: string;
    fromNumber: string;
    templates: Record<string, string>;
}
export interface PushNotificationSettings {
    enabled: boolean;
    provider: 'fcm' | 'apns' | 'onesignal';
    serverKey?: string;
    certificateURL?: string;
    appId?: string;
    apiKey?: string;
}
export interface InAppNotificationSettings {
    enabled: boolean;
    soundEnabled: boolean;
    desktopEnabled: boolean;
    retentionDays: number;
    maxNotifications: number;
}
export interface NotificationChannel {
    id: string;
    name: string;
    type: 'email' | 'sms' | 'push' | 'inApp';
    events: string[];
    enabled: boolean;
    conditions?: NotificationCondition[];
}
export interface NotificationCondition {
    field: string;
    operator: 'equals' | 'contains' | 'greaterThan' | 'lessThan';
    value: string | number;
}
export interface IntegrationSettings extends SettingsBase {
    integrations: Integration[];
    webhooks: Webhook[];
    apiLimits: APILimits;
}
export interface Integration {
    id: string;
    name: string;
    type: string;
    enabled: boolean;
    config: Record<string, unknown>;
    credentials: IntegrationCredentials;
    syncInterval?: number;
    lastSync?: Date;
    status: 'connected' | 'disconnected' | 'error';
    errorMessage?: string;
}
export interface IntegrationCredentials {
    apiKey?: string;
    apiSecret?: string;
    accessToken?: string;
    refreshToken?: string;
    expiresAt?: Date;
}
export interface Webhook {
    id: string;
    name: string;
    url: string;
    events: string[];
    enabled: boolean;
    secret?: string;
    headers?: Record<string, string>;
    retryCount: number;
    timeout: number;
    lastTriggered?: Date;
}
export interface APILimits {
    rateLimitPerMinute: number;
    rateLimitPerHour: number;
    maxPayloadSize: number;
    allowedOrigins: string[];
    requireApiKey: boolean;
}
export interface BillingSettings extends SettingsBase {
    subscription: Subscription;
    paymentMethods: PaymentMethod[];
    billingAddress: BillingAddress;
    invoices: Invoice[];
    usage: UsageMetrics;
}
export interface Subscription {
    plan: 'free' | 'starter' | 'professional' | 'enterprise';
    status: 'active' | 'canceled' | 'past_due' | 'trialing';
    billingCycle: 'monthly' | 'annual';
    currentPeriodStart: Date;
    currentPeriodEnd: Date;
    cancelAtPeriodEnd: boolean;
    trialEnd?: Date;
    seats: number;
    addons: SubscriptionAddon[];
}
export interface SubscriptionAddon {
    id: string;
    name: string;
    quantity: number;
    unitPrice: number;
}
export interface PaymentMethod {
    id: string;
    type: 'card' | 'bank' | 'paypal';
    isDefault: boolean;
    last4?: string;
    brand?: string;
    expiryMonth?: number;
    expiryYear?: number;
    bankName?: string;
    email?: string;
}
export interface BillingAddress {
    company?: string;
    addressLine1: string;
    addressLine2?: string;
    city: string;
    state?: string;
    postalCode: string;
    country: string;
    taxId?: string;
}
export interface Invoice {
    id: string;
    number: string;
    date: Date;
    dueDate: Date;
    status: 'draft' | 'open' | 'paid' | 'void' | 'uncollectible';
    subtotal: number;
    tax: number;
    total: number;
    currency: string;
    pdfURL?: string;
    items: InvoiceItem[];
}
export interface InvoiceItem {
    description: string;
    quantity: number;
    unitPrice: number;
    amount: number;
}
export interface UsageMetrics {
    users: number;
    storage: number;
    bandwidth: number;
    apiCalls: number;
    period: {
        start: Date;
        end: Date;
    };
    limits: {
        users: number;
        storage: number;
        bandwidth: number;
        apiCalls: number;
    };
}
export interface TeamSettings extends SettingsBase {
    members: TeamMember[];
    roles: Role[];
    invitations: Invitation[];
    groups: TeamGroup[];
}
export interface TeamMember {
    id: string;
    email: string;
    name: string;
    avatar?: string;
    role: string;
    status: 'active' | 'invited' | 'suspended';
    joinedAt: Date;
    lastActive?: Date;
    permissions: string[];
    groups: string[];
}
export interface Role {
    id: string;
    name: string;
    description: string;
    permissions: Permission[];
    isSystem: boolean;
    memberCount: number;
}
export interface Permission {
    id: string;
    resource: string;
    action: 'create' | 'read' | 'update' | 'delete' | 'manage';
    conditions?: PermissionCondition[];
}
export interface PermissionCondition {
    field: string;
    operator: 'equals' | 'contains' | 'in';
    value: string | string[];
}
export interface Invitation {
    id: string;
    email: string;
    role: string;
    invitedBy: string;
    invitedAt: Date;
    expiresAt: Date;
    status: 'pending' | 'accepted' | 'expired' | 'revoked';
}
export interface TeamGroup {
    id: string;
    name: string;
    description: string;
    memberIds: string[];
    permissions: string[];
}
export interface AdvancedSettings extends SettingsBase {
    developerMode: boolean;
    apiKeys: APIKey[];
    webhooks: AdvancedWebhook[];
    customDomains: CustomDomain[];
    cors: CORSSettings;
    logging: LoggingSettings;
    performance: PerformanceSettings;
}
export interface APIKey {
    id: string;
    name: string;
    key: string;
    prefix: string;
    permissions: string[];
    rateLimit: number;
    expiresAt?: Date;
    lastUsed?: Date;
    createdAt: Date;
    status: 'active' | 'revoked';
}
export interface AdvancedWebhook {
    id: string;
    name: string;
    url: string;
    events: string[];
    enabled: boolean;
    secret: string;
    headers: Record<string, string>;
    retryPolicy: RetryPolicy;
    filters: WebhookFilter[];
    deliveryLogs: WebhookDelivery[];
}
export interface RetryPolicy {
    maxAttempts: number;
    backoffMultiplier: number;
    initialDelay: number;
    maxDelay: number;
}
export interface WebhookFilter {
    field: string;
    operator: 'equals' | 'contains' | 'matches';
    value: string;
}
export interface WebhookDelivery {
    id: string;
    timestamp: Date;
    event: string;
    status: 'success' | 'failed' | 'pending';
    statusCode?: number;
    responseTime?: number;
    attempts: number;
    errorMessage?: string;
}
export interface CustomDomain {
    id: string;
    domain: string;
    status: 'pending' | 'active' | 'failed';
    sslEnabled: boolean;
    sslCertificate?: string;
    dnsRecords: DNSRecord[];
    verifiedAt?: Date;
}
export interface DNSRecord {
    type: 'A' | 'CNAME' | 'TXT';
    name: string;
    value: string;
    verified: boolean;
}
export interface CORSSettings {
    enabled: boolean;
    allowedOrigins: string[];
    allowedMethods: string[];
    allowedHeaders: string[];
    exposedHeaders: string[];
    maxAge: number;
    allowCredentials: boolean;
}
export interface LoggingSettings {
    level: 'debug' | 'info' | 'warn' | 'error';
    destinations: LogDestination[];
    retention: number;
    maskSensitiveData: boolean;
    includeStackTrace: boolean;
}
export interface LogDestination {
    type: 'console' | 'file' | 'syslog' | 'cloudwatch' | 'datadog';
    enabled: boolean;
    config: Record<string, unknown>;
}
export interface PerformanceSettings {
    cacheEnabled: boolean;
    cacheTTL: number;
    compressionEnabled: boolean;
    compressionLevel: number;
    minifyAssets: boolean;
    cdnEnabled: boolean;
    cdnURL?: string;
}
export interface ValidationError {
    field: string;
    message: string;
    code?: string;
}
export interface FormState<T> {
    values: T;
    errors: ValidationError[];
    isDirty: boolean;
    isSubmitting: boolean;
    isValid: boolean;
}
export interface SettingsHistory {
    id: string;
    section: string;
    action: 'create' | 'update' | 'delete';
    changes: SettingsChange[];
    userId: string;
    userName: string;
    timestamp: Date;
    metadata?: Record<string, unknown>;
}
export interface SettingsChange {
    field: string;
    oldValue: unknown;
    newValue: unknown;
}
export interface ConfirmationDialog {
    open: boolean;
    title: string;
    message: string;
    confirmText?: string;
    cancelText?: string;
    severity: 'info' | 'warning' | 'error';
    onConfirm: () => void;
    onCancel: () => void;
}
export interface AutoSaveState {
    saving: boolean;
    lastSaved?: Date;
    error?: string;
}
export interface UndoRedoState<T> {
    past: T[];
    present: T;
    future: T[];
}
export interface SearchResult {
    section: string;
    title: string;
    description: string;
    path: string;
    matches: SearchMatch[];
}
export interface SearchMatch {
    field: string;
    value: string;
    highlight: string;
}
export interface SettingsTab {
    id: string;
    label: string;
    icon: string;
    component: React.ComponentType<any>;
    badge?: number;
    disabled?: boolean;
}
export interface ToastNotification {
    id: string;
    type: 'success' | 'error' | 'warning' | 'info';
    message: string;
    duration?: number;
    action?: {
        label: string;
        onClick: () => void;
    };
}
//# sourceMappingURL=types.d.ts.map