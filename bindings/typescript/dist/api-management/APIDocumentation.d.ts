import React from 'react';
import { OpenAPISpec, APIEndpoint } from './types';
interface APIDocumentationProps {
    spec?: OpenAPISpec;
    endpoints?: APIEndpoint[];
    onTryEndpoint?: (endpoint: APIEndpoint) => void;
    showTryItOut?: boolean;
}
export declare const APIDocumentation: React.FC<APIDocumentationProps>;
export default APIDocumentation;
//# sourceMappingURL=APIDocumentation.d.ts.map