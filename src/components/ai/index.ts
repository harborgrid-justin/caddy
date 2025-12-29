/**
 * AI Components - Module Exports
 *
 * Centralized exports for all AI-powered accessibility components
 */

// Components
export { AIAssistant } from './AIAssistant';
export { SuggestionPanel, formatFixType } from './SuggestionPanel';
export { InsightsView } from './InsightsView';

// Types
export type {
  // Engine Types
  EngineConfig,
  ModelVersion,
  ModelMetrics,
  EngineHealth,

  // Vision Types
  ImageAccessibility,
  AltTextGeneration,
  DetectedObject,
  BoundingBox,
  ColorContrastAnalysis,
  ContrastRegion,
  Color,
  VisualHierarchy,
  HierarchyElement,
  IconRecognition,
  ChartAnalysis,
  ChartType,
  Axis,
  DataSeries,

  // NLP Types
  TextAnalysis,
  ReadabilityScore,
  ReadabilityMetrics,
  PlainLanguageSuggestion,
  SuggestionCategory,
  HeadingAnalysis,
  LinkTextAnalysis,
  FormLabelQuality,

  // Predictions Types
  TrendPrediction,
  IssueTrend,
  RemediationEstimate,
  ComplianceRisk,
  RiskLevel,
  RegressionProbability,
  ImpactScore,

  // Suggestions Types
  AutoFix,
  CodeCompletion,
  AltTextSuggestion,
  ARIASuggestion,
  BestPracticeSuggestion,
  SuggestionConfidence,

  // AI Assistant Types
  ChatMessage,
  AIContext,
  AssistantConfig,

  // Insights Types
  AIInsights,
  InsightSummary,
  CategoryCount,
  Recommendation,
  Prediction,

  // API Types
  AIAnalysisRequest,
  AIAnalysisResponse,

  // Component Props
  AIAssistantProps,
  SuggestionPanelProps,
  InsightsViewProps,

  // Utility Types
  LoadingState,
  ErrorState,
  AsyncState,
} from './types';
