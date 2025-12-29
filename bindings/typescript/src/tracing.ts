/**
 * Distributed Tracing Client
 *
 * Provides TypeScript bindings for CADDY's distributed tracing system
 * with W3C Trace Context support, multiple exporters, and metrics collection.
 */

import axios, { AxiosInstance } from 'axios';

/**
 * Trace exporter types
 */
export enum TraceExporter {
  /** OpenTelemetry Protocol */
  OTLP = 'otlp',
  /** Jaeger exporter */
  Jaeger = 'jaeger',
  /** Zipkin exporter */
  Zipkin = 'zipkin',
  /** Console logging */
  Console = 'console',
}

/**
 * Span kind
 */
export enum SpanKind {
  Internal = 'INTERNAL',
  Server = 'SERVER',
  Client = 'CLIENT',
  Producer = 'PRODUCER',
  Consumer = 'CONSUMER',
}

/**
 * Span status
 */
export enum SpanStatus {
  Unset = 'UNSET',
  Ok = 'OK',
  Error = 'ERROR',
}

/**
 * Tracing configuration
 */
export interface TracingConfig {
  /** API base URL */
  apiUrl: string;
  /** Authentication token */
  token?: string;
  /** Service name */
  serviceName: string;
  /** Service version */
  serviceVersion?: string;
  /** Sampling rate (0.0 to 1.0) */
  samplingRate?: number;
  /** Trace exporters */
  exporters?: TraceExporter[];
  /** Enable metrics collection */
  enableMetrics?: boolean;
}

/**
 * Span context (W3C Trace Context)
 */
export interface SpanContext {
  /** Trace ID (32 hex chars) */
  traceId: string;
  /** Span ID (16 hex chars) */
  spanId: string;
  /** Parent span ID */
  parentSpanId?: string;
  /** Trace flags */
  traceFlags: number;
  /** Trace state */
  traceState?: string;
}

/**
 * Span attributes
 */
export interface SpanAttributes {
  [key: string]: string | number | boolean | string[] | number[] | boolean[];
}

/**
 * Span event
 */
export interface SpanEvent {
  /** Event name */
  name: string;
  /** Event timestamp */
  timestamp: string;
  /** Event attributes */
  attributes?: SpanAttributes;
}

/**
 * Span data
 */
export interface Span {
  /** Span context */
  context: SpanContext;
  /** Span name */
  name: string;
  /** Span kind */
  kind: SpanKind;
  /** Start time */
  startTime: string;
  /** End time */
  endTime?: string;
  /** Span status */
  status: SpanStatus;
  /** Status message */
  statusMessage?: string;
  /** Span attributes */
  attributes: SpanAttributes;
  /** Span events */
  events: SpanEvent[];
}

/**
 * Distributed tracing client
 */
export class TracingClient {
  private client: AxiosInstance;
  private config: Required<TracingConfig>;
  private activeSpans: Map<string, Span>;

  constructor(config: TracingConfig) {
    this.config = {
      apiUrl: config.apiUrl,
      token: config.token || '',
      serviceName: config.serviceName,
      serviceVersion: config.serviceVersion || '0.0.0',
      samplingRate: config.samplingRate ?? 1.0,
      exporters: config.exporters || [TraceExporter.OTLP],
      enableMetrics: config.enableMetrics ?? true,
    };

    this.client = axios.create({
      baseURL: `${this.config.apiUrl}/api/tracing`,
      headers: {
        'Authorization': `Bearer ${this.config.token}`,
        'Content-Type': 'application/json',
      },
      timeout: 10000,
    });

    this.activeSpans = new Map();
  }

  /**
   * Start a new span
   */
  async startSpan(
    name: string,
    options?: {
      kind?: SpanKind;
      parentContext?: SpanContext;
      attributes?: SpanAttributes;
    }
  ): Promise<Span> {
    const response = await this.client.post<Span>('/spans/start', {
      name,
      serviceName: this.config.serviceName,
      serviceVersion: this.config.serviceVersion,
      kind: options?.kind || SpanKind.Internal,
      parentContext: options?.parentContext,
      attributes: options?.attributes || {},
    });

    const span = response.data;
    this.activeSpans.set(span.context.spanId, span);
    return span;
  }

  /**
   * End a span
   */
  async endSpan(
    spanId: string,
    options?: {
      status?: SpanStatus;
      statusMessage?: string;
      attributes?: SpanAttributes;
    }
  ): Promise<void> {
    await this.client.post(`/spans/${spanId}/end`, {
      status: options?.status || SpanStatus.Ok,
      statusMessage: options?.statusMessage,
      attributes: options?.attributes,
    });

    this.activeSpans.delete(spanId);
  }

  /**
   * Add an event to a span
   */
  async addSpanEvent(
    spanId: string,
    name: string,
    attributes?: SpanAttributes
  ): Promise<void> {
    await this.client.post(`/spans/${spanId}/events`, {
      name,
      attributes: attributes || {},
    });
  }

  /**
   * Set span attributes
   */
  async setSpanAttributes(
    spanId: string,
    attributes: SpanAttributes
  ): Promise<void> {
    await this.client.patch(`/spans/${spanId}/attributes`, { attributes });
  }

  /**
   * Get span by ID
   */
  async getSpan(spanId: string): Promise<Span | null> {
    try {
      const response = await this.client.get<Span>(`/spans/${spanId}`);
      return response.data;
    } catch (error: any) {
      if (error.response?.status === 404) {
        return null;
      }
      throw error;
    }
  }

  /**
   * Get trace by trace ID
   */
  async getTrace(traceId: string): Promise<Span[]> {
    const response = await this.client.get<Span[]>(`/traces/${traceId}`);
    return response.data;
  }

  /**
   * Execute a function within a traced context
   */
  async traced<T>(
    name: string,
    fn: (span: Span) => Promise<T>,
    options?: {
      kind?: SpanKind;
      parentContext?: SpanContext;
      attributes?: SpanAttributes;
    }
  ): Promise<T> {
    const span = await this.startSpan(name, options);

    try {
      const result = await fn(span);
      await this.endSpan(span.context.spanId, { status: SpanStatus.Ok });
      return result;
    } catch (error: any) {
      await this.endSpan(span.context.spanId, {
        status: SpanStatus.Error,
        statusMessage: error.message,
      });
      throw error;
    }
  }

  /**
   * Get active spans
   */
  getActiveSpans(): Span[] {
    return Array.from(this.activeSpans.values());
  }
}
