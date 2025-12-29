import React from 'react';
import { PluginApiContext, PluginEvent } from './types';
export interface PluginHostProps {
    pluginId: string;
    context: PluginApiContext;
    uiUrl?: string;
    className?: string;
    width?: string | number;
    height?: string | number;
    showLoading?: boolean;
    onReady?: () => void;
    onError?: (error: Error) => void;
    onEvent?: (event: PluginEvent) => void;
}
export declare const PluginHost: React.FC<PluginHostProps>;
export interface PluginContainerProps {
    plugins: Array<{
        pluginId: string;
        context: PluginApiContext;
        uiUrl?: string;
    }>;
    layout?: 'grid' | 'tabs' | 'sidebar';
    className?: string;
}
export declare const PluginContainer: React.FC<PluginContainerProps>;
export default PluginHost;
//# sourceMappingURL=PluginHost.d.ts.map