import React from 'react';
import { SSOProvider } from './types';
interface SSOConfigurationProps {
    onProviderCreate?: (provider: SSOProvider) => void;
    onProviderUpdate?: (provider: SSOProvider) => void;
    onProviderDelete?: (providerId: string) => void;
    className?: string;
}
export declare const SSOConfiguration: React.FC<SSOConfigurationProps>;
export default SSOConfiguration;
//# sourceMappingURL=SSOConfiguration.d.ts.map