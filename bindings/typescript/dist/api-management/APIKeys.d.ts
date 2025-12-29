import React from 'react';
import { APIKey } from './types';
interface APIKeysProps {
    userId?: string;
    onKeyCreate?: (name: string, scopes: string[]) => Promise<{
        key: APIKey;
        secret: string;
    }>;
    onKeyRevoke?: (keyId: string) => Promise<void>;
    onKeyRotate?: (keyId: string) => Promise<{
        key: APIKey;
        secret: string;
    }>;
}
export declare const APIKeys: React.FC<APIKeysProps>;
export default APIKeys;
//# sourceMappingURL=APIKeys.d.ts.map