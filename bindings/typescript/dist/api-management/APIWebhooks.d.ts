import React from 'react';
import { Webhook } from './types';
interface APIWebhooksProps {
    onWebhookCreate?: (webhook: Partial<Webhook>) => Promise<void>;
    onWebhookUpdate?: (id: string, webhook: Partial<Webhook>) => Promise<void>;
    onWebhookDelete?: (id: string) => Promise<void>;
    onWebhookTest?: (id: string) => Promise<void>;
}
export declare const APIWebhooks: React.FC<APIWebhooksProps>;
export default APIWebhooks;
//# sourceMappingURL=APIWebhooks.d.ts.map