import React from 'react';
import { APIVersion } from './types';
interface APIVersioningProps {
    onVersionCreate?: (version: Partial<APIVersion>) => Promise<void>;
    onVersionUpdate?: (version: string, update: Partial<APIVersion>) => Promise<void>;
}
export declare const APIVersioning: React.FC<APIVersioningProps>;
export default APIVersioning;
//# sourceMappingURL=APIVersioning.d.ts.map