import React from 'react';
import type { ActivityFeedItem } from './types';
export interface RealtimeFeedProps {
    wsUrl: string;
    channel?: string;
    initialItems?: ActivityFeedItem[];
    maxItems?: number;
    enableNotifications?: boolean;
    enableSound?: boolean;
    showFilters?: boolean;
    showSearch?: boolean;
    autoScroll?: boolean;
    onItemClick?: (item: ActivityFeedItem) => void;
    className?: string;
}
export declare const RealtimeFeed: React.FC<RealtimeFeedProps>;
export default RealtimeFeed;
//# sourceMappingURL=RealtimeFeed.d.ts.map