import React from 'react';
import { MockServer } from './types';
interface APIMockingProps {
    onServerCreate?: (server: Partial<MockServer>) => Promise<void>;
    onServerUpdate?: (id: string, server: Partial<MockServer>) => Promise<void>;
    onServerDelete?: (id: string) => Promise<void>;
}
export declare const APIMocking: React.FC<APIMockingProps>;
export default APIMocking;
//# sourceMappingURL=APIMocking.d.ts.map