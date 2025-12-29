export declare enum TraceExporter {
    OTLP = "otlp",
    Jaeger = "jaeger",
    Zipkin = "zipkin",
    Console = "console"
}
export declare enum SpanKind {
    Internal = "INTERNAL",
    Server = "SERVER",
    Client = "CLIENT",
    Producer = "PRODUCER",
    Consumer = "CONSUMER"
}
export declare enum SpanStatus {
    Unset = "UNSET",
    Ok = "OK",
    Error = "ERROR"
}
export interface TracingConfig {
    apiUrl: string;
    token?: string;
    serviceName: string;
    serviceVersion?: string;
    samplingRate?: number;
    exporters?: TraceExporter[];
    enableMetrics?: boolean;
}
export interface SpanContext {
    traceId: string;
    spanId: string;
    parentSpanId?: string;
    traceFlags: number;
    traceState?: string;
}
export interface SpanAttributes {
    [key: string]: string | number | boolean | string[] | number[] | boolean[];
}
export interface SpanEvent {
    name: string;
    timestamp: string;
    attributes?: SpanAttributes;
}
export interface Span {
    context: SpanContext;
    name: string;
    kind: SpanKind;
    startTime: string;
    endTime?: string;
    status: SpanStatus;
    statusMessage?: string;
    attributes: SpanAttributes;
    events: SpanEvent[];
}
export declare class TracingClient {
    private client;
    private config;
    private activeSpans;
    constructor(config: TracingConfig);
    startSpan(name: string, options?: {
        kind?: SpanKind;
        parentContext?: SpanContext;
        attributes?: SpanAttributes;
    }): Promise<Span>;
    endSpan(spanId: string, options?: {
        status?: SpanStatus;
        statusMessage?: string;
        attributes?: SpanAttributes;
    }): Promise<void>;
    addSpanEvent(spanId: string, name: string, attributes?: SpanAttributes): Promise<void>;
    setSpanAttributes(spanId: string, attributes: SpanAttributes): Promise<void>;
    getSpan(spanId: string): Promise<Span | null>;
    getTrace(traceId: string): Promise<Span[]>;
    traced<T>(name: string, fn: (span: Span) => Promise<T>, options?: {
        kind?: SpanKind;
        parentContext?: SpanContext;
        attributes?: SpanAttributes;
    }): Promise<T>;
    getActiveSpans(): Span[];
}
//# sourceMappingURL=tracing.d.ts.map