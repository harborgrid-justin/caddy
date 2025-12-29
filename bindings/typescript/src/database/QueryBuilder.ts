/**
 * CADDY Database - Type-Safe Query Builder
 *
 * Provides a fluent, type-safe API for building database queries.
 */

import { QueryFilter, QueryOptions, QueryResult } from './types';

/**
 * Type-safe query builder
 */
export class QueryBuilder<T = any> {
  private tableName: string;
  private filters: QueryFilter[] = [];
  private orderByFields: Array<{ field: string; direction: 'asc' | 'desc' }> = [];
  private limitValue?: number;
  private offsetValue?: number;
  private selectFields?: string[];
  private useCacheFlag: boolean = true;
  private cacheTtlValue?: number;

  constructor(table: string) {
    this.tableName = table;
  }

  /**
   * Add a WHERE filter
   */
  where(field: string, operator: QueryFilter['operator'], value: any): this {
    this.filters.push({ field, operator, value });
    return this;
  }

  /**
   * Add an equality filter
   */
  whereEquals(field: string, value: any): this {
    return this.where(field, 'eq', value);
  }

  /**
   * Add a NOT EQUAL filter
   */
  whereNotEquals(field: string, value: any): this {
    return this.where(field, 'ne', value);
  }

  /**
   * Add a GREATER THAN filter
   */
  whereGreaterThan(field: string, value: number): this {
    return this.where(field, 'gt', value);
  }

  /**
   * Add a GREATER THAN OR EQUAL filter
   */
  whereGreaterThanOrEqual(field: string, value: number): this {
    return this.where(field, 'gte', value);
  }

  /**
   * Add a LESS THAN filter
   */
  whereLessThan(field: string, value: number): this {
    return this.where(field, 'lt', value);
  }

  /**
   * Add a LESS THAN OR EQUAL filter
   */
  whereLessThanOrEqual(field: string, value: number): this {
    return this.where(field, 'lte', value);
  }

  /**
   * Add an IN filter
   */
  whereIn(field: string, values: any[]): this {
    return this.where(field, 'in', values);
  }

  /**
   * Add a LIKE filter
   */
  whereLike(field: string, pattern: string): this {
    return this.where(field, 'like', pattern);
  }

  /**
   * Add a BETWEEN filter
   */
  whereBetween(field: string, min: number, max: number): this {
    return this.where(field, 'between', [min, max]);
  }

  /**
   * Set ORDER BY clause
   */
  orderBy(field: string, direction: 'asc' | 'desc' = 'asc'): this {
    this.orderByFields.push({ field, direction });
    return this;
  }

  /**
   * Set LIMIT clause
   */
  limit(count: number): this {
    this.limitValue = count;
    return this;
  }

  /**
   * Set OFFSET clause
   */
  offset(count: number): this {
    this.offsetValue = count;
    return this;
  }

  /**
   * Set SELECT fields
   */
  select(...fields: string[]): this {
    this.selectFields = fields;
    return this;
  }

  /**
   * Enable or disable caching for this query
   */
  useCache(enabled: boolean): this {
    this.useCacheFlag = enabled;
    return this;
  }

  /**
   * Set cache TTL in milliseconds
   */
  cacheTtl(ttl: number): this {
    this.cacheTtlValue = ttl;
    return this;
  }

  /**
   * Build the query options object
   */
  build(): QueryOptions {
    return {
      where: this.filters.length > 0 ? this.filters : undefined,
      orderBy: this.orderByFields.length > 0 ? this.orderByFields : undefined,
      limit: this.limitValue,
      offset: this.offsetValue,
      select: this.selectFields,
      useCache: this.useCacheFlag,
      cacheTtl: this.cacheTtlValue,
    };
  }

  /**
   * Generate SQL query string (for debugging)
   */
  toSQL(): string {
    let sql = 'SELECT ';

    // SELECT clause
    if (this.selectFields && this.selectFields.length > 0) {
      sql += this.selectFields.join(', ');
    } else {
      sql += '*';
    }

    // FROM clause
    sql += ` FROM ${this.tableName}`;

    // WHERE clause
    if (this.filters.length > 0) {
      const whereClauses = this.filters.map((filter) => {
        switch (filter.operator) {
          case 'eq':
            return `${filter.field} = ${this.formatValue(filter.value)}`;
          case 'ne':
            return `${filter.field} != ${this.formatValue(filter.value)}`;
          case 'gt':
            return `${filter.field} > ${this.formatValue(filter.value)}`;
          case 'gte':
            return `${filter.field} >= ${this.formatValue(filter.value)}`;
          case 'lt':
            return `${filter.field} < ${this.formatValue(filter.value)}`;
          case 'lte':
            return `${filter.field} <= ${this.formatValue(filter.value)}`;
          case 'in':
            return `${filter.field} IN (${filter.value.map(this.formatValue).join(', ')})`;
          case 'like':
            return `${filter.field} LIKE ${this.formatValue(filter.value)}`;
          case 'between':
            return `${filter.field} BETWEEN ${this.formatValue(filter.value[0])} AND ${this.formatValue(filter.value[1])}`;
          default:
            return '';
        }
      });
      sql += ' WHERE ' + whereClauses.join(' AND ');
    }

    // ORDER BY clause
    if (this.orderByFields.length > 0) {
      const orderClauses = this.orderByFields.map(
        (field) => `${field.field} ${field.direction.toUpperCase()}`
      );
      sql += ' ORDER BY ' + orderClauses.join(', ');
    }

    // LIMIT clause
    if (this.limitValue !== undefined) {
      sql += ` LIMIT ${this.limitValue}`;
    }

    // OFFSET clause
    if (this.offsetValue !== undefined) {
      sql += ` OFFSET ${this.offsetValue}`;
    }

    return sql;
  }

  /**
   * Format a value for SQL
   */
  private formatValue(value: any): string {
    if (typeof value === 'string') {
      return `'${value.replace(/'/g, "''")}'`;
    }
    if (value === null) {
      return 'NULL';
    }
    if (typeof value === 'boolean') {
      return value ? '1' : '0';
    }
    return String(value);
  }

  /**
   * Clone this query builder
   */
  clone(): QueryBuilder<T> {
    const cloned = new QueryBuilder<T>(this.tableName);
    cloned.filters = [...this.filters];
    cloned.orderByFields = [...this.orderByFields];
    cloned.limitValue = this.limitValue;
    cloned.offsetValue = this.offsetValue;
    cloned.selectFields = this.selectFields ? [...this.selectFields] : undefined;
    cloned.useCacheFlag = this.useCacheFlag;
    cloned.cacheTtlValue = this.cacheTtlValue;
    return cloned;
  }

  /**
   * Reset the query builder
   */
  reset(): this {
    this.filters = [];
    this.orderByFields = [];
    this.limitValue = undefined;
    this.offsetValue = undefined;
    this.selectFields = undefined;
    this.useCacheFlag = true;
    this.cacheTtlValue = undefined;
    return this;
  }

  /**
   * Get the table name
   */
  getTable(): string {
    return this.tableName;
  }
}

/**
 * Factory function for creating query builders
 */
export function query<T = any>(table: string): QueryBuilder<T> {
  return new QueryBuilder<T>(table);
}

/**
 * Aggregate query builder
 */
export class AggregateQueryBuilder<T = any> extends QueryBuilder<T> {
  private aggregations: Array<{ function: string; field: string; alias?: string }> = [];
  private groupByFields: string[] = [];
  private havingFilters: QueryFilter[] = [];

  /**
   * Add a COUNT aggregation
   */
  count(field: string = '*', alias?: string): this {
    this.aggregations.push({ function: 'COUNT', field, alias });
    return this;
  }

  /**
   * Add a SUM aggregation
   */
  sum(field: string, alias?: string): this {
    this.aggregations.push({ function: 'SUM', field, alias });
    return this;
  }

  /**
   * Add an AVG aggregation
   */
  avg(field: string, alias?: string): this {
    this.aggregations.push({ function: 'AVG', field, alias });
    return this;
  }

  /**
   * Add a MIN aggregation
   */
  min(field: string, alias?: string): this {
    this.aggregations.push({ function: 'MIN', field, alias });
    return this;
  }

  /**
   * Add a MAX aggregation
   */
  max(field: string, alias?: string): this {
    this.aggregations.push({ function: 'MAX', field, alias });
    return this;
  }

  /**
   * Add a GROUP BY clause
   */
  groupBy(...fields: string[]): this {
    this.groupByFields.push(...fields);
    return this;
  }

  /**
   * Add a HAVING filter
   */
  having(field: string, operator: QueryFilter['operator'], value: any): this {
    this.havingFilters.push({ field, operator, value });
    return this;
  }

  /**
   * Generate SQL query string for aggregate queries
   */
  toSQL(): string {
    let sql = 'SELECT ';

    // Aggregations
    const aggClauses = this.aggregations.map((agg) => {
      const clause = `${agg.function}(${agg.field})`;
      return agg.alias ? `${clause} AS ${agg.alias}` : clause;
    });

    // GROUP BY fields
    if (this.groupByFields.length > 0) {
      sql += [...this.groupByFields, ...aggClauses].join(', ');
    } else {
      sql += aggClauses.join(', ');
    }

    sql += ` FROM ${this.getTable()}`;

    // Use parent's WHERE clause logic
    const parentSQL = super.toSQL();
    const whereMatch = parentSQL.match(/WHERE (.+?)(?=ORDER BY|LIMIT|OFFSET|$)/);
    if (whereMatch) {
      sql += ` WHERE ${whereMatch[1]}`;
    }

    // GROUP BY clause
    if (this.groupByFields.length > 0) {
      sql += ` GROUP BY ${this.groupByFields.join(', ')}`;
    }

    // HAVING clause
    if (this.havingFilters.length > 0) {
      const havingClauses = this.havingFilters.map((filter) => {
        // Reuse formatValue logic from parent
        return `${filter.field} ${filter.operator} ${filter.value}`;
      });
      sql += ' HAVING ' + havingClauses.join(' AND ');
    }

    return sql;
  }
}

/**
 * Factory function for creating aggregate query builders
 */
export function aggregate<T = any>(table: string): AggregateQueryBuilder<T> {
  return new AggregateQueryBuilder<T>(table);
}
