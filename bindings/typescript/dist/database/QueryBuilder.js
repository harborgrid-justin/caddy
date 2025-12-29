export class QueryBuilder {
    constructor(table) {
        this.filters = [];
        this.orderByFields = [];
        this.useCacheFlag = true;
        this.tableName = table;
    }
    where(field, operator, value) {
        this.filters.push({ field, operator, value });
        return this;
    }
    whereEquals(field, value) {
        return this.where(field, 'eq', value);
    }
    whereNotEquals(field, value) {
        return this.where(field, 'ne', value);
    }
    whereGreaterThan(field, value) {
        return this.where(field, 'gt', value);
    }
    whereGreaterThanOrEqual(field, value) {
        return this.where(field, 'gte', value);
    }
    whereLessThan(field, value) {
        return this.where(field, 'lt', value);
    }
    whereLessThanOrEqual(field, value) {
        return this.where(field, 'lte', value);
    }
    whereIn(field, values) {
        return this.where(field, 'in', values);
    }
    whereLike(field, pattern) {
        return this.where(field, 'like', pattern);
    }
    whereBetween(field, min, max) {
        return this.where(field, 'between', [min, max]);
    }
    orderBy(field, direction = 'asc') {
        this.orderByFields.push({ field, direction });
        return this;
    }
    limit(count) {
        this.limitValue = count;
        return this;
    }
    offset(count) {
        this.offsetValue = count;
        return this;
    }
    select(...fields) {
        this.selectFields = fields;
        return this;
    }
    useCache(enabled) {
        this.useCacheFlag = enabled;
        return this;
    }
    cacheTtl(ttl) {
        this.cacheTtlValue = ttl;
        return this;
    }
    build() {
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
    toSQL() {
        let sql = 'SELECT ';
        if (this.selectFields && this.selectFields.length > 0) {
            sql += this.selectFields.join(', ');
        }
        else {
            sql += '*';
        }
        sql += ` FROM ${this.tableName}`;
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
        if (this.orderByFields.length > 0) {
            const orderClauses = this.orderByFields.map((field) => `${field.field} ${field.direction.toUpperCase()}`);
            sql += ' ORDER BY ' + orderClauses.join(', ');
        }
        if (this.limitValue !== undefined) {
            sql += ` LIMIT ${this.limitValue}`;
        }
        if (this.offsetValue !== undefined) {
            sql += ` OFFSET ${this.offsetValue}`;
        }
        return sql;
    }
    formatValue(value) {
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
    clone() {
        const cloned = new QueryBuilder(this.tableName);
        cloned.filters = [...this.filters];
        cloned.orderByFields = [...this.orderByFields];
        cloned.limitValue = this.limitValue;
        cloned.offsetValue = this.offsetValue;
        cloned.selectFields = this.selectFields ? [...this.selectFields] : undefined;
        cloned.useCacheFlag = this.useCacheFlag;
        cloned.cacheTtlValue = this.cacheTtlValue;
        return cloned;
    }
    reset() {
        this.filters = [];
        this.orderByFields = [];
        this.limitValue = undefined;
        this.offsetValue = undefined;
        this.selectFields = undefined;
        this.useCacheFlag = true;
        this.cacheTtlValue = undefined;
        return this;
    }
    getTable() {
        return this.tableName;
    }
}
export function query(table) {
    return new QueryBuilder(table);
}
export class AggregateQueryBuilder extends QueryBuilder {
    constructor() {
        super(...arguments);
        this.aggregations = [];
        this.groupByFields = [];
        this.havingFilters = [];
    }
    count(field = '*', alias) {
        this.aggregations.push({ function: 'COUNT', field, alias });
        return this;
    }
    sum(field, alias) {
        this.aggregations.push({ function: 'SUM', field, alias });
        return this;
    }
    avg(field, alias) {
        this.aggregations.push({ function: 'AVG', field, alias });
        return this;
    }
    min(field, alias) {
        this.aggregations.push({ function: 'MIN', field, alias });
        return this;
    }
    max(field, alias) {
        this.aggregations.push({ function: 'MAX', field, alias });
        return this;
    }
    groupBy(...fields) {
        this.groupByFields.push(...fields);
        return this;
    }
    having(field, operator, value) {
        this.havingFilters.push({ field, operator, value });
        return this;
    }
    toSQL() {
        let sql = 'SELECT ';
        const aggClauses = this.aggregations.map((agg) => {
            const clause = `${agg.function}(${agg.field})`;
            return agg.alias ? `${clause} AS ${agg.alias}` : clause;
        });
        if (this.groupByFields.length > 0) {
            sql += [...this.groupByFields, ...aggClauses].join(', ');
        }
        else {
            sql += aggClauses.join(', ');
        }
        sql += ` FROM ${this.getTable()}`;
        const parentSQL = super.toSQL();
        const whereMatch = parentSQL.match(/WHERE (.+?)(?=ORDER BY|LIMIT|OFFSET|$)/);
        if (whereMatch) {
            sql += ` WHERE ${whereMatch[1]}`;
        }
        if (this.groupByFields.length > 0) {
            sql += ` GROUP BY ${this.groupByFields.join(', ')}`;
        }
        if (this.havingFilters.length > 0) {
            const havingClauses = this.havingFilters.map((filter) => {
                return `${filter.field} ${filter.operator} ${filter.value}`;
            });
            sql += ' HAVING ' + havingClauses.join(' AND ');
        }
        return sql;
    }
}
export function aggregate(table) {
    return new AggregateQueryBuilder(table);
}
//# sourceMappingURL=QueryBuilder.js.map