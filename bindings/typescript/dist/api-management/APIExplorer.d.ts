import React from 'react';
import { APIEndpoint, APITestRequest, APITestResponse, AuthConfig } from './types';
interface APIExplorerProps {
    endpoints?: APIEndpoint[];
    onExecuteRequest?: (request: APITestRequest) => Promise<APITestResponse>;
    enableCodeGen?: boolean;
    defaultAuth?: AuthConfig;
}
export declare const APIExplorer: React.FC<APIExplorerProps>;
export default APIExplorer;
//# sourceMappingURL=APIExplorer.d.ts.map