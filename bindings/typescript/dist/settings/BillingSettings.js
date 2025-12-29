import React, { useState, useCallback } from 'react';
const PLANS = [
    {
        id: 'free',
        name: 'Free',
        price: 0,
        features: ['5 users', '1GB storage', 'Basic support'],
    },
    {
        id: 'starter',
        name: 'Starter',
        price: 29,
        features: ['25 users', '10GB storage', 'Email support', 'API access'],
    },
    {
        id: 'professional',
        name: 'Professional',
        price: 99,
        features: ['100 users', '100GB storage', 'Priority support', 'Advanced analytics'],
    },
    {
        id: 'enterprise',
        name: 'Enterprise',
        price: 499,
        features: ['Unlimited users', 'Unlimited storage', '24/7 support', 'Custom integrations', 'SLA'],
    },
];
const BillingSettings = ({ onSave, onConfirm, addToast, addToHistory, }) => {
    const [settings] = useState({
        id: 'billing-1',
        version: 1,
        updatedAt: new Date(),
        updatedBy: 'current-user',
        subscription: {
            plan: 'professional',
            status: 'active',
            billingCycle: 'monthly',
            currentPeriodStart: new Date('2025-01-01'),
            currentPeriodEnd: new Date('2025-02-01'),
            cancelAtPeriodEnd: false,
            seats: 50,
            addons: [],
        },
        paymentMethods: [
            {
                id: 'pm-1',
                type: 'card',
                isDefault: true,
                last4: '4242',
                brand: 'Visa',
                expiryMonth: 12,
                expiryYear: 2026,
            },
        ],
        billingAddress: {
            company: 'Acme Corporation',
            addressLine1: '123 Main Street',
            city: 'San Francisco',
            state: 'CA',
            postalCode: '94102',
            country: 'US',
            taxId: 'US123456789',
        },
        invoices: [
            {
                id: 'inv-1',
                number: 'INV-2025-001',
                date: new Date('2025-01-01'),
                dueDate: new Date('2025-01-15'),
                status: 'paid',
                subtotal: 99.00,
                tax: 8.91,
                total: 107.91,
                currency: 'USD',
                items: [
                    {
                        description: 'Professional Plan - Monthly',
                        quantity: 1,
                        unitPrice: 99.00,
                        amount: 99.00,
                    },
                ],
            },
        ],
        usage: {
            users: 42,
            storage: 54.3 * 1024 * 1024 * 1024,
            bandwidth: 120.5 * 1024 * 1024 * 1024,
            apiCalls: 15420,
            period: {
                start: new Date('2025-01-01'),
                end: new Date('2025-02-01'),
            },
            limits: {
                users: 100,
                storage: 100 * 1024 * 1024 * 1024,
                bandwidth: 500 * 1024 * 1024 * 1024,
                apiCalls: 100000,
            },
        },
    });
    const [showUpgradeModal, setShowUpgradeModal] = useState(false);
    const [selectedPlan, setSelectedPlan] = useState(settings.subscription.plan);
    const [showAddPaymentMethod, setShowAddPaymentMethod] = useState(false);
    const changePlan = useCallback((planId) => {
        const plan = PLANS.find((p) => p.id === planId);
        if (!plan)
            return;
        onConfirm({
            title: `Upgrade to ${plan.name}`,
            message: `You will be charged $${plan.price}/month starting from your next billing cycle. Do you want to continue?`,
            severity: 'info',
            confirmText: 'Upgrade',
            cancelText: 'Cancel',
            onConfirm: async () => {
                try {
                    await new Promise((resolve) => setTimeout(resolve, 1000));
                    addToast({
                        type: 'success',
                        message: `Successfully upgraded to ${plan.name} plan`,
                    });
                    addToHistory({
                        section: 'Billing Settings',
                        action: 'update',
                        changes: [{ field: 'plan', oldValue: settings.subscription.plan, newValue: planId }],
                        userId: 'current-user',
                        userName: 'Current User',
                    });
                    setShowUpgradeModal(false);
                }
                catch (error) {
                    addToast({
                        type: 'error',
                        message: 'Failed to upgrade plan',
                    });
                }
            },
            onCancel: () => { },
        });
    }, [settings.subscription.plan, onConfirm, addToast, addToHistory]);
    const cancelSubscription = useCallback(() => {
        onConfirm({
            title: 'Cancel Subscription',
            message: 'Are you sure you want to cancel your subscription? You will retain access until the end of your current billing period.',
            severity: 'warning',
            confirmText: 'Cancel Subscription',
            cancelText: 'Keep Subscription',
            onConfirm: async () => {
                try {
                    await new Promise((resolve) => setTimeout(resolve, 1000));
                    addToast({
                        type: 'success',
                        message: 'Subscription will be canceled at the end of the billing period',
                    });
                    addToHistory({
                        section: 'Billing Settings',
                        action: 'update',
                        changes: [{ field: 'cancelAtPeriodEnd', oldValue: false, newValue: true }],
                        userId: 'current-user',
                        userName: 'Current User',
                    });
                }
                catch (error) {
                    addToast({
                        type: 'error',
                        message: 'Failed to cancel subscription',
                    });
                }
            },
            onCancel: () => { },
        });
    }, [onConfirm, addToast, addToHistory]);
    const downloadInvoice = useCallback((invoice) => {
        addToast({
            type: 'success',
            message: `Downloading invoice ${invoice.number}`,
        });
        console.log('Downloading invoice:', invoice.number);
    }, [addToast]);
    const formatBytes = (bytes) => {
        const gb = bytes / (1024 * 1024 * 1024);
        return `${gb.toFixed(1)} GB`;
    };
    const formatCurrency = (amount, currency = 'USD') => {
        return new Intl.NumberFormat('en-US', {
            style: 'currency',
            currency,
        }).format(amount);
    };
    const getUsagePercentage = (current, limit) => {
        return Math.min((current / limit) * 100, 100);
    };
    return (React.createElement("div", { style: { maxWidth: '800px' } },
        React.createElement("div", { style: { marginBottom: '2rem' } },
            React.createElement("h2", { style: { fontSize: '1.5rem', marginBottom: '0.5rem' } }, "Billing Settings"),
            React.createElement("p", { style: { color: '#666', margin: 0 } }, "Manage your subscription, payment methods, and invoices")),
        React.createElement("section", { style: {
                backgroundColor: '#fff',
                borderRadius: '8px',
                padding: '1.5rem',
                marginBottom: '1.5rem',
                border: '1px solid #e0e0e0',
            } },
            React.createElement("h3", { style: { fontSize: '1.125rem', marginBottom: '1rem' } }, "Current Plan"),
            React.createElement("div", { style: { display: 'flex', justifyContent: 'space-between', alignItems: 'start' } },
                React.createElement("div", null,
                    React.createElement("div", { style: { fontSize: '1.5rem', fontWeight: 600, marginBottom: '0.5rem' } }, PLANS.find((p) => p.id === settings.subscription.plan)?.name),
                    React.createElement("div", { style: { fontSize: '1.25rem', color: '#666', marginBottom: '1rem' } },
                        formatCurrency(PLANS.find((p) => p.id === settings.subscription.plan)?.price || 0),
                        React.createElement("span", { style: { fontSize: '0.875rem' } }, "/month")),
                    React.createElement("div", { style: { fontSize: '0.875rem', color: '#666' } },
                        settings.subscription.seats,
                        " seats \u2022 Renews on",
                        ' ',
                        settings.subscription.currentPeriodEnd.toLocaleDateString()),
                    settings.subscription.cancelAtPeriodEnd && (React.createElement("div", { style: {
                            marginTop: '0.5rem',
                            padding: '0.5rem',
                            backgroundColor: '#fff3e0',
                            color: '#e65100',
                            borderRadius: '4px',
                            fontSize: '0.875rem',
                        } },
                        "Subscription will be canceled on ",
                        settings.subscription.currentPeriodEnd.toLocaleDateString()))),
                React.createElement("div", { style: { display: 'flex', gap: '0.5rem' } },
                    React.createElement("button", { onClick: () => setShowUpgradeModal(true), style: {
                            padding: '0.5rem 1rem',
                            backgroundColor: '#1976d2',
                            color: '#fff',
                            border: 'none',
                            borderRadius: '4px',
                            cursor: 'pointer',
                        } }, "Change Plan"),
                    !settings.subscription.cancelAtPeriodEnd && (React.createElement("button", { onClick: cancelSubscription, style: {
                            padding: '0.5rem 1rem',
                            backgroundColor: '#fff',
                            color: '#d32f2f',
                            border: '1px solid #d32f2f',
                            borderRadius: '4px',
                            cursor: 'pointer',
                        } }, "Cancel")))),
            React.createElement("div", { style: { marginTop: '1.5rem' } },
                React.createElement("h4", { style: { fontSize: '0.875rem', fontWeight: 600, marginBottom: '0.5rem' } }, "Plan Features"),
                React.createElement("ul", { style: { margin: 0, paddingLeft: '1.25rem' } }, PLANS.find((p) => p.id === settings.subscription.plan)?.features.map((feature, idx) => (React.createElement("li", { key: idx, style: { marginBottom: '0.25rem', fontSize: '0.875rem' } }, feature)))))),
        React.createElement("section", { style: {
                backgroundColor: '#fff',
                borderRadius: '8px',
                padding: '1.5rem',
                marginBottom: '1.5rem',
                border: '1px solid #e0e0e0',
            } },
            React.createElement("h3", { style: { fontSize: '1.125rem', marginBottom: '1rem' } }, "Usage This Period"),
            React.createElement("div", { style: { display: 'flex', flexDirection: 'column', gap: '1rem' } },
                React.createElement("div", null,
                    React.createElement("div", { style: { display: 'flex', justifyContent: 'space-between', marginBottom: '0.5rem' } },
                        React.createElement("span", { style: { fontSize: '0.875rem', fontWeight: 500 } }, "Users"),
                        React.createElement("span", { style: { fontSize: '0.875rem' } },
                            settings.usage.users,
                            " / ",
                            settings.usage.limits.users)),
                    React.createElement("div", { style: { width: '100%', height: '8px', backgroundColor: '#e0e0e0', borderRadius: '4px', overflow: 'hidden' } },
                        React.createElement("div", { style: {
                                width: `${getUsagePercentage(settings.usage.users, settings.usage.limits.users)}%`,
                                height: '100%',
                                backgroundColor: '#1976d2',
                            } }))),
                React.createElement("div", null,
                    React.createElement("div", { style: { display: 'flex', justifyContent: 'space-between', marginBottom: '0.5rem' } },
                        React.createElement("span", { style: { fontSize: '0.875rem', fontWeight: 500 } }, "Storage"),
                        React.createElement("span", { style: { fontSize: '0.875rem' } },
                            formatBytes(settings.usage.storage),
                            " / ",
                            formatBytes(settings.usage.limits.storage))),
                    React.createElement("div", { style: { width: '100%', height: '8px', backgroundColor: '#e0e0e0', borderRadius: '4px', overflow: 'hidden' } },
                        React.createElement("div", { style: {
                                width: `${getUsagePercentage(settings.usage.storage, settings.usage.limits.storage)}%`,
                                height: '100%',
                                backgroundColor: '#4caf50',
                            } }))),
                React.createElement("div", null,
                    React.createElement("div", { style: { display: 'flex', justifyContent: 'space-between', marginBottom: '0.5rem' } },
                        React.createElement("span", { style: { fontSize: '0.875rem', fontWeight: 500 } }, "Bandwidth"),
                        React.createElement("span", { style: { fontSize: '0.875rem' } },
                            formatBytes(settings.usage.bandwidth),
                            " / ",
                            formatBytes(settings.usage.limits.bandwidth))),
                    React.createElement("div", { style: { width: '100%', height: '8px', backgroundColor: '#e0e0e0', borderRadius: '4px', overflow: 'hidden' } },
                        React.createElement("div", { style: {
                                width: `${getUsagePercentage(settings.usage.bandwidth, settings.usage.limits.bandwidth)}%`,
                                height: '100%',
                                backgroundColor: '#ff9800',
                            } }))),
                React.createElement("div", null,
                    React.createElement("div", { style: { display: 'flex', justifyContent: 'space-between', marginBottom: '0.5rem' } },
                        React.createElement("span", { style: { fontSize: '0.875rem', fontWeight: 500 } }, "API Calls"),
                        React.createElement("span", { style: { fontSize: '0.875rem' } },
                            settings.usage.apiCalls.toLocaleString(),
                            " / ",
                            settings.usage.limits.apiCalls.toLocaleString())),
                    React.createElement("div", { style: { width: '100%', height: '8px', backgroundColor: '#e0e0e0', borderRadius: '4px', overflow: 'hidden' } },
                        React.createElement("div", { style: {
                                width: `${getUsagePercentage(settings.usage.apiCalls, settings.usage.limits.apiCalls)}%`,
                                height: '100%',
                                backgroundColor: '#9c27b0',
                            } }))))),
        React.createElement("section", { style: {
                backgroundColor: '#fff',
                borderRadius: '8px',
                padding: '1.5rem',
                marginBottom: '1.5rem',
                border: '1px solid #e0e0e0',
            } },
            React.createElement("div", { style: { display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' } },
                React.createElement("h3", { style: { fontSize: '1.125rem', margin: 0 } }, "Payment Methods"),
                React.createElement("button", { onClick: () => setShowAddPaymentMethod(true), style: {
                        padding: '0.5rem 1rem',
                        backgroundColor: '#1976d2',
                        color: '#fff',
                        border: 'none',
                        borderRadius: '4px',
                        cursor: 'pointer',
                    } }, "+ Add Payment Method")),
            React.createElement("div", { style: { display: 'flex', flexDirection: 'column', gap: '1rem' } }, settings.paymentMethods.map((method) => (React.createElement("div", { key: method.id, style: {
                    padding: '1rem',
                    border: '1px solid #e0e0e0',
                    borderRadius: '4px',
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                } },
                React.createElement("div", null,
                    React.createElement("div", { style: { fontWeight: 600, marginBottom: '0.25rem' } },
                        method.brand,
                        " \u2022\u2022\u2022\u2022 ",
                        method.last4,
                        method.isDefault && (React.createElement("span", { style: {
                                marginLeft: '0.5rem',
                                padding: '0.125rem 0.5rem',
                                backgroundColor: '#e3f2fd',
                                color: '#1976d2',
                                fontSize: '0.75rem',
                                borderRadius: '12px',
                            } }, "Default"))),
                    React.createElement("div", { style: { fontSize: '0.875rem', color: '#666' } },
                        "Expires ",
                        method.expiryMonth,
                        "/",
                        method.expiryYear)),
                !method.isDefault && (React.createElement("button", { style: {
                        padding: '0.25rem 0.75rem',
                        backgroundColor: '#fff',
                        color: '#d32f2f',
                        border: '1px solid #d32f2f',
                        borderRadius: '4px',
                        cursor: 'pointer',
                        fontSize: '0.875rem',
                    } }, "Remove"))))))),
        React.createElement("section", { style: {
                backgroundColor: '#fff',
                borderRadius: '8px',
                padding: '1.5rem',
                marginBottom: '1.5rem',
                border: '1px solid #e0e0e0',
            } },
            React.createElement("h3", { style: { fontSize: '1.125rem', marginBottom: '1rem' } }, "Billing Address"),
            React.createElement("div", { style: { fontSize: '0.875rem', lineHeight: 1.6 } },
                settings.billingAddress.company && (React.createElement("div", { style: { fontWeight: 600 } }, settings.billingAddress.company)),
                React.createElement("div", null, settings.billingAddress.addressLine1),
                settings.billingAddress.addressLine2 && (React.createElement("div", null, settings.billingAddress.addressLine2)),
                React.createElement("div", null,
                    settings.billingAddress.city,
                    ", ",
                    settings.billingAddress.state,
                    " ",
                    settings.billingAddress.postalCode),
                React.createElement("div", null, settings.billingAddress.country),
                settings.billingAddress.taxId && (React.createElement("div", { style: { marginTop: '0.5rem' } },
                    "Tax ID: ",
                    settings.billingAddress.taxId)))),
        React.createElement("section", { style: {
                backgroundColor: '#fff',
                borderRadius: '8px',
                padding: '1.5rem',
                border: '1px solid #e0e0e0',
            } },
            React.createElement("h3", { style: { fontSize: '1.125rem', marginBottom: '1rem' } }, "Invoices"),
            React.createElement("div", { style: { overflowX: 'auto' } },
                React.createElement("table", { style: {
                        width: '100%',
                        borderCollapse: 'collapse',
                        fontSize: '0.875rem',
                    } },
                    React.createElement("thead", null,
                        React.createElement("tr", { style: { borderBottom: '2px solid #e0e0e0' } },
                            React.createElement("th", { style: { padding: '0.75rem', textAlign: 'left', fontWeight: 600 } }, "Invoice"),
                            React.createElement("th", { style: { padding: '0.75rem', textAlign: 'left', fontWeight: 600 } }, "Date"),
                            React.createElement("th", { style: { padding: '0.75rem', textAlign: 'left', fontWeight: 600 } }, "Status"),
                            React.createElement("th", { style: { padding: '0.75rem', textAlign: 'right', fontWeight: 600 } }, "Amount"),
                            React.createElement("th", { style: { padding: '0.75rem', textAlign: 'right', fontWeight: 600 } }, "Action"))),
                    React.createElement("tbody", null, settings.invoices.map((invoice) => (React.createElement("tr", { key: invoice.id, style: { borderBottom: '1px solid #f0f0f0' } },
                        React.createElement("td", { style: { padding: '0.75rem' } }, invoice.number),
                        React.createElement("td", { style: { padding: '0.75rem' } }, invoice.date.toLocaleDateString()),
                        React.createElement("td", { style: { padding: '0.75rem' } },
                            React.createElement("span", { style: {
                                    padding: '0.25rem 0.5rem',
                                    backgroundColor: invoice.status === 'paid' ? '#e8f5e9' : '#fff3e0',
                                    color: invoice.status === 'paid' ? '#2e7d32' : '#e65100',
                                    borderRadius: '4px',
                                    fontSize: '0.75rem',
                                    textTransform: 'capitalize',
                                } }, invoice.status)),
                        React.createElement("td", { style: { padding: '0.75rem', textAlign: 'right' } }, formatCurrency(invoice.total, invoice.currency)),
                        React.createElement("td", { style: { padding: '0.75rem', textAlign: 'right' } },
                            React.createElement("button", { onClick: () => downloadInvoice(invoice), style: {
                                    padding: '0.25rem 0.75rem',
                                    backgroundColor: '#1976d2',
                                    color: '#fff',
                                    border: 'none',
                                    borderRadius: '4px',
                                    cursor: 'pointer',
                                    fontSize: '0.75rem',
                                } }, "Download"))))))))),
        showUpgradeModal && (React.createElement("div", { style: {
                position: 'fixed',
                top: 0,
                left: 0,
                right: 0,
                bottom: 0,
                backgroundColor: 'rgba(0, 0, 0, 0.5)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                zIndex: 1000,
            }, onClick: () => setShowUpgradeModal(false) },
            React.createElement("div", { onClick: (e) => e.stopPropagation(), style: {
                    backgroundColor: '#fff',
                    borderRadius: '8px',
                    padding: '2rem',
                    maxWidth: '900px',
                    width: '90%',
                    maxHeight: '80vh',
                    overflowY: 'auto',
                } },
                React.createElement("h2", { style: { marginTop: 0 } }, "Choose a Plan"),
                React.createElement("div", { style: { display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '1rem' } }, PLANS.map((plan) => (React.createElement("div", { key: plan.id, style: {
                        padding: '1.5rem',
                        border: plan.id === selectedPlan ? '2px solid #1976d2' : '1px solid #e0e0e0',
                        borderRadius: '8px',
                        cursor: 'pointer',
                    }, onClick: () => setSelectedPlan(plan.id) },
                    React.createElement("div", { style: { fontSize: '1.25rem', fontWeight: 600, marginBottom: '0.5rem' } }, plan.name),
                    React.createElement("div", { style: { fontSize: '1.5rem', marginBottom: '1rem' } },
                        "$",
                        plan.price,
                        React.createElement("span", { style: { fontSize: '0.875rem', color: '#666' } }, "/mo")),
                    React.createElement("ul", { style: { margin: 0, paddingLeft: '1.25rem', fontSize: '0.875rem' } }, plan.features.map((feature, idx) => (React.createElement("li", { key: idx, style: { marginBottom: '0.25rem' } }, feature)))))))),
                React.createElement("div", { style: { marginTop: '2rem', display: 'flex', gap: '1rem', justifyContent: 'flex-end' } },
                    React.createElement("button", { onClick: () => setShowUpgradeModal(false), style: {
                            padding: '0.5rem 1.5rem',
                            backgroundColor: '#f5f5f5',
                            border: '1px solid #e0e0e0',
                            borderRadius: '4px',
                            cursor: 'pointer',
                        } }, "Cancel"),
                    React.createElement("button", { onClick: () => changePlan(selectedPlan), disabled: selectedPlan === settings.subscription.plan, style: {
                            padding: '0.5rem 1.5rem',
                            backgroundColor: selectedPlan === settings.subscription.plan ? '#ccc' : '#1976d2',
                            color: '#fff',
                            border: 'none',
                            borderRadius: '4px',
                            cursor: selectedPlan === settings.subscription.plan ? 'not-allowed' : 'pointer',
                        } }, selectedPlan === settings.subscription.plan ? 'Current Plan' : 'Upgrade')))))));
};
export default BillingSettings;
//# sourceMappingURL=BillingSettings.js.map