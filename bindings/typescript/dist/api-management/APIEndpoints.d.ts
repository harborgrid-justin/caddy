import React from 'react';
import { APIEndpoint } from './types';
interface APIEndpointsProps {
    projectId?: string;
    onEndpointCreate?: (endpoint: Partial<APIEndpoint>) => Promise<void>;
    onEndpointUpdate?: (id: string, endpoint: Partial<APIEndpoint>) => Promise<void>;
    onEndpointDelete?: (id: string) => Promise<void>;
}
export declare const APIEndpoints: React.FC<APIEndpointsProps>;
export default APIEndpoints;
//# sourceMappingURL=APIEndpoints.d.ts.map