/**
 * CADDY v0.3.0 - Issue Highlighter
 * Visual overlay system for accessibility issues
 */

import type {
  AccessibilityIssue,
  HighlightConfig,
  HighlightStyle,
  SeverityLevel,
} from '../shared/types';

// ============================================================================
// Highlighter Class
// ============================================================================

export class IssueHighlighter {
  private overlays = new Map<string, HTMLElement>();
  private tooltips = new Map<string, HTMLElement>();
  private enabled = true;
  private config: HighlightConfig;
  private container: HTMLElement | null = null;

  constructor() {
    this.config = this.getDefaultConfig();
    this.init();
  }

  // --------------------------------------------------------------------------
  // Initialization
  // --------------------------------------------------------------------------

  private init(): void {
    // Create container for overlays
    this.container = document.createElement('div');
    this.container.id = 'caddy-highlighter-container';
    this.container.setAttribute('aria-hidden', 'true');
    this.container.style.cssText = `
      position: fixed;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      pointer-events: none;
      z-index: 2147483647;
    `;

    document.body.appendChild(this.container);

    // Listen for scroll and resize events
    window.addEventListener('scroll', () => this.updatePositions(), true);
    window.addEventListener('resize', () => this.updatePositions());
  }

  // --------------------------------------------------------------------------
  // Public Methods
  // --------------------------------------------------------------------------

  highlightIssues(issues: AccessibilityIssue[]): void {
    this.clear();

    for (const issue of issues) {
      this.highlightIssue(issue);
    }
  }

  highlightIssue(issue: AccessibilityIssue): void {
    const element = this.findElement(issue.element.xpath, issue.selector);
    if (!element) {
      console.warn('[CADDY Highlighter] Element not found:', issue.selector);
      return;
    }

    const overlay = this.createOverlay(issue, element);
    const tooltip = this.createTooltip(issue);

    this.overlays.set(issue.id, overlay);
    this.tooltips.set(issue.id, tooltip);

    if (this.container) {
      this.container.appendChild(overlay);
      this.container.appendChild(tooltip);
    }

    this.attachEventListeners(overlay, tooltip, element);
  }

  unhighlightIssue(issueId: string): void {
    const overlay = this.overlays.get(issueId);
    const tooltip = this.tooltips.get(issueId);

    if (overlay) {
      overlay.remove();
      this.overlays.delete(issueId);
    }

    if (tooltip) {
      tooltip.remove();
      this.tooltips.delete(issueId);
    }
  }

  toggle(): void {
    this.enabled = !this.enabled;

    if (this.container) {
      this.container.style.display = this.enabled ? 'block' : 'none';
    }
  }

  clear(): void {
    this.overlays.forEach((overlay) => overlay.remove());
    this.tooltips.forEach((tooltip) => tooltip.remove());
    this.overlays.clear();
    this.tooltips.clear();
  }

  destroy(): void {
    this.clear();
    this.container?.remove();
  }

  updateConfig(config: Partial<HighlightConfig>): void {
    this.config = { ...this.config, ...config };
    this.reapplyStyles();
  }

  // --------------------------------------------------------------------------
  // Overlay Creation
  // --------------------------------------------------------------------------

  private createOverlay(issue: AccessibilityIssue, element: HTMLElement): HTMLElement {
    const rect = element.getBoundingClientRect();
    const style = this.getStyleForSeverity(issue.severity);

    const overlay = document.createElement('div');
    overlay.className = 'caddy-issue-overlay';
    overlay.dataset.issueId = issue.id;
    overlay.dataset.severity = issue.severity;

    overlay.style.cssText = `
      position: absolute;
      left: ${rect.left + window.scrollX}px;
      top: ${rect.top + window.scrollY}px;
      width: ${rect.width}px;
      height: ${rect.height}px;
      border: ${style.borderWidth}px ${style.borderStyle} ${style.color};
      background-color: ${style.backgroundColor};
      opacity: ${style.opacity};
      pointer-events: auto;
      cursor: pointer;
      transition: all 0.2s ease;
      z-index: ${style.zIndex};
      box-sizing: border-box;
    `;

    // Add severity badge
    const badge = this.createSeverityBadge(issue.severity);
    overlay.appendChild(badge);

    return overlay;
  }

  private createSeverityBadge(severity: SeverityLevel): HTMLElement {
    const badge = document.createElement('div');
    badge.className = 'caddy-severity-badge';

    const colors = {
      critical: '#DC2626',
      serious: '#F59E0B',
      moderate: '#3B82F6',
      minor: '#10B981',
    };

    badge.style.cssText = `
      position: absolute;
      top: -10px;
      left: -10px;
      width: 20px;
      height: 20px;
      border-radius: 50%;
      background-color: ${colors[severity]};
      border: 2px solid white;
      box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 10px;
      font-weight: bold;
      color: white;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    `;

    badge.textContent = severity[0].toUpperCase();
    badge.title = severity;

    return badge;
  }

  private createTooltip(issue: AccessibilityIssue): HTMLElement {
    const tooltip = document.createElement('div');
    tooltip.className = 'caddy-tooltip';
    tooltip.dataset.issueId = issue.id;

    tooltip.style.cssText = `
      position: absolute;
      background: white;
      border: 2px solid ${this.getSeverityColor(issue.severity)};
      border-radius: 6px;
      padding: 12px;
      max-width: 320px;
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      font-size: 14px;
      line-height: 1.5;
      z-index: 2147483646;
      pointer-events: auto;
      display: none;
    `;

    // Build tooltip content
    const content = `
      <div style="margin-bottom: 8px;">
        <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 6px;">
          <span style="
            display: inline-block;
            padding: 2px 8px;
            border-radius: 4px;
            background: ${this.getSeverityColor(issue.severity)};
            color: white;
            font-size: 11px;
            font-weight: 600;
            text-transform: uppercase;
          ">${issue.severity}</span>
          <span style="color: #666; font-size: 12px;">${issue.category}</span>
        </div>
        <div style="font-weight: 600; color: #1F2937; margin-bottom: 4px;">
          ${this.escapeHtml(issue.title)}
        </div>
      </div>
      <div style="color: #4B5563; margin-bottom: 8px; font-size: 13px;">
        ${this.escapeHtml(issue.description)}
      </div>
      <div style="
        background: #F3F4F6;
        padding: 8px;
        border-radius: 4px;
        margin-bottom: 8px;
        font-size: 12px;
      ">
        <div style="font-weight: 600; color: #374151; margin-bottom: 4px;">
          💡 Suggestion:
        </div>
        <div style="color: #6B7280;">
          ${this.escapeHtml(issue.suggestion)}
        </div>
      </div>
      <div style="
        display: flex;
        justify-content: space-between;
        align-items: center;
        font-size: 11px;
        color: #9CA3AF;
        border-top: 1px solid #E5E7EB;
        padding-top: 8px;
      ">
        <span>WCAG ${issue.wcagLevel}: ${issue.wcagCriteria.join(', ')}</span>
        <a href="${issue.helpUrl}" target="_blank" style="
          color: #3B82F6;
          text-decoration: none;
          font-weight: 500;
        ">Learn more →</a>
      </div>
    `;

    tooltip.innerHTML = content;

    return tooltip;
  }

  // --------------------------------------------------------------------------
  // Event Handlers
  // --------------------------------------------------------------------------

  private attachEventListeners(
    overlay: HTMLElement,
    tooltip: HTMLElement,
    element: HTMLElement
  ): void {
    let hideTimeout: number | null = null;

    const showTooltip = (e: MouseEvent) => {
      if (hideTimeout) {
        clearTimeout(hideTimeout);
        hideTimeout = null;
      }

      const rect = overlay.getBoundingClientRect();
      let top = rect.bottom + window.scrollY + 8;
      let left = rect.left + window.scrollX;

      // Adjust if tooltip would go off screen
      if (left + 320 > window.innerWidth) {
        left = window.innerWidth - 320 - 10;
      }

      if (top + 200 > window.innerHeight + window.scrollY) {
        top = rect.top + window.scrollY - 200 - 8;
      }

      tooltip.style.top = `${top}px`;
      tooltip.style.left = `${left}px`;
      tooltip.style.display = 'block';

      // Highlight the element
      overlay.style.opacity = '0.8';
      overlay.style.transform = 'scale(1.02)';
    };

    const hideTooltip = () => {
      hideTimeout = window.setTimeout(() => {
        tooltip.style.display = 'none';
        overlay.style.opacity = this.config.critical.opacity.toString();
        overlay.style.transform = 'scale(1)';
      }, 200);
    };

    overlay.addEventListener('mouseenter', showTooltip);
    overlay.addEventListener('mouseleave', hideTooltip);
    tooltip.addEventListener('mouseenter', () => {
      if (hideTimeout) {
        clearTimeout(hideTimeout);
        hideTimeout = null;
      }
    });
    tooltip.addEventListener('mouseleave', hideTooltip);

    // Click to inspect element
    overlay.addEventListener('click', (e) => {
      e.stopPropagation();
      element.scrollIntoView({ behavior: 'smooth', block: 'center' });
      element.focus({ preventScroll: true });

      // Flash the element
      const originalOutline = element.style.outline;
      element.style.outline = '3px solid #3B82F6';
      setTimeout(() => {
        element.style.outline = originalOutline;
      }, 1000);
    });
  }

  // --------------------------------------------------------------------------
  // Helper Methods
  // --------------------------------------------------------------------------

  private findElement(xpath: string, selector: string): HTMLElement | null {
    // Try selector first
    try {
      const element = document.querySelector(selector);
      if (element instanceof HTMLElement) return element;
    } catch {
      // Invalid selector, continue to XPath
    }

    // Try XPath
    try {
      const result = document.evaluate(
        xpath,
        document,
        null,
        XPathResult.FIRST_ORDERED_NODE_TYPE,
        null
      );
      if (result.singleNodeValue instanceof HTMLElement) {
        return result.singleNodeValue;
      }
    } catch {
      // XPath failed
    }

    return null;
  }

  private updatePositions(): void {
    this.overlays.forEach((overlay, issueId) => {
      const selector = overlay.dataset.selector;
      if (!selector) return;

      const element = document.querySelector(selector);
      if (element) {
        const rect = element.getBoundingClientRect();
        overlay.style.left = `${rect.left + window.scrollX}px`;
        overlay.style.top = `${rect.top + window.scrollY}px`;
        overlay.style.width = `${rect.width}px`;
        overlay.style.height = `${rect.height}px`;
      }
    });
  }

  private reapplyStyles(): void {
    this.overlays.forEach((overlay, issueId) => {
      const severity = overlay.dataset.severity as SeverityLevel;
      const style = this.getStyleForSeverity(severity);

      overlay.style.borderColor = style.color;
      overlay.style.borderWidth = `${style.borderWidth}px`;
      overlay.style.backgroundColor = style.backgroundColor;
      overlay.style.opacity = style.opacity.toString();
    });
  }

  private getStyleForSeverity(severity: SeverityLevel): HighlightStyle {
    return this.config[severity];
  }

  private getSeverityColor(severity: SeverityLevel): string {
    const colors = {
      critical: '#DC2626',
      serious: '#F59E0B',
      moderate: '#3B82F6',
      minor: '#10B981',
    };
    return colors[severity];
  }

  private escapeHtml(text: string): string {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }

  private getDefaultConfig(): HighlightConfig {
    return {
      critical: {
        color: '#DC2626',
        borderWidth: 3,
        borderStyle: 'solid',
        backgroundColor: 'rgba(220, 38, 38, 0.1)',
        opacity: 0.6,
        zIndex: 2147483645,
      },
      serious: {
        color: '#F59E0B',
        borderWidth: 2,
        borderStyle: 'solid',
        backgroundColor: 'rgba(245, 158, 11, 0.1)',
        opacity: 0.5,
        zIndex: 2147483644,
      },
      moderate: {
        color: '#3B82F6',
        borderWidth: 2,
        borderStyle: 'dashed',
        backgroundColor: 'rgba(59, 130, 246, 0.05)',
        opacity: 0.4,
        zIndex: 2147483643,
      },
      minor: {
        color: '#10B981',
        borderWidth: 1,
        borderStyle: 'dotted',
        backgroundColor: 'rgba(16, 185, 129, 0.05)',
        opacity: 0.3,
        zIndex: 2147483642,
      },
    };
  }
}

// ============================================================================
// Export singleton instance
// ============================================================================

export const highlighter = new IssueHighlighter();
