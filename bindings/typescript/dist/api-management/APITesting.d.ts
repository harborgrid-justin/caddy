import React from 'react';
import { TestResult } from './types';
interface APITestingProps {
    onRunTest?: (testId: string) => Promise<TestResult>;
    onRunSuite?: (suiteId: string) => Promise<TestResult[]>;
}
export declare const APITesting: React.FC<APITestingProps>;
export default APITesting;
//# sourceMappingURL=APITesting.d.ts.map