/**
 * CI/CD Integration Types
 *
 * Type definitions for CADDY's CI/CD integration system.
 */

/**
 * Supported CI/CD platforms
 */
export enum CIPlatform {
  GitHub = 'github',
  GitLab = 'gitlab',
  Jenkins = 'jenkins',
  AzureDevOps = 'azure-devops',
  Bitbucket = 'bitbucket',
  CircleCI = 'circleci',
  TravisCI = 'travis-ci',
  Custom = 'custom',
}

/**
 * Integration status
 */
export enum IntegrationStatus {
  NotConfigured = 'not-configured',
  Configured = 'configured',
  Active = 'active',
  Error = 'error',
  Disabled = 'disabled',
}

/**
 * Integration configuration
 */
export interface IntegrationConfig {
  id: string;
  platform: CIPlatform;
  name: string;
  description?: string;
  status: IntegrationStatus;
  enabled: boolean;
  createdAt: Date;
  updatedAt: Date;
  settings: Record<string, unknown>;
}

/**
 * GitHub-specific configuration
 */
export interface GitHubConfig {
  appId: string;
  installationId?: string;
  privateKey?: string;
  webhookSecret?: string;
  checkName?: string;
  autoInstall?: boolean;
}

/**
 * GitLab-specific configuration
 */
export interface GitLabConfig {
  url: string;
  token: string;
  projectId: string;
  webhookSecret?: string;
  pipelineName?: string;
}

/**
 * Jenkins-specific configuration
 */
export interface JenkinsConfig {
  url: string;
  user: string;
  token: string;
  jobName?: string;
  outputDir?: string;
}

/**
 * Azure DevOps-specific configuration
 */
export interface AzureDevOpsConfig {
  organization: string;
  project: string;
  token: string;
  repositoryId?: string;
}

/**
 * Bitbucket-specific configuration
 */
export interface BitbucketConfig {
  platform: 'cloud' | 'server';
  url?: string;
  workspace: string;
  repository: string;
  username: string;
  token: string;
  webhookSecret?: string;
}

/**
 * Integration marketplace item
 */
export interface IntegrationMarketplaceItem {
  id: string;
  platform: CIPlatform;
  name: string;
  displayName: string;
  description: string;
  icon: string;
  category: IntegrationCategory;
  version: string;
  author: string;
  documentation: string;
  features: string[];
  requirements: string[];
  screenshots?: string[];
  popularity: number;
  rating: number;
  downloads: number;
  isOfficial: boolean;
  isPremium: boolean;
  tags: string[];
}

/**
 * Integration category
 */
export enum IntegrationCategory {
  VersionControl = 'version-control',
  CI = 'ci',
  CD = 'cd',
  Monitoring = 'monitoring',
  Notification = 'notification',
  Testing = 'testing',
  Security = 'security',
  Other = 'other',
}

/**
 * Check result status
 */
export enum CheckStatus {
  Queued = 'queued',
  InProgress = 'in-progress',
  Success = 'success',
  Warning = 'warning',
  Failure = 'failure',
  Cancelled = 'cancelled',
  Skipped = 'skipped',
}

/**
 * Annotation level
 */
export enum AnnotationLevel {
  Notice = 'notice',
  Warning = 'warning',
  Error = 'error',
}

/**
 * Code annotation
 */
export interface Annotation {
  path: string;
  startLine: number;
  endLine: number;
  startColumn?: number;
  endColumn?: number;
  level: AnnotationLevel;
  message: string;
  title?: string;
  rawDetails?: string;
}

/**
 * Check result
 */
export interface CheckResult {
  status: CheckStatus;
  summary: string;
  details?: string;
  annotations: Annotation[];
  executionTimeMs: number;
  metadata: Record<string, string>;
}

/**
 * CI configuration template
 */
export interface CIConfigTemplate {
  platform: CIPlatform;
  name: string;
  description: string;
  filename: string;
  content: string;
  variables: CIConfigVariable[];
}

/**
 * CI configuration variable
 */
export interface CIConfigVariable {
  key: string;
  description: string;
  defaultValue?: string;
  required: boolean;
  type: 'string' | 'number' | 'boolean' | 'secret';
}

/**
 * Webhook event
 */
export interface WebhookEvent {
  id: string;
  platform: CIPlatform;
  eventType: string;
  timestamp: Date;
  payload: Record<string, unknown>;
  signature?: string;
  verified: boolean;
  processed: boolean;
}

/**
 * Integration statistics
 */
export interface IntegrationStats {
  totalChecks: number;
  successfulChecks: number;
  failedChecks: number;
  averageExecutionTime: number;
  checksPerDay: number[];
  lastCheckAt?: Date;
  errorRate: number;
}

/**
 * Setup wizard step
 */
export interface SetupStep {
  id: string;
  title: string;
  description: string;
  component: React.ComponentType<SetupStepProps>;
  optional?: boolean;
  completed: boolean;
}

/**
 * Setup step component props
 */
export interface SetupStepProps {
  config: Partial<IntegrationConfig>;
  onUpdate: (config: Partial<IntegrationConfig>) => void;
  onNext: () => void;
  onBack: () => void;
}

/**
 * Integration connection test result
 */
export interface ConnectionTestResult {
  success: boolean;
  message: string;
  details?: string;
  latency?: number;
  timestamp: Date;
}

/**
 * CI/CD pipeline run
 */
export interface PipelineRun {
  id: string;
  platform: CIPlatform;
  integrationId: string;
  status: CheckStatus;
  branch: string;
  commit: string;
  author: string;
  message: string;
  startedAt: Date;
  completedAt?: Date;
  duration?: number;
  result?: CheckResult;
}

/**
 * Integration activity log entry
 */
export interface ActivityLogEntry {
  id: string;
  integrationId: string;
  timestamp: Date;
  type: 'info' | 'warning' | 'error' | 'success';
  message: string;
  details?: string;
  metadata?: Record<string, unknown>;
}

/**
 * Export/import configuration format
 */
export interface ExportedConfig {
  version: string;
  timestamp: Date;
  integrations: IntegrationConfig[];
}
