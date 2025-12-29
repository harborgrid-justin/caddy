/**
 * TypeScript type definitions for AI-powered accessibility analysis
 */

// ============================================================================
// AI Engine Types
// ============================================================================

export interface EngineConfig {
  maxConcurrentTasks: number;
  enableGpu: boolean;
  gpuDeviceId?: number;
  batchSize: number;
  inferenceTimeoutSecs: number;
  enableVersioning: boolean;
  enableAbTesting: boolean;
  modelCacheSizeMb: number;
  enableTelemetry: boolean;
}

export interface ModelVersion {
  version: string;
  modelName: string;
  modelPath: string;
  modelType: string;
  createdAt: string;
  isActive: boolean;
  metrics: ModelMetrics;
}

export interface ModelMetrics {
  avgInferenceTimeMs: number;
  accuracy: number;
  totalInferences: number;
  errorRate: number;
}

export interface EngineHealth {
  status: string;
  modelsLoaded: number;
  gpuAvailable: boolean;
  gpuMemoryUsedMb: number;
  gpuMemoryTotalMb: number;
  activeAbTests: number;
  timestamp: string;
}

// ============================================================================
// Vision Analysis Types
// ============================================================================

export interface ImageAccessibility {
  altText?: AltTextGeneration;
  contrast?: ColorContrastAnalysis;
  hierarchy?: VisualHierarchy;
  icons: IconRecognition[];
  chartData?: ChartAnalysis;
  timestamp: string;
}

export interface AltTextGeneration {
  generatedText: string;
  detectedObjects: DetectedObject[];
  confidence: number;
  suggestions: string[];
}

export interface DetectedObject {
  label: string;
  confidence: number;
  boundingBox: BoundingBox;
}

export interface BoundingBox {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface ColorContrastAnalysis {
  overallScore: number;
  wcagAaCompliant: boolean;
  wcagAaaCompliant: boolean;
  problematicRegions: ContrastRegion[];
  detectedTextRegions: number;
  averageContrastRatio: number;
}

export interface ContrastRegion {
  foreground: Color;
  background: Color;
  contrastRatio: number;
  wcagLevel: 'AAA' | 'AA' | 'A' | 'Fail';
  region: BoundingBox;
  suggestion: string;
}

export interface Color {
  r: number;
  g: number;
  b: number;
  a?: number;
}

export interface VisualHierarchy {
  primaryElements: HierarchyElement[];
  secondaryElements: HierarchyElement[];
  tertiaryElements: HierarchyElement[];
  readingOrder: number[];
  focusFlowScore: number;
  issues: string[];
}

export interface HierarchyElement {
  elementType: string;
  importance: number;
  region: BoundingBox;
  semanticRole?: string;
}

export interface IconRecognition {
  iconType: string;
  confidence: number;
  boundingBox: BoundingBox;
  hasLabel: boolean;
  suggestedLabel: string;
  isDecorative: boolean;
}

export interface ChartAnalysis {
  chartType: ChartType;
  title?: string;
  axes: Axis[];
  dataSeries: DataSeries[];
  hasLegend: boolean;
  hasDataLabels: boolean;
  accessibilityIssues: string[];
  suggestedTextAlternative?: string;
}

export type ChartType = 'BarChart' | 'LineChart' | 'PieChart' | 'ScatterPlot' | 'AreaChart' | { Other: string };

export interface Axis {
  label: string;
  axisType: 'Categorical' | 'Numerical' | 'Temporal';
  values: string[];
}

export interface DataSeries {
  label: string;
  values: number[];
  color?: Color;
}

// ============================================================================
// NLP Analysis Types
// ============================================================================

export interface TextAnalysis {
  readability?: ReadabilityScore;
  plainLanguageSuggestions: PlainLanguageSuggestion[];
}

export interface ReadabilityScore {
  fleschReadingEase: number;
  fleschKincaidGrade: number;
  smogIndex: number;
  gunningFog: number;
  automatedReadabilityIndex: number;
  colemanLiauIndex: number;
  averageGradeLevel: number;
  isAccessible: boolean;
  metrics: ReadabilityMetrics;
  recommendations: string[];
}

export interface ReadabilityMetrics {
  totalWords: number;
  totalSentences: number;
  totalSyllables: number;
  totalCharacters: number;
  wordsBySyllables: Record<number, number>;
  sentenceLengths: number[];
  averageSentenceLength: number;
  averageWordLength: number;
}

export interface PlainLanguageSuggestion {
  originalText: string;
  suggestion: string;
  replacement?: string;
  confidence: number;
  category: SuggestionCategory;
  position: number;
  length: number;
}

export type SuggestionCategory =
  | 'PassiveVoice'
  | 'Jargon'
  | 'WordyPhrase'
  | 'Nominalization'
  | 'ComplexSentence'
  | 'Other';

export interface HeadingAnalysis {
  totalHeadings: number;
  hierarchyDepth: number;
  isValidHierarchy: boolean;
  issues: string[];
  suggestions: string[];
}

export interface LinkTextAnalysis {
  text: string;
  url: string;
  isAccessible: boolean;
  issues: string[];
  suggestions: string[];
}

export interface FormLabelQuality {
  labelText: string;
  inputType: string;
  isAccessible: boolean;
  issues: string[];
  suggestions: string[];
}

// ============================================================================
// Predictions Types
// ============================================================================

export interface TrendPrediction {
  trendDirection: 'Increasing' | 'Decreasing' | 'Stable';
  currentValue: number;
  predictedValue: number;
  predictionDate: string;
  confidence: number;
  rSquared: number;
  issues: IssueTrend[];
}

export interface IssueTrend {
  category: string;
  trend: 'Increasing' | 'Decreasing' | 'Stable';
  severity: 'High' | 'Medium' | 'Low';
  description: string;
  recommendation: string;
}

export interface RemediationEstimate {
  totalIssues: number;
  estimatedHours: number;
  estimatedHoursMin: number;
  estimatedHoursMax: number;
  complexityFactor: number;
  confidence: number;
  recommendedTeamSize: number;
  estimatedCompletionDays: number;
  breakdown: Record<string, number>;
}

export interface ComplianceRisk {
  riskScore: number;
  riskLevel: RiskLevel;
  totalViolations: number;
  severityBreakdown: Record<string, number>;
  compliancePercentage: number;
  mitigationStrategies: string[];
  estimatedTimeToComplianceDays: number;
}

export type RiskLevel = 'Critical' | 'High' | 'Medium' | 'Low' | 'Minimal';

export interface RegressionProbability {
  probability: number;
  riskLevel: RiskLevel;
  filePath: string;
  changeType: string;
  confidence: number;
  recommendations: string[];
}

export interface ImpactScore {
  overallScore: number;
  userImpactScore: number;
  frequencyScore: number;
  complianceScore: number;
  remediationScore: number;
  priority: string;
  recommendedAction: string;
}

// ============================================================================
// Suggestions Types
// ============================================================================

export interface AutoFix {
  issueId: string;
  fixType: string;
  originalCode: string;
  fixedCode: string;
  diff: string;
  confidence: SuggestionConfidence;
  requiresManualReview: boolean;
  explanation: string;
  wcagCriteria: string[];
  applied: boolean;
  timestamp: string;
}

export interface CodeCompletion {
  completionText: string;
  displayText: string;
  description: string;
  confidence: SuggestionConfidence;
  category: string;
}

export interface AltTextSuggestion {
  suggestedText: string;
  confidence: SuggestionConfidence;
  reasoning: string;
  isDecorative: boolean;
}

export interface ARIASuggestion {
  attributeName: string;
  suggestedValue: string;
  reason: string;
  confidence: SuggestionConfidence;
  isRequired: boolean;
  wcagCriteria: string[];
}

export interface BestPracticeSuggestion {
  title: string;
  description: string;
  priority: number;
  category: string;
  exampleCode?: string;
  wcagReference: string[];
}

export type SuggestionConfidence = 'High' | 'Medium' | 'Low';

// ============================================================================
// AI Assistant Types
// ============================================================================

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: string;
  suggestions?: AutoFix[];
  metadata?: Record<string, any>;
}

export interface AIContext {
  currentPage?: string;
  selectedElement?: string;
  recentIssues: string[];
  conversationHistory: ChatMessage[];
}

export interface AssistantConfig {
  enableAutoSuggestions: boolean;
  enableContextAwareness: boolean;
  maxHistoryLength: number;
  suggestionDelay: number;
}

// ============================================================================
// Insights Types
// ============================================================================

export interface AIInsights {
  summary: InsightSummary;
  trends: TrendPrediction[];
  riskAssessment: ComplianceRisk;
  recommendations: Recommendation[];
  predictions: Prediction[];
  timestamp: string;
}

export interface InsightSummary {
  totalIssuesAnalyzed: number;
  avgConfidence: number;
  topCategories: CategoryCount[];
  estimatedEffort: number;
  complianceScore: number;
}

export interface CategoryCount {
  category: string;
  count: number;
  percentage: number;
}

export interface Recommendation {
  id: string;
  title: string;
  description: string;
  priority: number;
  impact: 'High' | 'Medium' | 'Low';
  effort: 'High' | 'Medium' | 'Low';
  category: string;
  actionItems: string[];
}

export interface Prediction {
  type: string;
  prediction: string;
  confidence: number;
  timeframe: string;
  impact: string;
}

// ============================================================================
// API Response Types
// ============================================================================

export interface AIAnalysisRequest {
  type: 'vision' | 'nlp' | 'prediction' | 'suggestion';
  data: any;
  options?: {
    enableCache?: boolean;
    priority?: number;
    timeout?: number;
  };
}

export interface AIAnalysisResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  metadata: {
    inferenceTime: number;
    modelVersion: string;
    confidence: number;
    timestamp: string;
  };
}

// ============================================================================
// Component Props Types
// ============================================================================

export interface AIAssistantProps {
  onSuggestionApply?: (suggestion: AutoFix) => void;
  onClose?: () => void;
  initialContext?: AIContext;
  config?: AssistantConfig;
}

export interface SuggestionPanelProps {
  suggestions: AutoFix[];
  onApply: (suggestion: AutoFix) => void;
  onDismiss: (suggestionId: string) => void;
  onPreview?: (suggestion: AutoFix) => void;
  loading?: boolean;
}

export interface InsightsViewProps {
  insights: AIInsights;
  onRefresh?: () => void;
  onRecommendationClick?: (recommendation: Recommendation) => void;
  timeRange?: {
    start: Date;
    end: Date;
  };
}

// ============================================================================
// Utility Types
// ============================================================================

export interface LoadingState {
  isLoading: boolean;
  message?: string;
  progress?: number;
}

export interface ErrorState {
  hasError: boolean;
  message?: string;
  code?: string;
  details?: any;
}

export type AsyncState<T> = {
  data: T | null;
  loading: boolean;
  error: string | null;
};
