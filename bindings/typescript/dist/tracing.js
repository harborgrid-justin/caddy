import axios from 'axios';
export var TraceExporter;
(function (TraceExporter) {
    TraceExporter["OTLP"] = "otlp";
    TraceExporter["Jaeger"] = "jaeger";
    TraceExporter["Zipkin"] = "zipkin";
    TraceExporter["Console"] = "console";
})(TraceExporter || (TraceExporter = {}));
export var SpanKind;
(function (SpanKind) {
    SpanKind["Internal"] = "INTERNAL";
    SpanKind["Server"] = "SERVER";
    SpanKind["Client"] = "CLIENT";
    SpanKind["Producer"] = "PRODUCER";
    SpanKind["Consumer"] = "CONSUMER";
})(SpanKind || (SpanKind = {}));
export var SpanStatus;
(function (SpanStatus) {
    SpanStatus["Unset"] = "UNSET";
    SpanStatus["Ok"] = "OK";
    SpanStatus["Error"] = "ERROR";
})(SpanStatus || (SpanStatus = {}));
export class TracingClient {
    constructor(config) {
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
    async startSpan(name, options) {
        const response = await this.client.post('/spans/start', {
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
    async endSpan(spanId, options) {
        await this.client.post(`/spans/${spanId}/end`, {
            status: options?.status || SpanStatus.Ok,
            statusMessage: options?.statusMessage,
            attributes: options?.attributes,
        });
        this.activeSpans.delete(spanId);
    }
    async addSpanEvent(spanId, name, attributes) {
        await this.client.post(`/spans/${spanId}/events`, {
            name,
            attributes: attributes || {},
        });
    }
    async setSpanAttributes(spanId, attributes) {
        await this.client.patch(`/spans/${spanId}/attributes`, { attributes });
    }
    async getSpan(spanId) {
        try {
            const response = await this.client.get(`/spans/${spanId}`);
            return response.data;
        }
        catch (error) {
            if (error.response?.status === 404) {
                return null;
            }
            throw error;
        }
    }
    async getTrace(traceId) {
        const response = await this.client.get(`/traces/${traceId}`);
        return response.data;
    }
    async traced(name, fn, options) {
        const span = await this.startSpan(name, options);
        try {
            const result = await fn(span);
            await this.endSpan(span.context.spanId, { status: SpanStatus.Ok });
            return result;
        }
        catch (error) {
            await this.endSpan(span.context.spanId, {
                status: SpanStatus.Error,
                statusMessage: error.message,
            });
            throw error;
        }
    }
    getActiveSpans() {
        return Array.from(this.activeSpans.values());
    }
}
//# sourceMappingURL=tracing.js.map