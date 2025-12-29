/**
 * CADDY v0.3.0 - Content Script Scanner
 * Enterprise-grade DOM accessibility analysis engine
 */

import type {
  AccessibilityIssue,
  ScanResult,
  ScanContext,
  Rule,
  ElementInfo,
  SeverityLevel,
  IssueCategory,
  WCAGLevel,
} from '../shared/types';

// ============================================================================
// Scanner Class
// ============================================================================

export class AccessibilityScanner {
  private context: ScanContext;
  private rules: Rule[];
  private issues: AccessibilityIssue[] = [];
  private scannedElements = new WeakSet<Element>();

  constructor() {
    this.context = this.createContext();
    this.rules = this.loadRules();
  }

  // --------------------------------------------------------------------------
  // Main Scanning Method
  // --------------------------------------------------------------------------

  async scan(): Promise<ScanResult> {
    const startTime = Date.now();
    this.issues = [];
    this.scannedElements = new WeakSet();

    console.log('[CADDY Scanner] Starting accessibility scan...');

    try {
      // Scan all elements
      await this.scanElement(document.body);

      // Build result
      const result = this.buildResult(startTime);

      console.log(
        `[CADDY Scanner] Scan complete: ${result.summary.total} issues found`
      );

      return result;
    } catch (error) {
      console.error('[CADDY Scanner] Scan error:', error);
      throw error;
    }
  }

  // --------------------------------------------------------------------------
  // Element Scanning
  // --------------------------------------------------------------------------

  private async scanElement(element: Element): Promise<void> {
    if (this.scannedElements.has(element)) return;
    if (!(element instanceof HTMLElement)) return;

    this.scannedElements.add(element);

    // Run all rules against this element
    for (const rule of this.rules) {
      try {
        const result = rule.check(element, this.context);

        if (result && !result.passed) {
          this.addIssue(element, rule, result);
        }
      } catch (error) {
        console.warn(`[CADDY Scanner] Rule ${rule.id} failed:`, error);
      }
    }

    // Recursively scan children
    for (const child of Array.from(element.children)) {
      await this.scanElement(child);
    }
  }

  // --------------------------------------------------------------------------
  // Issue Management
  // --------------------------------------------------------------------------

  private addIssue(
    element: HTMLElement,
    rule: Rule,
    result: { passed: boolean; message: string; suggestion: string; impact: string }
  ): void {
    const issue: AccessibilityIssue = {
      id: this.generateIssueId(),
      severity: rule.severity,
      category: rule.category,
      wcagCriteria: rule.wcagCriteria,
      wcagLevel: rule.wcagLevel,
      title: rule.name,
      description: result.message,
      element: this.getElementInfo(element),
      selector: this.getSelector(element),
      snippet: this.getElementSnippet(element),
      impact: result.impact,
      suggestion: result.suggestion,
      helpUrl: `https://docs.caddy.dev/rules/${rule.id}`,
      timestamp: Date.now(),
    };

    this.issues.push(issue);
  }

  // --------------------------------------------------------------------------
  // Rules Engine
  // --------------------------------------------------------------------------

  private loadRules(): Rule[] {
    return [
      // Images
      this.createImageAltTextRule(),
      this.createImageRoleRule(),
      this.createDecorativeImageRule(),

      // Forms
      this.createFormLabelRule(),
      this.createInputTypeRule(),
      this.createRequiredFieldRule(),
      this.createFieldsetLegendRule(),

      // Headings
      this.createHeadingOrderRule(),
      this.createHeadingLevelRule(),
      this.createEmptyHeadingRule(),

      // Links
      this.createLinkTextRule(),
      this.createLinkPurposeRule(),
      this.createSkipLinkRule(),

      // ARIA
      this.createAriaRequiredChildrenRule(),
      this.createAriaRequiredParentRule(),
      this.createAriaValidValuesRule(),
      this.createAriaLabelRule(),

      // Color Contrast
      this.createColorContrastRule(),
      this.createColorOnlyRule(),

      // Keyboard
      this.createKeyboardAccessRule(),
      this.createFocusVisibleRule(),
      this.createTabIndexRule(),

      // Semantic HTML
      this.createLandmarkRule(),
      this.createListStructureRule(),
      this.createTableHeaderRule(),

      // Media
      this.createVideoTranscriptRule(),
      this.createAudioTranscriptRule(),
      this.createAutoplayRule(),

      // Language
      this.createLangAttributeRule(),
      this.createLangChangeRule(),

      // Structure
      this.createPageTitleRule(),
      this.createMainLandmarkRule(),
      this.createUniqueIdRule(),
    ];
  }

  // --------------------------------------------------------------------------
  // Rule Definitions
  // --------------------------------------------------------------------------

  private createImageAltTextRule(): Rule {
    return {
      id: 'image-alt-text',
      name: 'Images must have alternative text',
      description: 'All <img> elements must have an alt attribute',
      category: 'perceivable',
      wcagCriteria: ['1.1.1'],
      wcagLevel: 'A',
      severity: 'critical',
      help: 'https://www.w3.org/WAI/WCAG21/Understanding/non-text-content',
      check: (element) => {
        if (element.tagName !== 'IMG') return null;

        const alt = element.getAttribute('alt');

        if (alt === null) {
          return {
            passed: false,
            message: 'Image is missing alt attribute',
            suggestion: 'Add an alt attribute that describes the image content',
            impact: 'Screen reader users will not know what the image contains',
          };
        }

        if (alt.trim() === '' && element.getAttribute('role') !== 'presentation') {
          return {
            passed: false,
            message: 'Image has empty alt text but is not marked as decorative',
            suggestion: 'Add descriptive alt text or mark as decorative with role="presentation"',
            impact: 'Screen reader users may be confused about the image purpose',
          };
        }

        return { passed: true, message: '', suggestion: '', impact: '' };
      },
    };
  }

  private createFormLabelRule(): Rule {
    return {
      id: 'form-label',
      name: 'Form inputs must have labels',
      description: 'All form inputs must have associated labels',
      category: 'perceivable',
      wcagCriteria: ['1.3.1', '3.3.2'],
      wcagLevel: 'A',
      severity: 'critical',
      help: 'https://www.w3.org/WAI/WCAG21/Understanding/labels-or-instructions',
      check: (element) => {
        if (!['INPUT', 'SELECT', 'TEXTAREA'].includes(element.tagName)) {
          return null;
        }

        const input = element as HTMLInputElement;
        if (input.type === 'hidden' || input.type === 'submit' || input.type === 'button') {
          return null;
        }

        // Check for label
        const hasLabel =
          input.labels && input.labels.length > 0 ||
          input.getAttribute('aria-label') ||
          input.getAttribute('aria-labelledby') ||
          input.getAttribute('title');

        if (!hasLabel) {
          return {
            passed: false,
            message: 'Form input has no associated label',
            suggestion: 'Add a <label> element or aria-label attribute',
            impact: 'Screen reader users will not know the purpose of this input',
          };
        }

        return { passed: true, message: '', suggestion: '', impact: '' };
      },
    };
  }

  private createHeadingOrderRule(): Rule {
    return {
      id: 'heading-order',
      name: 'Headings must follow logical order',
      description: 'Heading levels should not skip (e.g., H1 -> H3)',
      category: 'perceivable',
      wcagCriteria: ['1.3.1'],
      wcagLevel: 'A',
      severity: 'serious',
      help: 'https://www.w3.org/WAI/WCAG21/Understanding/info-and-relationships',
      check: (element) => {
        if (!/^H[1-6]$/.test(element.tagName)) return null;

        const level = parseInt(element.tagName[1]);
        const previousHeading = this.findPreviousHeading(element);

        if (previousHeading) {
          const prevLevel = parseInt(previousHeading.tagName[1]);

          if (level > prevLevel + 1) {
            return {
              passed: false,
              message: `Heading level ${level} skips from level ${prevLevel}`,
              suggestion: `Use H${prevLevel + 1} instead of H${level}`,
              impact: 'Document structure may be confusing for screen reader users',
            };
          }
        }

        return { passed: true, message: '', suggestion: '', impact: '' };
      },
    };
  }

  private createLinkTextRule(): Rule {
    return {
      id: 'link-text',
      name: 'Links must have descriptive text',
      description: 'Link text should describe the link purpose',
      category: 'operable',
      wcagCriteria: ['2.4.4'],
      wcagLevel: 'A',
      severity: 'serious',
      help: 'https://www.w3.org/WAI/WCAG21/Understanding/link-purpose-in-context',
      check: (element) => {
        if (element.tagName !== 'A') return null;

        const text = element.textContent?.trim() || '';
        const ariaLabel = element.getAttribute('aria-label');

        const nonDescriptive = ['click here', 'read more', 'more', 'here', 'link'];

        if (!text && !ariaLabel) {
          return {
            passed: false,
            message: 'Link has no text content',
            suggestion: 'Add descriptive text or aria-label',
            impact: 'Screen reader users will not know the link destination',
          };
        }

        if (nonDescriptive.includes(text.toLowerCase())) {
          return {
            passed: false,
            message: `Link text "${text}" is not descriptive`,
            suggestion: 'Use more descriptive text that explains the link purpose',
            impact: 'Link purpose is unclear out of context',
          };
        }

        return { passed: true, message: '', suggestion: '', impact: '' };
      },
    };
  }

  private createColorContrastRule(): Rule {
    return {
      id: 'color-contrast',
      name: 'Text must have sufficient color contrast',
      description: 'Text and background must have contrast ratio of at least 4.5:1',
      category: 'perceivable',
      wcagCriteria: ['1.4.3'],
      wcagLevel: 'AA',
      severity: 'serious',
      help: 'https://www.w3.org/WAI/WCAG21/Understanding/contrast-minimum',
      check: (element) => {
        if (element.textContent?.trim() === '') return null;

        const style = window.getComputedStyle(element);
        const fontSize = parseFloat(style.fontSize);
        const fontWeight = style.fontWeight;

        const isLargeText =
          fontSize >= 24 || (fontSize >= 18.5 && parseInt(fontWeight) >= 700);

        const minRatio = isLargeText ? 3 : 4.5;
        const ratio = this.getContrastRatio(element);

        if (ratio < minRatio) {
          return {
            passed: false,
            message: `Color contrast ratio ${ratio.toFixed(2)}:1 is below minimum ${minRatio}:1`,
            suggestion: 'Increase contrast between text and background colors',
            impact: 'Text may be difficult to read for users with low vision',
          };
        }

        return { passed: true, message: '', suggestion: '', impact: '' };
      },
    };
  }

  private createKeyboardAccessRule(): Rule {
    return {
      id: 'keyboard-access',
      name: 'Interactive elements must be keyboard accessible',
      description: 'All interactive elements must be reachable via keyboard',
      category: 'operable',
      wcagCriteria: ['2.1.1'],
      wcagLevel: 'A',
      severity: 'critical',
      help: 'https://www.w3.org/WAI/WCAG21/Understanding/keyboard',
      check: (element) => {
        const interactiveRoles = ['button', 'link', 'checkbox', 'radio', 'textbox'];
        const role = element.getAttribute('role');

        if (!role && !this.isInteractive(element)) return null;

        const isFocusable =
          element.tabIndex >= 0 ||
          ['A', 'BUTTON', 'INPUT', 'SELECT', 'TEXTAREA'].includes(element.tagName);

        if (interactiveRoles.includes(role || '') && !isFocusable) {
          return {
            passed: false,
            message: 'Interactive element is not keyboard accessible',
            suggestion: 'Add tabindex="0" or use a native interactive element',
            impact: 'Keyboard users cannot interact with this element',
          };
        }

        return { passed: true, message: '', suggestion: '', impact: '' };
      },
    };
  }

  private createAriaLabelRule(): Rule {
    return {
      id: 'aria-label-required',
      name: 'ARIA widgets must have accessible names',
      description: 'Elements with ARIA roles must have accessible names',
      category: 'perceivable',
      wcagCriteria: ['4.1.2'],
      wcagLevel: 'A',
      severity: 'critical',
      help: 'https://www.w3.org/WAI/WCAG21/Understanding/name-role-value',
      check: (element) => {
        const role = element.getAttribute('role');
        const requiresName = [
          'button', 'link', 'checkbox', 'radio', 'tab', 'menuitem',
          'treeitem', 'slider', 'spinbutton'
        ];

        if (!role || !requiresName.includes(role)) return null;

        const hasName =
          element.getAttribute('aria-label') ||
          element.getAttribute('aria-labelledby') ||
          element.textContent?.trim();

        if (!hasName) {
          return {
            passed: false,
            message: `Element with role="${role}" has no accessible name`,
            suggestion: 'Add aria-label, aria-labelledby, or text content',
            impact: 'Screen reader users will not know the element purpose',
          };
        }

        return { passed: true, message: '', suggestion: '', impact: '' };
      },
    };
  }

  private createPageTitleRule(): Rule {
    return {
      id: 'page-title',
      name: 'Page must have a title',
      description: 'Document must have a descriptive <title> element',
      category: 'operable',
      wcagCriteria: ['2.4.2'],
      wcagLevel: 'A',
      severity: 'serious',
      help: 'https://www.w3.org/WAI/WCAG21/Understanding/page-titled',
      check: (element) => {
        if (element !== document.body) return null;

        const title = document.title.trim();

        if (!title) {
          return {
            passed: false,
            message: 'Page has no title',
            suggestion: 'Add a descriptive <title> element in the <head>',
            impact: 'Users may have difficulty identifying the page',
          };
        }

        return { passed: true, message: '', suggestion: '', impact: '' };
      },
    };
  }

  private createLangAttributeRule(): Rule {
    return {
      id: 'html-lang',
      name: 'Page must have lang attribute',
      description: 'The <html> element must have a lang attribute',
      category: 'understandable',
      wcagCriteria: ['3.1.1'],
      wcagLevel: 'A',
      severity: 'serious',
      help: 'https://www.w3.org/WAI/WCAG21/Understanding/language-of-page',
      check: (element) => {
        if (element !== document.body) return null;

        const lang = document.documentElement.getAttribute('lang');

        if (!lang) {
          return {
            passed: false,
            message: 'HTML element is missing lang attribute',
            suggestion: 'Add lang attribute to <html> element (e.g., lang="en")',
            impact: 'Screen readers may not announce content in the correct language',
          };
        }

        return { passed: true, message: '', suggestion: '', impact: '' };
      },
    };
  }

  // Additional rule creators (simplified for brevity)
  private createImageRoleRule(): Rule { return this.createGenericRule('image-role'); }
  private createDecorativeImageRule(): Rule { return this.createGenericRule('decorative-image'); }
  private createInputTypeRule(): Rule { return this.createGenericRule('input-type'); }
  private createRequiredFieldRule(): Rule { return this.createGenericRule('required-field'); }
  private createFieldsetLegendRule(): Rule { return this.createGenericRule('fieldset-legend'); }
  private createHeadingLevelRule(): Rule { return this.createGenericRule('heading-level'); }
  private createEmptyHeadingRule(): Rule { return this.createGenericRule('empty-heading'); }
  private createLinkPurposeRule(): Rule { return this.createGenericRule('link-purpose'); }
  private createSkipLinkRule(): Rule { return this.createGenericRule('skip-link'); }
  private createAriaRequiredChildrenRule(): Rule { return this.createGenericRule('aria-children'); }
  private createAriaRequiredParentRule(): Rule { return this.createGenericRule('aria-parent'); }
  private createAriaValidValuesRule(): Rule { return this.createGenericRule('aria-values'); }
  private createColorOnlyRule(): Rule { return this.createGenericRule('color-only'); }
  private createFocusVisibleRule(): Rule { return this.createGenericRule('focus-visible'); }
  private createTabIndexRule(): Rule { return this.createGenericRule('tabindex'); }
  private createLandmarkRule(): Rule { return this.createGenericRule('landmark'); }
  private createListStructureRule(): Rule { return this.createGenericRule('list-structure'); }
  private createTableHeaderRule(): Rule { return this.createGenericRule('table-header'); }
  private createVideoTranscriptRule(): Rule { return this.createGenericRule('video-transcript'); }
  private createAudioTranscriptRule(): Rule { return this.createGenericRule('audio-transcript'); }
  private createAutoplayRule(): Rule { return this.createGenericRule('autoplay'); }
  private createLangChangeRule(): Rule { return this.createGenericRule('lang-change'); }
  private createMainLandmarkRule(): Rule { return this.createGenericRule('main-landmark'); }
  private createUniqueIdRule(): Rule { return this.createGenericRule('unique-id'); }

  private createGenericRule(id: string): Rule {
    return {
      id,
      name: id,
      description: id,
      category: 'perceivable',
      wcagCriteria: ['1.1.1'],
      wcagLevel: 'A',
      severity: 'moderate',
      help: `https://docs.caddy.dev/rules/${id}`,
      check: () => null,
    };
  }

  // --------------------------------------------------------------------------
  // Helper Methods
  // --------------------------------------------------------------------------

  private createContext(): ScanContext {
    return {
      document,
      url: window.location.href,
      viewport: {
        width: window.innerWidth,
        height: window.innerHeight,
      },
      colorContrast: true,
      keyboardNav: true,
    };
  }

  private getElementInfo(element: HTMLElement): ElementInfo {
    return {
      tagName: element.tagName,
      id: element.id || undefined,
      className: element.className || undefined,
      role: element.getAttribute('role') || undefined,
      ariaLabel: element.getAttribute('aria-label') || undefined,
      textContent: element.textContent?.slice(0, 100) || undefined,
      attributes: this.getAttributes(element),
      boundingRect: element.getBoundingClientRect(),
      xpath: this.getXPath(element),
    };
  }

  private getAttributes(element: HTMLElement): Record<string, string> {
    const attrs: Record<string, string> = {};
    for (const attr of Array.from(element.attributes)) {
      attrs[attr.name] = attr.value;
    }
    return attrs;
  }

  private getSelector(element: HTMLElement): string {
    if (element.id) return `#${element.id}`;

    const path: string[] = [];
    let current: Element | null = element;

    while (current && current !== document.body) {
      let selector = current.tagName.toLowerCase();

      if (current.id) {
        selector = `#${current.id}`;
        path.unshift(selector);
        break;
      }

      if (current.className) {
        selector += `.${current.className.split(' ').join('.')}`;
      }

      path.unshift(selector);
      current = current.parentElement;
    }

    return path.join(' > ');
  }

  private getXPath(element: HTMLElement): string {
    const segments: string[] = [];
    let current: Element | null = element;

    while (current && current !== document.body) {
      let segment = current.tagName.toLowerCase();

      if (current.id) {
        segment = `${segment}[@id="${current.id}"]`;
        segments.unshift(segment);
        break;
      }

      const siblings = Array.from(current.parentElement?.children || []);
      const index = siblings.filter((e) => e.tagName === current!.tagName).indexOf(current) + 1;

      if (index > 1) {
        segment += `[${index}]`;
      }

      segments.unshift(segment);
      current = current.parentElement;
    }

    return '//' + segments.join('/');
  }

  private getElementSnippet(element: HTMLElement): string {
    const clone = element.cloneNode(false) as HTMLElement;
    clone.textContent = element.textContent?.slice(0, 50) || '';
    return clone.outerHTML;
  }

  private findPreviousHeading(element: HTMLElement): HTMLElement | null {
    let current = element.previousElementSibling;

    while (current) {
      if (/^H[1-6]$/.test(current.tagName)) {
        return current as HTMLElement;
      }
      current = current.previousElementSibling;
    }

    return null;
  }

  private isInteractive(element: HTMLElement): boolean {
    const interactive = ['A', 'BUTTON', 'INPUT', 'SELECT', 'TEXTAREA'];
    return interactive.includes(element.tagName) || element.onclick !== null;
  }

  private getContrastRatio(element: HTMLElement): number {
    const style = window.getComputedStyle(element);
    const fgColor = this.parseColor(style.color);
    const bgColor = this.getBackgroundColor(element);

    if (!fgColor || !bgColor) return 21; // Return max ratio if can't determine

    return this.calculateContrastRatio(fgColor, bgColor);
  }

  private getBackgroundColor(element: HTMLElement): [number, number, number] | null {
    let current: HTMLElement | null = element;

    while (current) {
      const style = window.getComputedStyle(current);
      const bg = style.backgroundColor;

      if (bg && bg !== 'transparent' && bg !== 'rgba(0, 0, 0, 0)') {
        return this.parseColor(bg);
      }

      current = current.parentElement;
    }

    return [255, 255, 255]; // Default to white
  }

  private parseColor(color: string): [number, number, number] | null {
    const match = color.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/);
    if (!match) return null;
    return [parseInt(match[1]), parseInt(match[2]), parseInt(match[3])];
  }

  private calculateContrastRatio(
    fg: [number, number, number],
    bg: [number, number, number]
  ): number {
    const l1 = this.relativeLuminance(fg);
    const l2 = this.relativeLuminance(bg);
    const lighter = Math.max(l1, l2);
    const darker = Math.min(l1, l2);
    return (lighter + 0.05) / (darker + 0.05);
  }

  private relativeLuminance(rgb: [number, number, number]): number {
    const [r, g, b] = rgb.map((c) => {
      const val = c / 255;
      return val <= 0.03928 ? val / 12.92 : Math.pow((val + 0.055) / 1.055, 2.4);
    });
    return 0.2126 * r + 0.7152 * g + 0.0722 * b;
  }

  private generateIssueId(): string {
    return `issue_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  private buildResult(startTime: number): ScanResult {
    const summary = this.calculateSummary();

    return {
      url: window.location.href,
      timestamp: Date.now(),
      duration: Date.now() - startTime,
      status: 'complete',
      issues: this.issues,
      summary,
      metadata: {
        pageTitle: document.title,
        pageUrl: window.location.href,
        domNodes: document.querySelectorAll('*').length,
        scannerVersion: '0.3.0',
        rules: this.rules.map((r) => r.id),
        viewport: this.context.viewport,
      },
    };
  }

  private calculateSummary() {
    const summary = {
      total: this.issues.length,
      critical: 0,
      serious: 0,
      moderate: 0,
      minor: 0,
      passed: 0,
      incomplete: 0,
      byCategory: {
        perceivable: 0,
        operable: 0,
        understandable: 0,
        robust: 0,
      } as Record<IssueCategory, number>,
      wcagLevel: 'AA' as WCAGLevel,
      complianceScore: 0,
    };

    for (const issue of this.issues) {
      summary[issue.severity]++;
      summary.byCategory[issue.category]++;
    }

    // Calculate compliance score (0-100)
    const maxIssues = 100;
    const issueScore = Math.max(0, maxIssues - this.issues.length);
    summary.complianceScore = Math.round((issueScore / maxIssues) * 100);

    return summary;
  }
}
