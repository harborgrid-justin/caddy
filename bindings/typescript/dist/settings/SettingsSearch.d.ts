import React from 'react';
interface SettingsSearchProps {
    query: string;
    onQueryChange: (query: string) => void;
    onClose: () => void;
    onNavigate: (path: string) => void;
}
declare const SettingsSearch: React.FC<SettingsSearchProps>;
export default SettingsSearch;
//# sourceMappingURL=SettingsSearch.d.ts.map