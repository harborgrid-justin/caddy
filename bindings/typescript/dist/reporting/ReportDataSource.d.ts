import React from 'react';
import { DataSource, DataSourceSchema } from './types';
export interface ReportDataSourceProps {
    dataSources: DataSource[];
    selectedDataSource?: DataSource;
    onSelect: (dataSource: DataSource) => void;
    onSchemaExplore?: (dataSourceId: string) => Promise<DataSourceSchema>;
    onTestConnection?: (dataSource: DataSource) => Promise<boolean>;
    onCreateDataSource?: () => void;
    readOnly?: boolean;
}
export declare const ReportDataSource: React.FC<ReportDataSourceProps>;
export default ReportDataSource;
//# sourceMappingURL=ReportDataSource.d.ts.map