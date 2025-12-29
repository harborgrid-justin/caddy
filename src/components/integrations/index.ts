/**
 * CI/CD Integrations Module
 *
 * Exports all integration-related components and types.
 */

// Main components
export { IntegrationHub } from './IntegrationHub';
export { GitHubSetup } from './GitHubSetup';
export { CIConfigGenerator } from './CIConfigGenerator';

// Types
export type {
  IntegrationConfig,
  IntegrationMarketplaceItem,
  IntegrationStats,
  GitHubConfig,
  GitLabConfig,
  JenkinsConfig,
  AzureDevOpsConfig,
  BitbucketConfig,
  CheckResult,
  Annotation,
  CIConfigTemplate,
  CIConfigVariable,
  WebhookEvent,
  PipelineRun,
  ActivityLogEntry,
  SetupStep,
  SetupStepProps,
  ConnectionTestResult,
  ExportedConfig,
} from './types';

// Enums
export {
  CIPlatform,
  IntegrationStatus,
  IntegrationCategory,
  CheckStatus,
  AnnotationLevel,
} from './types';

// Re-export as default for convenience
import { IntegrationHub } from './IntegrationHub';
export default IntegrationHub;
