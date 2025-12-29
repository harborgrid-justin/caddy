import React from 'react';
import { RateLimit } from './types';
interface APIRateLimitsProps {
    onRateLimitCreate?: (rateLimit: Partial<RateLimit>) => Promise<void>;
    onRateLimitUpdate?: (id: string, rateLimit: Partial<RateLimit>) => Promise<void>;
    onRateLimitDelete?: (id: string) => Promise<void>;
}
export declare const APIRateLimits: React.FC<APIRateLimitsProps>;
export default APIRateLimits;
//# sourceMappingURL=APIRateLimits.d.ts.map