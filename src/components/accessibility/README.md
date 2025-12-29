# Accessibility Dashboard - CADDY v0.3.0

Enterprise-grade accessibility monitoring and compliance tracking for CADDY applications.

## Overview

The Accessibility Dashboard provides comprehensive tools for monitoring, tracking, and improving the accessibility of your application. Built with WCAG 2.2, Section 508, and ADA compliance in mind.

## Components

### AccessibilityDashboard

Main dashboard displaying real-time accessibility metrics and overview.

```tsx
import { AccessibilityDashboard } from '@/components/accessibility';

function App() {
  return (
    <AccessibilityProvider>
      <AccessibilityDashboard
        onNavigateToIssues={() => navigate('/issues')}
        onNavigateToReports={() => navigate('/reports')}
        onNavigateToSettings={() => navigate('/settings')}
      />
    </AccessibilityProvider>
  );
}
```

**Features:**
- Real-time accessibility score with letter grade
- Issue breakdown by severity (Critical, Serious, Moderate, Minor)
- Compliance status indicators (WCAG, Section 508, ADA)
- Trend charts showing improvement over time
- Quick action buttons for common tasks
- Overview, compliance, and trends tabs

### IssueExplorer

Advanced issue browsing with filtering, sorting, and inline fixes.

```tsx
import { IssueExplorer } from '@/components/accessibility';

function IssuesPage() {
  return (
    <AccessibilityProvider>
      <IssueExplorer
        initialFilter={{
          levels: [IssueLevel.Critical, IssueLevel.Serious],
          status: ['open']
        }}
        onIssueClick={(issue) => console.log('Issue clicked:', issue)}
        showBulkActions={true}
      />
    </AccessibilityProvider>
  );
}
```

**Features:**
- Filterable by severity, category, status, assignee
- Sortable columns with multi-column sorting
- Real-time search with debouncing
- Element highlighting in page
- Inline code snippets with suggested fixes
- Bulk operations (mark as fixed, assign, etc.)
- Issue details modal with full context
- WCAG criteria mapping

### ComplianceReport

Comprehensive reporting with multiple export formats.

```tsx
import { ComplianceReport } from '@/components/accessibility';

function ReportsPage() {
  return (
    <AccessibilityProvider>
      <ComplianceReport
        defaultStandards={[
          ComplianceStandard.WCAG_2_1_AA,
          ComplianceStandard.Section508
        ]}
        onExport={(format, blob) => {
          console.log(`Report exported as ${format}`);
        }}
      />
    </AccessibilityProvider>
  );
}
```

**Features:**
- Executive summary view
- Detailed technical report
- Export to PDF, CSV, JSON, HTML, Excel
- Scheduled report generation (daily, weekly, monthly)
- Email delivery configuration
- Custom report builder with configurable sections
- Compliance status tracking
- Historical trend analysis

### AccessibilityProvider

React context provider for global state management.

```tsx
import { AccessibilityProvider } from '@/components/accessibility';

function App() {
  return (
    <AccessibilityProvider
      tenantId="my-tenant"
      apiBaseUrl="/api/accessibility"
    >
      {/* Your app components */}
    </AccessibilityProvider>
  );
}
```

**Features:**
- Multi-tenant support
- Real-time data synchronization
- Settings persistence
- Automatic refresh
- Error boundary handling
- API integration layer

## Hooks

### useAccessibility

Main hook for accessing accessibility context.

```tsx
const {
  issues,
  score,
  complianceStatus,
  startScan,
  updateIssue,
  generateReport
} = useAccessibility();
```

### useFilteredIssues

Filter and sort issues with advanced criteria.

```tsx
const filteredIssues = useFilteredIssues(
  {
    levels: [IssueLevel.Critical],
    categories: [IssueCategory.ColorContrast],
    searchQuery: 'button'
  },
  { field: 'detectedAt', direction: 'desc' }
);
```

### useIssueStats

Get aggregated statistics about issues.

```tsx
const {
  totalIssues,
  openIssues,
  fixedIssues,
  byLevel,
  byCategory,
  score,
  trend
} = useIssueStats();
```

### useBulkIssueOperations

Manage bulk operations on multiple issues.

```tsx
const {
  selectedIssues,
  toggleIssue,
  selectAll,
  deselectAll,
  bulkUpdate
} = useBulkIssueOperations();

// Mark multiple issues as fixed
await bulkUpdate({ status: 'fixed', fixedAt: new Date() });
```

### useIssueHighlight

Highlight elements in the page.

```tsx
const { highlight, unhighlight, highlightedIssueId } = useIssueHighlight();

// Highlight an issue's element
highlight(issue);

// Remove all highlights
unhighlight();
```

### useAccessibilityScore

Get score with color coding and grade.

```tsx
const scoreData = useAccessibilityScore();
// { overall: 85, grade: 'B', color: 'warning', trend: 'improving', change: 5.2 }
```

## Types

### IssueLevel
```typescript
enum IssueLevel {
  Critical = 'critical',
  Serious = 'serious',
  Moderate = 'moderate',
  Minor = 'minor'
}
```

### ComplianceStandard
```typescript
enum ComplianceStandard {
  WCAG_2_1_AA = 'WCAG 2.1 Level AA',
  WCAG_2_2_AA = 'WCAG 2.2 Level AA',
  Section508 = 'Section 508',
  ADA = 'ADA'
}
```

### AccessibilityIssue
```typescript
interface AccessibilityIssue {
  id: string;
  level: IssueLevel;
  category: IssueCategory;
  title: string;
  description: string;
  wcagCriteria: string[];
  element?: {
    selector: string;
    html: string;
    xpath: string;
  };
  suggestedFix?: string;
  codeSnippet?: string;
  fixedCodeSnippet?: string;
  status: 'open' | 'in-progress' | 'fixed' | 'wont-fix' | 'false-positive';
  detectedAt: Date;
  fixedAt?: Date;
}
```

## Accessibility Features

The dashboard itself follows WCAG 2.2 Level AAA guidelines:

- **Keyboard Navigation**: Full keyboard support with focus management
- **Screen Reader**: Proper ARIA labels and semantic HTML
- **Color Contrast**: Minimum 7:1 contrast ratio for all text
- **Focus Indicators**: Clear, visible focus indicators
- **Skip Links**: Skip to main content functionality
- **Reduced Motion**: Respects `prefers-reduced-motion`
- **High Contrast**: Supports high contrast mode
- **Responsive**: Mobile-friendly and touch-accessible

## Styling

Import the CSS file in your application:

```tsx
import '@/components/accessibility/accessibility.css';
```

The components use the enterprise theme system and can be customized via the ThemeProvider.

## API Integration

The components expect the following API endpoints:

- `GET /api/accessibility/score` - Current accessibility score
- `GET /api/accessibility/issues` - List all issues
- `POST /api/accessibility/scan/start` - Start new scan
- `GET /api/accessibility/compliance` - Compliance status
- `POST /api/accessibility/report/generate` - Generate report
- `PATCH /api/accessibility/issues/:id` - Update issue
- `DELETE /api/accessibility/issues/:id` - Delete issue

## Multi-Tenant Support

For multi-tenant applications, pass the `tenantId` prop:

```tsx
<AccessibilityProvider tenantId={currentUser.tenantId}>
  <AccessibilityDashboard />
</AccessibilityProvider>
```

## Best Practices

1. **Regular Scanning**: Run scans on every deployment
2. **CI/CD Integration**: Fail builds on critical issues
3. **Team Training**: Educate developers on accessibility
4. **Progressive Enhancement**: Fix high-severity issues first
5. **User Testing**: Test with assistive technologies
6. **Documentation**: Document accessibility decisions

## Browser Support

- Chrome/Edge 90+
- Firefox 88+
- Safari 14+
- Mobile browsers with touch support

## License

Part of CADDY v0.3.0 Enterprise Edition
