/**
 * CADDY v0.4.0 Enterprise Billing Settings
 * Subscription, payment methods, invoices
 */

import React, { useState, useCallback } from 'react';
import {
  BillingSettings as BillingSettingsType,
  Subscription,
  PaymentMethod,
  Invoice,
  ToastNotification,
  ConfirmationDialog,
  SettingsHistory,
} from './types';

interface BillingSettingsProps {
  onSave: (section: string, data: BillingSettingsType) => Promise<void>;
  onConfirm: (config: Omit<ConfirmationDialog, 'open'>) => void;
  addToast: (toast: Omit<ToastNotification, 'id'>) => void;
  addToHistory: (entry: Omit<SettingsHistory, 'id' | 'timestamp'>) => void;
}

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

const BillingSettings: React.FC<BillingSettingsProps> = ({
  onSave,
  onConfirm,
  addToast,
  addToHistory,
}) => {
  const [settings] = useState<BillingSettingsType>({
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
      storage: 54.3 * 1024 * 1024 * 1024, // 54.3 GB in bytes
      bandwidth: 120.5 * 1024 * 1024 * 1024, // 120.5 GB
      apiCalls: 15420,
      period: {
        start: new Date('2025-01-01'),
        end: new Date('2025-02-01'),
      },
      limits: {
        users: 100,
        storage: 100 * 1024 * 1024 * 1024, // 100 GB
        bandwidth: 500 * 1024 * 1024 * 1024, // 500 GB
        apiCalls: 100000,
      },
    },
  });

  const [showUpgradeModal, setShowUpgradeModal] = useState(false);
  const [selectedPlan, setSelectedPlan] = useState<string>(settings.subscription.plan);
  const [showAddPaymentMethod, setShowAddPaymentMethod] = useState(false);

  // Change plan
  const changePlan = useCallback(
    (planId: string) => {
      const plan = PLANS.find((p) => p.id === planId);
      if (!plan) return;

      onConfirm({
        title: `Upgrade to ${plan.name}`,
        message: `You will be charged $${plan.price}/month starting from your next billing cycle. Do you want to continue?`,
        severity: 'info',
        confirmText: 'Upgrade',
        cancelText: 'Cancel',
        onConfirm: async () => {
          try {
            // Simulate plan change
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
          } catch (error) {
            addToast({
              type: 'error',
              message: 'Failed to upgrade plan',
            });
          }
        },
        onCancel: () => {},
      });
    },
    [settings.subscription.plan, onConfirm, addToast, addToHistory]
  );

  // Cancel subscription
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
        } catch (error) {
          addToast({
            type: 'error',
            message: 'Failed to cancel subscription',
          });
        }
      },
      onCancel: () => {},
    });
  }, [onConfirm, addToast, addToHistory]);

  // Download invoice
  const downloadInvoice = useCallback(
    (invoice: Invoice) => {
      addToast({
        type: 'success',
        message: `Downloading invoice ${invoice.number}`,
      });
      // Simulate download
      console.log('Downloading invoice:', invoice.number);
    },
    [addToast]
  );

  const formatBytes = (bytes: number): string => {
    const gb = bytes / (1024 * 1024 * 1024);
    return `${gb.toFixed(1)} GB`;
  };

  const formatCurrency = (amount: number, currency: string = 'USD'): string => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency,
    }).format(amount);
  };

  const getUsagePercentage = (current: number, limit: number): number => {
    return Math.min((current / limit) * 100, 100);
  };

  return (
    <div style={{ maxWidth: '800px' }}>
      <div style={{ marginBottom: '2rem' }}>
        <h2 style={{ fontSize: '1.5rem', marginBottom: '0.5rem' }}>Billing Settings</h2>
        <p style={{ color: '#666', margin: 0 }}>
          Manage your subscription, payment methods, and invoices
        </p>
      </div>

      {/* Current Plan */}
      <section
        style={{
          backgroundColor: '#fff',
          borderRadius: '8px',
          padding: '1.5rem',
          marginBottom: '1.5rem',
          border: '1px solid #e0e0e0',
        }}
      >
        <h3 style={{ fontSize: '1.125rem', marginBottom: '1rem' }}>Current Plan</h3>

        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'start' }}>
          <div>
            <div style={{ fontSize: '1.5rem', fontWeight: 600, marginBottom: '0.5rem' }}>
              {PLANS.find((p) => p.id === settings.subscription.plan)?.name}
            </div>
            <div style={{ fontSize: '1.25rem', color: '#666', marginBottom: '1rem' }}>
              {formatCurrency(PLANS.find((p) => p.id === settings.subscription.plan)?.price || 0)}
              <span style={{ fontSize: '0.875rem' }}>/month</span>
            </div>
            <div style={{ fontSize: '0.875rem', color: '#666' }}>
              {settings.subscription.seats} seats • Renews on{' '}
              {settings.subscription.currentPeriodEnd.toLocaleDateString()}
            </div>
            {settings.subscription.cancelAtPeriodEnd && (
              <div
                style={{
                  marginTop: '0.5rem',
                  padding: '0.5rem',
                  backgroundColor: '#fff3e0',
                  color: '#e65100',
                  borderRadius: '4px',
                  fontSize: '0.875rem',
                }}
              >
                Subscription will be canceled on {settings.subscription.currentPeriodEnd.toLocaleDateString()}
              </div>
            )}
          </div>
          <div style={{ display: 'flex', gap: '0.5rem' }}>
            <button
              onClick={() => setShowUpgradeModal(true)}
              style={{
                padding: '0.5rem 1rem',
                backgroundColor: '#1976d2',
                color: '#fff',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
            >
              Change Plan
            </button>
            {!settings.subscription.cancelAtPeriodEnd && (
              <button
                onClick={cancelSubscription}
                style={{
                  padding: '0.5rem 1rem',
                  backgroundColor: '#fff',
                  color: '#d32f2f',
                  border: '1px solid #d32f2f',
                  borderRadius: '4px',
                  cursor: 'pointer',
                }}
              >
                Cancel
              </button>
            )}
          </div>
        </div>

        <div style={{ marginTop: '1.5rem' }}>
          <h4 style={{ fontSize: '0.875rem', fontWeight: 600, marginBottom: '0.5rem' }}>
            Plan Features
          </h4>
          <ul style={{ margin: 0, paddingLeft: '1.25rem' }}>
            {PLANS.find((p) => p.id === settings.subscription.plan)?.features.map((feature, idx) => (
              <li key={idx} style={{ marginBottom: '0.25rem', fontSize: '0.875rem' }}>
                {feature}
              </li>
            ))}
          </ul>
        </div>
      </section>

      {/* Usage Statistics */}
      <section
        style={{
          backgroundColor: '#fff',
          borderRadius: '8px',
          padding: '1.5rem',
          marginBottom: '1.5rem',
          border: '1px solid #e0e0e0',
        }}
      >
        <h3 style={{ fontSize: '1.125rem', marginBottom: '1rem' }}>Usage This Period</h3>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
          {/* Users */}
          <div>
            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.5rem' }}>
              <span style={{ fontSize: '0.875rem', fontWeight: 500 }}>Users</span>
              <span style={{ fontSize: '0.875rem' }}>
                {settings.usage.users} / {settings.usage.limits.users}
              </span>
            </div>
            <div style={{ width: '100%', height: '8px', backgroundColor: '#e0e0e0', borderRadius: '4px', overflow: 'hidden' }}>
              <div
                style={{
                  width: `${getUsagePercentage(settings.usage.users, settings.usage.limits.users)}%`,
                  height: '100%',
                  backgroundColor: '#1976d2',
                }}
              />
            </div>
          </div>

          {/* Storage */}
          <div>
            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.5rem' }}>
              <span style={{ fontSize: '0.875rem', fontWeight: 500 }}>Storage</span>
              <span style={{ fontSize: '0.875rem' }}>
                {formatBytes(settings.usage.storage)} / {formatBytes(settings.usage.limits.storage)}
              </span>
            </div>
            <div style={{ width: '100%', height: '8px', backgroundColor: '#e0e0e0', borderRadius: '4px', overflow: 'hidden' }}>
              <div
                style={{
                  width: `${getUsagePercentage(settings.usage.storage, settings.usage.limits.storage)}%`,
                  height: '100%',
                  backgroundColor: '#4caf50',
                }}
              />
            </div>
          </div>

          {/* Bandwidth */}
          <div>
            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.5rem' }}>
              <span style={{ fontSize: '0.875rem', fontWeight: 500 }}>Bandwidth</span>
              <span style={{ fontSize: '0.875rem' }}>
                {formatBytes(settings.usage.bandwidth)} / {formatBytes(settings.usage.limits.bandwidth)}
              </span>
            </div>
            <div style={{ width: '100%', height: '8px', backgroundColor: '#e0e0e0', borderRadius: '4px', overflow: 'hidden' }}>
              <div
                style={{
                  width: `${getUsagePercentage(settings.usage.bandwidth, settings.usage.limits.bandwidth)}%`,
                  height: '100%',
                  backgroundColor: '#ff9800',
                }}
              />
            </div>
          </div>

          {/* API Calls */}
          <div>
            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.5rem' }}>
              <span style={{ fontSize: '0.875rem', fontWeight: 500 }}>API Calls</span>
              <span style={{ fontSize: '0.875rem' }}>
                {settings.usage.apiCalls.toLocaleString()} / {settings.usage.limits.apiCalls.toLocaleString()}
              </span>
            </div>
            <div style={{ width: '100%', height: '8px', backgroundColor: '#e0e0e0', borderRadius: '4px', overflow: 'hidden' }}>
              <div
                style={{
                  width: `${getUsagePercentage(settings.usage.apiCalls, settings.usage.limits.apiCalls)}%`,
                  height: '100%',
                  backgroundColor: '#9c27b0',
                }}
              />
            </div>
          </div>
        </div>
      </section>

      {/* Payment Methods */}
      <section
        style={{
          backgroundColor: '#fff',
          borderRadius: '8px',
          padding: '1.5rem',
          marginBottom: '1.5rem',
          border: '1px solid #e0e0e0',
        }}
      >
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
          <h3 style={{ fontSize: '1.125rem', margin: 0 }}>Payment Methods</h3>
          <button
            onClick={() => setShowAddPaymentMethod(true)}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: '#1976d2',
              color: '#fff',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
            }}
          >
            + Add Payment Method
          </button>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
          {settings.paymentMethods.map((method) => (
            <div
              key={method.id}
              style={{
                padding: '1rem',
                border: '1px solid #e0e0e0',
                borderRadius: '4px',
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
              }}
            >
              <div>
                <div style={{ fontWeight: 600, marginBottom: '0.25rem' }}>
                  {method.brand} •••• {method.last4}
                  {method.isDefault && (
                    <span
                      style={{
                        marginLeft: '0.5rem',
                        padding: '0.125rem 0.5rem',
                        backgroundColor: '#e3f2fd',
                        color: '#1976d2',
                        fontSize: '0.75rem',
                        borderRadius: '12px',
                      }}
                    >
                      Default
                    </span>
                  )}
                </div>
                <div style={{ fontSize: '0.875rem', color: '#666' }}>
                  Expires {method.expiryMonth}/{method.expiryYear}
                </div>
              </div>
              {!method.isDefault && (
                <button
                  style={{
                    padding: '0.25rem 0.75rem',
                    backgroundColor: '#fff',
                    color: '#d32f2f',
                    border: '1px solid #d32f2f',
                    borderRadius: '4px',
                    cursor: 'pointer',
                    fontSize: '0.875rem',
                  }}
                >
                  Remove
                </button>
              )}
            </div>
          ))}
        </div>
      </section>

      {/* Billing Address */}
      <section
        style={{
          backgroundColor: '#fff',
          borderRadius: '8px',
          padding: '1.5rem',
          marginBottom: '1.5rem',
          border: '1px solid #e0e0e0',
        }}
      >
        <h3 style={{ fontSize: '1.125rem', marginBottom: '1rem' }}>Billing Address</h3>

        <div style={{ fontSize: '0.875rem', lineHeight: 1.6 }}>
          {settings.billingAddress.company && (
            <div style={{ fontWeight: 600 }}>{settings.billingAddress.company}</div>
          )}
          <div>{settings.billingAddress.addressLine1}</div>
          {settings.billingAddress.addressLine2 && (
            <div>{settings.billingAddress.addressLine2}</div>
          )}
          <div>
            {settings.billingAddress.city}, {settings.billingAddress.state} {settings.billingAddress.postalCode}
          </div>
          <div>{settings.billingAddress.country}</div>
          {settings.billingAddress.taxId && (
            <div style={{ marginTop: '0.5rem' }}>Tax ID: {settings.billingAddress.taxId}</div>
          )}
        </div>
      </section>

      {/* Invoices */}
      <section
        style={{
          backgroundColor: '#fff',
          borderRadius: '8px',
          padding: '1.5rem',
          border: '1px solid #e0e0e0',
        }}
      >
        <h3 style={{ fontSize: '1.125rem', marginBottom: '1rem' }}>Invoices</h3>

        <div style={{ overflowX: 'auto' }}>
          <table
            style={{
              width: '100%',
              borderCollapse: 'collapse',
              fontSize: '0.875rem',
            }}
          >
            <thead>
              <tr style={{ borderBottom: '2px solid #e0e0e0' }}>
                <th style={{ padding: '0.75rem', textAlign: 'left', fontWeight: 600 }}>Invoice</th>
                <th style={{ padding: '0.75rem', textAlign: 'left', fontWeight: 600 }}>Date</th>
                <th style={{ padding: '0.75rem', textAlign: 'left', fontWeight: 600 }}>Status</th>
                <th style={{ padding: '0.75rem', textAlign: 'right', fontWeight: 600 }}>Amount</th>
                <th style={{ padding: '0.75rem', textAlign: 'right', fontWeight: 600 }}>Action</th>
              </tr>
            </thead>
            <tbody>
              {settings.invoices.map((invoice) => (
                <tr key={invoice.id} style={{ borderBottom: '1px solid #f0f0f0' }}>
                  <td style={{ padding: '0.75rem' }}>{invoice.number}</td>
                  <td style={{ padding: '0.75rem' }}>{invoice.date.toLocaleDateString()}</td>
                  <td style={{ padding: '0.75rem' }}>
                    <span
                      style={{
                        padding: '0.25rem 0.5rem',
                        backgroundColor: invoice.status === 'paid' ? '#e8f5e9' : '#fff3e0',
                        color: invoice.status === 'paid' ? '#2e7d32' : '#e65100',
                        borderRadius: '4px',
                        fontSize: '0.75rem',
                        textTransform: 'capitalize',
                      }}
                    >
                      {invoice.status}
                    </span>
                  </td>
                  <td style={{ padding: '0.75rem', textAlign: 'right' }}>
                    {formatCurrency(invoice.total, invoice.currency)}
                  </td>
                  <td style={{ padding: '0.75rem', textAlign: 'right' }}>
                    <button
                      onClick={() => downloadInvoice(invoice)}
                      style={{
                        padding: '0.25rem 0.75rem',
                        backgroundColor: '#1976d2',
                        color: '#fff',
                        border: 'none',
                        borderRadius: '4px',
                        cursor: 'pointer',
                        fontSize: '0.75rem',
                      }}
                    >
                      Download
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {/* Upgrade Modal */}
      {showUpgradeModal && (
        <div
          style={{
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
          }}
          onClick={() => setShowUpgradeModal(false)}
        >
          <div
            onClick={(e) => e.stopPropagation()}
            style={{
              backgroundColor: '#fff',
              borderRadius: '8px',
              padding: '2rem',
              maxWidth: '900px',
              width: '90%',
              maxHeight: '80vh',
              overflowY: 'auto',
            }}
          >
            <h2 style={{ marginTop: 0 }}>Choose a Plan</h2>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '1rem' }}>
              {PLANS.map((plan) => (
                <div
                  key={plan.id}
                  style={{
                    padding: '1.5rem',
                    border: plan.id === selectedPlan ? '2px solid #1976d2' : '1px solid #e0e0e0',
                    borderRadius: '8px',
                    cursor: 'pointer',
                  }}
                  onClick={() => setSelectedPlan(plan.id)}
                >
                  <div style={{ fontSize: '1.25rem', fontWeight: 600, marginBottom: '0.5rem' }}>
                    {plan.name}
                  </div>
                  <div style={{ fontSize: '1.5rem', marginBottom: '1rem' }}>
                    ${plan.price}
                    <span style={{ fontSize: '0.875rem', color: '#666' }}>/mo</span>
                  </div>
                  <ul style={{ margin: 0, paddingLeft: '1.25rem', fontSize: '0.875rem' }}>
                    {plan.features.map((feature, idx) => (
                      <li key={idx} style={{ marginBottom: '0.25rem' }}>
                        {feature}
                      </li>
                    ))}
                  </ul>
                </div>
              ))}
            </div>
            <div style={{ marginTop: '2rem', display: 'flex', gap: '1rem', justifyContent: 'flex-end' }}>
              <button
                onClick={() => setShowUpgradeModal(false)}
                style={{
                  padding: '0.5rem 1.5rem',
                  backgroundColor: '#f5f5f5',
                  border: '1px solid #e0e0e0',
                  borderRadius: '4px',
                  cursor: 'pointer',
                }}
              >
                Cancel
              </button>
              <button
                onClick={() => changePlan(selectedPlan)}
                disabled={selectedPlan === settings.subscription.plan}
                style={{
                  padding: '0.5rem 1.5rem',
                  backgroundColor: selectedPlan === settings.subscription.plan ? '#ccc' : '#1976d2',
                  color: '#fff',
                  border: 'none',
                  borderRadius: '4px',
                  cursor: selectedPlan === settings.subscription.plan ? 'not-allowed' : 'pointer',
                }}
              >
                {selectedPlan === settings.subscription.plan ? 'Current Plan' : 'Upgrade'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default BillingSettings;
