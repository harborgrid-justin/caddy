export var ServiceStatus;
(function (ServiceStatus) {
    ServiceStatus["HEALTHY"] = "healthy";
    ServiceStatus["DEGRADED"] = "degraded";
    ServiceStatus["DOWN"] = "down";
    ServiceStatus["MAINTENANCE"] = "maintenance";
    ServiceStatus["UNKNOWN"] = "unknown";
})(ServiceStatus || (ServiceStatus = {}));
export var AlertSeverity;
(function (AlertSeverity) {
    AlertSeverity["CRITICAL"] = "critical";
    AlertSeverity["HIGH"] = "high";
    AlertSeverity["MEDIUM"] = "medium";
    AlertSeverity["LOW"] = "low";
    AlertSeverity["INFO"] = "info";
})(AlertSeverity || (AlertSeverity = {}));
export var AlertState;
(function (AlertState) {
    AlertState["ACTIVE"] = "active";
    AlertState["ACKNOWLEDGED"] = "acknowledged";
    AlertState["RESOLVED"] = "resolved";
    AlertState["SILENCED"] = "silenced";
})(AlertState || (AlertState = {}));
export var IncidentStatus;
(function (IncidentStatus) {
    IncidentStatus["INVESTIGATING"] = "investigating";
    IncidentStatus["IDENTIFIED"] = "identified";
    IncidentStatus["MONITORING"] = "monitoring";
    IncidentStatus["RESOLVED"] = "resolved";
})(IncidentStatus || (IncidentStatus = {}));
export var MetricType;
(function (MetricType) {
    MetricType["CPU"] = "cpu";
    MetricType["MEMORY"] = "memory";
    MetricType["DISK"] = "disk";
    MetricType["NETWORK"] = "network";
    MetricType["LATENCY"] = "latency";
    MetricType["THROUGHPUT"] = "throughput";
    MetricType["ERROR_RATE"] = "error_rate";
    MetricType["CUSTOM"] = "custom";
})(MetricType || (MetricType = {}));
//# sourceMappingURL=types.js.map