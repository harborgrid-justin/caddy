import { QueryFilter, QueryOptions } from './types';
export declare class QueryBuilder<T = any> {
    private tableName;
    private filters;
    private orderByFields;
    private limitValue?;
    private offsetValue?;
    private selectFields?;
    private useCacheFlag;
    private cacheTtlValue?;
    constructor(table: string);
    where(field: string, operator: QueryFilter['operator'], value: any): this;
    whereEquals(field: string, value: any): this;
    whereNotEquals(field: string, value: any): this;
    whereGreaterThan(field: string, value: number): this;
    whereGreaterThanOrEqual(field: string, value: number): this;
    whereLessThan(field: string, value: number): this;
    whereLessThanOrEqual(field: string, value: number): this;
    whereIn(field: string, values: any[]): this;
    whereLike(field: string, pattern: string): this;
    whereBetween(field: string, min: number, max: number): this;
    orderBy(field: string, direction?: 'asc' | 'desc'): this;
    limit(count: number): this;
    offset(count: number): this;
    select(...fields: string[]): this;
    useCache(enabled: boolean): this;
    cacheTtl(ttl: number): this;
    build(): QueryOptions;
    toSQL(): string;
    private formatValue;
    clone(): QueryBuilder<T>;
    reset(): this;
    getTable(): string;
}
export declare function query<T = any>(table: string): QueryBuilder<T>;
export declare class AggregateQueryBuilder<T = any> extends QueryBuilder<T> {
    private aggregations;
    private groupByFields;
    private havingFilters;
    count(field?: string, alias?: string): this;
    sum(field: string, alias?: string): this;
    avg(field: string, alias?: string): this;
    min(field: string, alias?: string): this;
    max(field: string, alias?: string): this;
    groupBy(...fields: string[]): this;
    having(field: string, operator: QueryFilter['operator'], value: any): this;
    toSQL(): string;
}
export declare function aggregate<T = any>(table: string): AggregateQueryBuilder<T>;
//# sourceMappingURL=QueryBuilder.d.ts.map