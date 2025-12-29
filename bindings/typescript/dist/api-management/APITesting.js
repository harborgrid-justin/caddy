import React, { useState, useEffect } from 'react';
export const APITesting = ({ onRunTest, onRunSuite }) => {
    const [suites, setSuites] = useState([]);
    const [selectedSuite, setSelectedSuite] = useState(null);
    const [selectedTest, setSelectedTest] = useState(null);
    const [testResults, setTestResults] = useState({});
    const [isRunning, setIsRunning] = useState(false);
    const [isCreatingSuite, setIsCreatingSuite] = useState(false);
    const [isCreatingTest, setIsCreatingTest] = useState(false);
    useEffect(() => {
        loadTestSuites();
    }, []);
    const loadTestSuites = async () => {
        try {
            const mockEndpoint = {
                id: '1',
                path: '/api/v1/users',
                method: 'GET',
                version: 'v1',
                summary: 'Get users',
                description: 'Get all users',
                tags: ['users'],
                deprecated: false,
                security: [],
                parameters: [],
                responses: {},
                operationId: 'getUsers',
                metadata: {},
                createdAt: Date.now(),
                updatedAt: Date.now(),
            };
            const mockSuites = [
                {
                    id: '1',
                    name: 'User API Tests',
                    description: 'Test suite for user-related endpoints',
                    tests: [
                        {
                            id: 't1',
                            name: 'Get all users returns 200',
                            description: 'Verify that GET /users returns 200 status',
                            request: {
                                endpoint: mockEndpoint,
                                parameters: {},
                                headers: { Authorization: 'Bearer token' },
                            },
                            assertions: [
                                {
                                    type: 'status',
                                    operator: 'equals',
                                    expected: 200,
                                },
                                {
                                    type: 'response_time',
                                    operator: 'less_than',
                                    expected: 500,
                                },
                            ],
                            createdAt: Date.now() - 86400000,
                            updatedAt: Date.now() - 86400000,
                        },
                        {
                            id: 't2',
                            name: 'Response contains users array',
                            description: 'Verify response has users array',
                            request: {
                                endpoint: mockEndpoint,
                                parameters: {},
                                headers: { Authorization: 'Bearer token' },
                            },
                            assertions: [
                                {
                                    type: 'body',
                                    operator: 'contains',
                                    expected: 'users',
                                },
                            ],
                            createdAt: Date.now() - 86400000,
                            updatedAt: Date.now() - 86400000,
                        },
                    ],
                    environment: 'staging',
                    variables: {
                        baseUrl: 'https://api.staging.example.com',
                        apiKey: 'test_key',
                    },
                    createdAt: Date.now() - 86400000 * 7,
                    updatedAt: Date.now() - 86400000,
                },
            ];
            setSuites(mockSuites);
            if (mockSuites.length > 0) {
                setSelectedSuite(mockSuites[0]);
            }
        }
        catch (error) {
            console.error('Failed to load test suites:', error);
        }
    };
    const handleRunTest = async (testId) => {
        setIsRunning(true);
        try {
            let result;
            if (onRunTest) {
                result = await onRunTest(testId);
            }
            else {
                await new Promise((resolve) => setTimeout(resolve, 1000));
                const passed = Math.random() > 0.3;
                result = {
                    testId,
                    passed,
                    assertions: [
                        {
                            type: 'status',
                            operator: 'equals',
                            expected: 200,
                            actual: passed ? 200 : 500,
                            passed,
                            message: passed ? 'Status code matched' : 'Expected 200 but got 500',
                        },
                    ],
                    response: {
                        status: passed ? 200 : 500,
                        statusText: passed ? 'OK' : 'Internal Server Error',
                        headers: { 'content-type': 'application/json' },
                        body: { users: [] },
                        duration: 142,
                        size: 156,
                        timestamp: Date.now(),
                    },
                    duration: 142,
                    timestamp: Date.now(),
                };
            }
            setTestResults({ ...testResults, [testId]: result });
        }
        catch (error) {
            console.error('Failed to run test:', error);
        }
        finally {
            setIsRunning(false);
        }
    };
    const handleRunSuite = async (suiteId) => {
        const suite = suites.find((s) => s.id === suiteId);
        if (!suite)
            return;
        setIsRunning(true);
        try {
            const results = {};
            if (onRunSuite) {
                const suiteResults = await onRunSuite(suiteId);
                suiteResults.forEach((result) => {
                    results[result.testId] = result;
                });
            }
            else {
                for (const test of suite.tests) {
                    await new Promise((resolve) => setTimeout(resolve, 500));
                    const passed = Math.random() > 0.3;
                    results[test.id] = {
                        testId: test.id,
                        passed,
                        assertions: test.assertions.map((assertion) => ({
                            ...assertion,
                            actual: passed ? assertion.expected : 'failed',
                            passed,
                        })),
                        duration: 142,
                        timestamp: Date.now(),
                    };
                }
            }
            setTestResults({ ...testResults, ...results });
        }
        catch (error) {
            console.error('Failed to run test suite:', error);
        }
        finally {
            setIsRunning(false);
        }
    };
    const getSuitePassRate = (suite) => {
        const results = suite.tests.map((t) => testResults[t.id]).filter(Boolean);
        if (results.length === 0)
            return 0;
        const passed = results.filter((r) => r.passed).length;
        return (passed / results.length) * 100;
    };
    return (React.createElement("div", { className: "min-h-screen bg-gray-50 dark:bg-gray-900" },
        React.createElement("div", { className: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8" },
            React.createElement("div", { className: "flex items-center justify-between mb-6" },
                React.createElement("h1", { className: "text-2xl font-bold text-gray-900 dark:text-white" }, "API Testing"),
                React.createElement("button", { onClick: () => setIsCreatingSuite(true), className: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors" }, "+ Create Test Suite")),
            React.createElement("div", { className: "grid grid-cols-1 md:grid-cols-3 gap-6 mb-8" },
                React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6" },
                    React.createElement("div", { className: "text-sm text-gray-600 dark:text-gray-400 mb-1" }, "Total Suites"),
                    React.createElement("div", { className: "text-3xl font-bold text-gray-900 dark:text-white" }, suites.length)),
                React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6" },
                    React.createElement("div", { className: "text-sm text-gray-600 dark:text-gray-400 mb-1" }, "Total Tests"),
                    React.createElement("div", { className: "text-3xl font-bold text-gray-900 dark:text-white" }, suites.reduce((sum, s) => sum + s.tests.length, 0))),
                React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6" },
                    React.createElement("div", { className: "text-sm text-gray-600 dark:text-gray-400 mb-1" }, "Pass Rate"),
                    React.createElement("div", { className: "text-3xl font-bold text-green-600 dark:text-green-400" },
                        selectedSuite ? getSuitePassRate(selectedSuite).toFixed(1) : 0,
                        "%"))),
            React.createElement("div", { className: "grid grid-cols-1 lg:grid-cols-3 gap-6" },
                React.createElement("div", { className: "lg:col-span-2 space-y-4" },
                    suites.map((suite) => (React.createElement("div", { key: suite.id, className: "bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700" },
                        React.createElement("div", { onClick: () => setSelectedSuite(suite), className: `px-6 py-4 cursor-pointer border-b border-gray-200 dark:border-gray-700 ${selectedSuite?.id === suite.id ? 'bg-blue-50 dark:bg-blue-900/20' : ''}` },
                            React.createElement("div", { className: "flex items-start justify-between mb-2" },
                                React.createElement("div", { className: "flex-1" },
                                    React.createElement("h3", { className: "text-lg font-semibold text-gray-900 dark:text-white mb-1" }, suite.name),
                                    React.createElement("p", { className: "text-sm text-gray-600 dark:text-gray-400" }, suite.description)),
                                React.createElement("button", { onClick: (e) => {
                                        e.stopPropagation();
                                        handleRunSuite(suite.id);
                                    }, disabled: isRunning, className: "px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 disabled:opacity-50 transition-colors text-sm" }, isRunning ? 'Running...' : 'Run Suite')),
                            React.createElement("div", { className: "flex items-center space-x-4 text-sm text-gray-600 dark:text-gray-400" },
                                React.createElement("span", null,
                                    suite.tests.length,
                                    " tests"),
                                React.createElement("span", null,
                                    "Environment: ",
                                    suite.environment),
                                getSuitePassRate(suite) > 0 && (React.createElement("span", { className: "text-green-600 dark:text-green-400" },
                                    getSuitePassRate(suite).toFixed(0),
                                    "% pass rate")))),
                        selectedSuite?.id === suite.id && (React.createElement("div", { className: "divide-y divide-gray-200 dark:divide-gray-700" },
                            suite.tests.map((test) => {
                                const result = testResults[test.id];
                                return (React.createElement("div", { key: test.id, onClick: () => setSelectedTest(test), className: "px-6 py-4 hover:bg-gray-50 dark:hover:bg-gray-700/50 cursor-pointer" },
                                    React.createElement("div", { className: "flex items-start justify-between" },
                                        React.createElement("div", { className: "flex-1" },
                                            React.createElement("div", { className: "flex items-center space-x-2 mb-1" },
                                                result && (React.createElement("span", { className: "text-lg" }, result.passed ? '✅' : '❌')),
                                                React.createElement("span", { className: "font-medium text-gray-900 dark:text-white" }, test.name)),
                                            React.createElement("p", { className: "text-sm text-gray-600 dark:text-gray-400" }, test.description),
                                            result && (React.createElement("div", { className: "mt-2 text-xs text-gray-500 dark:text-gray-400" },
                                                "Duration: ",
                                                result.duration,
                                                "ms \u2022 ",
                                                result.assertions.length,
                                                ' ',
                                                "assertion(s)"))),
                                        React.createElement("button", { onClick: (e) => {
                                                e.stopPropagation();
                                                handleRunTest(test.id);
                                            }, disabled: isRunning, className: "px-3 py-1 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50" }, "Run"))));
                            }),
                            React.createElement("div", { className: "px-6 py-3 bg-gray-50 dark:bg-gray-700" },
                                React.createElement("button", { onClick: () => setIsCreatingTest(true), className: "text-sm text-blue-600 dark:text-blue-400 hover:text-blue-800" }, "+ Add Test to Suite"))))))),
                    suites.length === 0 && (React.createElement("div", { className: "text-center py-12 text-gray-500 dark:text-gray-400" }, "No test suites found. Create your first test suite to get started."))),
                React.createElement("div", { className: "lg:col-span-1" }, selectedTest && testResults[selectedTest.id] ? (React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 sticky top-4" },
                    React.createElement("div", { className: "px-6 py-4 border-b border-gray-200 dark:border-gray-700" },
                        React.createElement("h3", { className: "text-lg font-semibold text-gray-900 dark:text-white" }, "Test Results")),
                    React.createElement("div", { className: "p-6 space-y-4 max-h-[600px] overflow-y-auto" },
                        React.createElement("div", { className: `p-4 rounded-lg ${testResults[selectedTest.id].passed
                                ? 'bg-green-50 dark:bg-green-900/20'
                                : 'bg-red-50 dark:bg-red-900/20'}` },
                            React.createElement("div", { className: "text-center" },
                                React.createElement("div", { className: "text-4xl mb-2" }, testResults[selectedTest.id].passed ? '✅' : '❌'),
                                React.createElement("div", { className: `font-semibold ${testResults[selectedTest.id].passed
                                        ? 'text-green-800 dark:text-green-200'
                                        : 'text-red-800 dark:text-red-200'}` }, testResults[selectedTest.id].passed ? 'PASSED' : 'FAILED'))),
                        React.createElement("div", null,
                            React.createElement("h4", { className: "text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2" }, "Assertions"),
                            React.createElement("div", { className: "space-y-2" }, testResults[selectedTest.id].assertions.map((assertion, index) => (React.createElement("div", { key: index, className: `p-3 rounded border ${assertion.passed
                                    ? 'border-green-200 dark:border-green-800 bg-green-50 dark:bg-green-900/20'
                                    : 'border-red-200 dark:border-red-800 bg-red-50 dark:bg-red-900/20'}` },
                                React.createElement("div", { className: "flex items-start space-x-2" },
                                    React.createElement("span", null, assertion.passed ? '✅' : '❌'),
                                    React.createElement("div", { className: "flex-1 text-sm" },
                                        React.createElement("div", { className: "font-medium text-gray-900 dark:text-white" },
                                            assertion.type,
                                            ": ",
                                            assertion.operator),
                                        React.createElement("div", { className: "text-gray-600 dark:text-gray-400 mt-1" },
                                            "Expected: ",
                                            JSON.stringify(assertion.expected)),
                                        assertion.actual !== undefined && (React.createElement("div", { className: "text-gray-600 dark:text-gray-400" },
                                            "Actual: ",
                                            JSON.stringify(assertion.actual))),
                                        assertion.message && (React.createElement("div", { className: "text-xs mt-1 text-gray-500 dark:text-gray-400" }, assertion.message))))))))),
                        testResults[selectedTest.id].response && (React.createElement("div", null,
                            React.createElement("h4", { className: "text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2" }, "Response"),
                            React.createElement("div", { className: "bg-gray-900 text-gray-100 p-3 rounded text-xs overflow-x-auto" },
                                React.createElement("pre", null, JSON.stringify(testResults[selectedTest.id].response, null, 2)))))))) : (React.createElement("div", { className: "bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6 text-center text-gray-500 dark:text-gray-400 sticky top-4" }, "Run a test to see results")))))));
};
export default APITesting;
//# sourceMappingURL=APITesting.js.map