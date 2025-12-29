import React from 'react';
import { CompressionStats as ICompressionStats } from './types';
export interface CompressionStatsProps {
    stats?: ICompressionStats | null;
    showDetailed?: boolean;
    refreshInterval?: number;
    className?: string;
}
export declare const CompressionStats: React.FC<CompressionStatsProps>;
export default CompressionStats;
//# sourceMappingURL=CompressionStats.d.ts.map