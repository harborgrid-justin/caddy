export var NotificationPriority;
(function (NotificationPriority) {
    NotificationPriority["LOW"] = "low";
    NotificationPriority["MEDIUM"] = "medium";
    NotificationPriority["HIGH"] = "high";
    NotificationPriority["URGENT"] = "urgent";
    NotificationPriority["CRITICAL"] = "critical";
})(NotificationPriority || (NotificationPriority = {}));
export var NotificationStatus;
(function (NotificationStatus) {
    NotificationStatus["PENDING"] = "pending";
    NotificationStatus["SENT"] = "sent";
    NotificationStatus["DELIVERED"] = "delivered";
    NotificationStatus["READ"] = "read";
    NotificationStatus["FAILED"] = "failed";
    NotificationStatus["ARCHIVED"] = "archived";
})(NotificationStatus || (NotificationStatus = {}));
export var NotificationType;
(function (NotificationType) {
    NotificationType["INFO"] = "info";
    NotificationType["SUCCESS"] = "success";
    NotificationType["WARNING"] = "warning";
    NotificationType["ERROR"] = "error";
    NotificationType["SYSTEM"] = "system";
    NotificationType["TASK"] = "task";
    NotificationType["MENTION"] = "mention";
    NotificationType["COMMENT"] = "comment";
    NotificationType["APPROVAL"] = "approval";
    NotificationType["REMINDER"] = "reminder";
    NotificationType["ALERT"] = "alert";
})(NotificationType || (NotificationType = {}));
export var NotificationChannel;
(function (NotificationChannel) {
    NotificationChannel["IN_APP"] = "in_app";
    NotificationChannel["EMAIL"] = "email";
    NotificationChannel["SMS"] = "sms";
    NotificationChannel["PUSH"] = "push";
    NotificationChannel["SLACK"] = "slack";
    NotificationChannel["TEAMS"] = "teams";
    NotificationChannel["WEBHOOK"] = "webhook";
})(NotificationChannel || (NotificationChannel = {}));
export var NotificationGroupBy;
(function (NotificationGroupBy) {
    NotificationGroupBy["NONE"] = "none";
    NotificationGroupBy["TYPE"] = "type";
    NotificationGroupBy["SOURCE"] = "source";
    NotificationGroupBy["DATE"] = "date";
    NotificationGroupBy["PRIORITY"] = "priority";
})(NotificationGroupBy || (NotificationGroupBy = {}));
//# sourceMappingURL=types.js.map