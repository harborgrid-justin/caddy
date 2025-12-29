/**
 * AI Assistant - Conversational accessibility help interface
 *
 * Provides an intelligent chat interface for accessibility assistance with:
 * - Context-aware suggestions
 * - Natural language queries
 * - Real-time fix recommendations
 * - Learning from user interactions
 */

import React, { useState, useRef, useEffect, useCallback } from 'react';
import type {
  AIAssistantProps,
  ChatMessage,
  AIContext,
  AssistantConfig,
  AutoFix,
  AsyncState,
} from './types';

const DEFAULT_CONFIG: AssistantConfig = {
  enableAutoSuggestions: true,
  enableContextAwareness: true,
  maxHistoryLength: 50,
  suggestionDelay: 500,
};

export function AIAssistant({
  onSuggestionApply,
  onClose,
  initialContext,
  config = DEFAULT_CONFIG,
}: AIAssistantProps) {
  const [messages, setMessages] = useState<ChatMessage[]>([
    {
      id: '1',
      role: 'assistant',
      content: 'Hello! I\'m your AI accessibility assistant. I can help you identify and fix accessibility issues, answer questions about WCAG compliance, and provide suggestions for improvements. How can I help you today?',
      timestamp: new Date().toISOString(),
    },
  ]);
  const [inputValue, setInputValue] = useState('');
  const [isTyping, setIsTyping] = useState(false);
  const [context, setContext] = useState<AIContext>(
    initialContext || {
      recentIssues: [],
      conversationHistory: [],
    }
  );

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const suggestionTimeoutRef = useRef<NodeJS.Timeout>();

  // Auto-scroll to bottom when new messages arrive
  const scrollToBottom = useCallback(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, []);

  useEffect(() => {
    scrollToBottom();
  }, [messages, scrollToBottom]);

  // Focus input on mount
  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // Handle message submission
  const handleSendMessage = useCallback(async () => {
    if (!inputValue.trim()) return;

    const userMessage: ChatMessage = {
      id: Date.now().toString(),
      role: 'user',
      content: inputValue.trim(),
      timestamp: new Date().toISOString(),
    };

    setMessages((prev) => [...prev, userMessage]);
    setInputValue('');
    setIsTyping(true);

    // Update context
    setContext((prev) => ({
      ...prev,
      conversationHistory: [...prev.conversationHistory, userMessage],
    }));

    try {
      // Simulate AI response (in production, this would call the backend)
      const response = await getAIResponse(userMessage.content, context);

      const assistantMessage: ChatMessage = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: response.content,
        timestamp: new Date().toISOString(),
        suggestions: response.suggestions,
        metadata: response.metadata,
      };

      setMessages((prev) => [...prev, assistantMessage]);
      setContext((prev) => ({
        ...prev,
        conversationHistory: [...prev.conversationHistory, assistantMessage],
      }));
    } catch (error) {
      const errorMessage: ChatMessage = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: 'I apologize, but I encountered an error processing your request. Please try again.',
        timestamp: new Date().toISOString(),
      };
      setMessages((prev) => [...prev, errorMessage]);
    } finally {
      setIsTyping(false);
    }
  }, [inputValue, context]);

  // Handle keyboard shortcuts
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        handleSendMessage();
      }
    },
    [handleSendMessage]
  );

  // Handle suggestion application
  const handleApplySuggestion = useCallback(
    (suggestion: AutoFix) => {
      if (onSuggestionApply) {
        onSuggestionApply(suggestion);
      }

      // Add confirmation message
      const confirmationMessage: ChatMessage = {
        id: Date.now().toString(),
        role: 'assistant',
        content: `Great! I've applied the fix for "${suggestion.fixType}". The change has been made to your code.`,
        timestamp: new Date().toISOString(),
      };

      setMessages((prev) => [...prev, confirmationMessage]);
    },
    [onSuggestionApply]
  );

  // Quick action buttons
  const quickActions = [
    { label: 'Check accessibility', query: 'Check my page for accessibility issues' },
    { label: 'Explain WCAG', query: 'Explain WCAG compliance levels' },
    { label: 'Fix alt text', query: 'Help me fix missing alt text' },
    { label: 'Color contrast', query: 'Check color contrast issues' },
  ];

  const handleQuickAction = useCallback((query: string) => {
    setInputValue(query);
    inputRef.current?.focus();
  }, []);

  return (
    <div className="ai-assistant">
      {/* Header */}
      <div className="ai-assistant-header">
        <div className="header-content">
          <div className="assistant-avatar">AI</div>
          <div className="header-text">
            <h2>Accessibility Assistant</h2>
            <p className="status">
              {isTyping ? (
                <span className="typing-indicator">Thinking...</span>
              ) : (
                <span className="online-indicator">Online</span>
              )}
            </p>
          </div>
        </div>
        {onClose && (
          <button
            className="close-button"
            onClick={onClose}
            aria-label="Close assistant"
          >
            ×
          </button>
        )}
      </div>

      {/* Messages Area */}
      <div className="ai-assistant-messages">
        {messages.map((message) => (
          <MessageBubble
            key={message.id}
            message={message}
            onApplySuggestion={handleApplySuggestion}
          />
        ))}

        {isTyping && (
          <div className="message assistant-message">
            <div className="message-avatar">AI</div>
            <div className="message-content">
              <div className="typing-indicator">
                <span></span>
                <span></span>
                <span></span>
              </div>
            </div>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* Quick Actions */}
      {messages.length <= 2 && (
        <div className="quick-actions">
          <p className="quick-actions-label">Quick actions:</p>
          <div className="quick-actions-grid">
            {quickActions.map((action) => (
              <button
                key={action.label}
                className="quick-action-button"
                onClick={() => handleQuickAction(action.query)}
              >
                {action.label}
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Input Area */}
      <div className="ai-assistant-input">
        <div className="input-container">
          <input
            ref={inputRef}
            type="text"
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Ask me anything about accessibility..."
            className="message-input"
            aria-label="Message input"
            disabled={isTyping}
          />
          <button
            onClick={handleSendMessage}
            disabled={!inputValue.trim() || isTyping}
            className="send-button"
            aria-label="Send message"
          >
            <svg width="20" height="20" viewBox="0 0 20 20" fill="currentColor">
              <path d="M2 10l16-8-8 16-2-8-6-0z" />
            </svg>
          </button>
        </div>
        <p className="input-hint">
          Press Enter to send, Shift+Enter for new line
        </p>
      </div>

      {/* Context Indicator */}
      {config.enableContextAwareness && context.currentPage && (
        <div className="context-indicator">
          <span className="context-icon">📄</span>
          <span className="context-text">
            Analyzing: {context.currentPage}
          </span>
        </div>
      )}
    </div>
  );
}

// Message Bubble Component
interface MessageBubbleProps {
  message: ChatMessage;
  onApplySuggestion?: (suggestion: AutoFix) => void;
}

function MessageBubble({ message, onApplySuggestion }: MessageBubbleProps) {
  const isUser = message.role === 'user';

  return (
    <div className={`message ${isUser ? 'user-message' : 'assistant-message'}`}>
      {!isUser && <div className="message-avatar">AI</div>}
      <div className="message-content">
        <div className="message-text">{message.content}</div>
        {message.suggestions && message.suggestions.length > 0 && (
          <div className="message-suggestions">
            <p className="suggestions-label">Suggested fixes:</p>
            {message.suggestions.map((suggestion) => (
              <SuggestionCard
                key={suggestion.issueId}
                suggestion={suggestion}
                onApply={onApplySuggestion}
              />
            ))}
          </div>
        )}
        <div className="message-timestamp">
          {new Date(message.timestamp).toLocaleTimeString()}
        </div>
      </div>
      {isUser && <div className="message-avatar user-avatar">You</div>}
    </div>
  );
}

// Suggestion Card Component
interface SuggestionCardProps {
  suggestion: AutoFix;
  onApply?: (suggestion: AutoFix) => void;
}

function SuggestionCard({ suggestion, onApply }: SuggestionCardProps) {
  const [isExpanded, setIsExpanded] = useState(false);

  const confidenceColor =
    suggestion.confidence === 'High'
      ? 'green'
      : suggestion.confidence === 'Medium'
      ? 'orange'
      : 'red';

  return (
    <div className="suggestion-card">
      <div className="suggestion-header">
        <h4 className="suggestion-title">{suggestion.fixType}</h4>
        <span className={`confidence-badge confidence-${confidenceColor}`}>
          {suggestion.confidence} confidence
        </span>
      </div>

      <p className="suggestion-explanation">{suggestion.explanation}</p>

      {isExpanded && (
        <div className="suggestion-details">
          <div className="code-diff">
            <div className="code-section">
              <span className="code-label">Before:</span>
              <pre className="code-block original">{suggestion.originalCode}</pre>
            </div>
            <div className="code-section">
              <span className="code-label">After:</span>
              <pre className="code-block fixed">{suggestion.fixedCode}</pre>
            </div>
          </div>

          {suggestion.wcagCriteria.length > 0 && (
            <div className="wcag-info">
              <span className="wcag-label">WCAG Criteria:</span>
              <span className="wcag-criteria">
                {suggestion.wcagCriteria.join(', ')}
              </span>
            </div>
          )}
        </div>
      )}

      <div className="suggestion-actions">
        <button
          className="toggle-details-button"
          onClick={() => setIsExpanded(!isExpanded)}
        >
          {isExpanded ? 'Hide details' : 'Show details'}
        </button>
        {onApply && (
          <button
            className="apply-button"
            onClick={() => onApply(suggestion)}
          >
            Apply fix
          </button>
        )}
      </div>

      {suggestion.requiresManualReview && (
        <div className="warning-badge">
          ⚠️ Manual review recommended
        </div>
      )}
    </div>
  );
}

// AI Response Generator (mock - in production would call backend)
async function getAIResponse(
  query: string,
  context: AIContext
): Promise<{
  content: string;
  suggestions?: AutoFix[];
  metadata?: Record<string, any>;
}> {
  // Simulate network delay
  await new Promise((resolve) => setTimeout(resolve, 1000));

  const queryLower = query.toLowerCase();

  // Pattern matching for common queries
  if (queryLower.includes('wcag')) {
    return {
      content:
        'WCAG (Web Content Accessibility Guidelines) has three conformance levels:\n\n' +
        '• Level A: Basic accessibility features\n' +
        '• Level AA: Recommended level for most organizations (removes major barriers)\n' +
        '• Level AAA: Highest level of accessibility\n\n' +
        'Most organizations aim for WCAG 2.1 Level AA compliance.',
    };
  }

  if (queryLower.includes('alt text') || queryLower.includes('image')) {
    return {
      content:
        'I can help you fix missing alt text! Here are some tips:\n\n' +
        '1. Describe what the image shows, not what it is\n' +
        '2. Keep it concise (usually under 125 characters)\n' +
        '3. For decorative images, use alt=""\n' +
        '4. Include relevant context for the page\n\n' +
        'Would you like me to analyze images on your current page?',
      suggestions: [
        {
          issueId: 'img-1',
          fixType: 'missing_alt_text',
          originalCode: '<img src="logo.png">',
          fixedCode: '<img src="logo.png" alt="Company logo">',
          diff: '+ alt="Company logo"',
          confidence: 'High',
          requiresManualReview: false,
          explanation: 'Added descriptive alt text for logo image',
          wcagCriteria: ['1.1.1'],
          applied: false,
          timestamp: new Date().toISOString(),
        },
      ],
    };
  }

  if (queryLower.includes('color') || queryLower.includes('contrast')) {
    return {
      content:
        'Color contrast is important for users with low vision. WCAG requires:\n\n' +
        '• Normal text: 4.5:1 contrast ratio (Level AA)\n' +
        '• Large text: 3:1 contrast ratio (Level AA)\n' +
        '• Enhanced: 7:1 for normal text (Level AAA)\n\n' +
        'I can analyze your page for contrast issues. Would you like me to check?',
    };
  }

  if (queryLower.includes('check') || queryLower.includes('scan')) {
    return {
      content:
        'I\'ll perform a comprehensive accessibility scan. This includes:\n\n' +
        '✓ Missing alt text\n' +
        '✓ Color contrast issues\n' +
        '✓ Heading hierarchy\n' +
        '✓ Form labels\n' +
        '✓ Keyboard accessibility\n' +
        '✓ ARIA attributes\n\n' +
        'Scanning your page now...',
      metadata: {
        action: 'scan',
      },
    };
  }

  // Default response
  return {
    content:
      'I\'m here to help with accessibility! I can:\n\n' +
      '• Identify and fix accessibility issues\n' +
      '• Explain WCAG guidelines\n' +
      '• Provide code suggestions\n' +
      '• Answer accessibility questions\n\n' +
      'What would you like help with?',
  };
}
