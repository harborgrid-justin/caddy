/**
 * Integration Hub Component
 *
 * Marketplace UI for browsing, installing, and managing CI/CD integrations.
 */

import React, { useState, useEffect, useMemo } from 'react';
import {
  CIPlatform,
  IntegrationMarketplaceItem,
  IntegrationCategory,
  IntegrationConfig,
  IntegrationStatus,
} from './types';
import { Button, Input, Modal, Select, Table, Tabs } from '../enterprise';

interface IntegrationHubProps {
  /** Currently installed integrations */
  installedIntegrations?: IntegrationConfig[];

  /** Callback when integration is installed */
  onInstall?: (platform: CIPlatform) => void;

  /** Callback when integration is configured */
  onConfigure?: (config: IntegrationConfig) => void;

  /** Callback when integration is removed */
  onRemove?: (id: string) => void;
}

/**
 * Integration Hub - Marketplace and management UI
 */
export const IntegrationHub: React.FC<IntegrationHubProps> = ({
  installedIntegrations = [],
  onInstall,
  onConfigure,
  onRemove,
}) => {
  const [activeTab, setActiveTab] = useState<'marketplace' | 'installed'>('marketplace');
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<IntegrationCategory | 'all'>('all');
  const [marketplaceItems, setMarketplaceItems] = useState<IntegrationMarketplaceItem[]>([]);
  const [selectedItem, setSelectedItem] = useState<IntegrationMarketplaceItem | null>(null);
  const [showDetails, setShowDetails] = useState(false);

  // Load marketplace items
  useEffect(() => {
    loadMarketplaceItems();
  }, []);

  const loadMarketplaceItems = () => {
    // In a real application, this would fetch from an API
    const items: IntegrationMarketplaceItem[] = [
      {
        id: 'github',
        platform: CIPlatform.GitHub,
        name: 'github',
        displayName: 'GitHub Actions',
        description: 'Integrate with GitHub Actions for automated pull request checks and status updates.',
        icon: '🐙',
        category: IntegrationCategory.CI,
        version: '1.0.0',
        author: 'CADDY Team',
        documentation: 'https://docs.caddy.dev/integrations/github',
        features: [
          'GitHub App installation',
          'Pull request checks',
          'Status checks API',
          'Commit annotations',
          'GitHub Actions integration',
          'Webhook support',
        ],
        requirements: [
          'GitHub App with appropriate permissions',
          'Repository access',
        ],
        popularity: 1500,
        rating: 4.8,
        downloads: 12500,
        isOfficial: true,
        isPremium: false,
        tags: ['github', 'ci', 'pull-request', 'checks'],
      },
      {
        id: 'gitlab',
        platform: CIPlatform.GitLab,
        name: 'gitlab',
        displayName: 'GitLab CI/CD',
        description: 'Integrate with GitLab CI/CD pipelines and merge requests.',
        icon: '🦊',
        category: IntegrationCategory.CI,
        version: '1.0.0',
        author: 'CADDY Team',
        documentation: 'https://docs.caddy.dev/integrations/gitlab',
        features: [
          'GitLab CI/CD integration',
          'Merge request comments',
          'Pipeline status updates',
          'Code quality reports',
          'Webhook support',
        ],
        requirements: [
          'GitLab Personal Access Token',
          'Project access',
        ],
        popularity: 1200,
        rating: 4.7,
        downloads: 9800,
        isOfficial: true,
        isPremium: false,
        tags: ['gitlab', 'ci', 'merge-request', 'pipeline'],
      },
      {
        id: 'jenkins',
        platform: CIPlatform.Jenkins,
        name: 'jenkins',
        displayName: 'Jenkins',
        description: 'Integrate with Jenkins for continuous integration and delivery.',
        icon: '🏗️',
        category: IntegrationCategory.CI,
        version: '1.0.0',
        author: 'CADDY Team',
        documentation: 'https://docs.caddy.dev/integrations/jenkins',
        features: [
          'Jenkins plugin support',
          'Build step integration',
          'JUnit XML reports',
          'HTML reports',
          'Pipeline support',
        ],
        requirements: [
          'Jenkins instance',
          'API token',
        ],
        popularity: 800,
        rating: 4.5,
        downloads: 6500,
        isOfficial: true,
        isPremium: false,
        tags: ['jenkins', 'ci', 'build', 'reports'],
      },
      {
        id: 'azure-devops',
        platform: CIPlatform.AzureDevOps,
        name: 'azure-devops',
        displayName: 'Azure DevOps',
        description: 'Integrate with Azure Pipelines and DevOps services.',
        icon: '☁️',
        category: IntegrationCategory.CI,
        version: '1.0.0',
        author: 'CADDY Team',
        documentation: 'https://docs.caddy.dev/integrations/azure-devops',
        features: [
          'Azure Pipelines tasks',
          'Pull request policies',
          'Work item linking',
          'Board integration',
          'Test results publishing',
        ],
        requirements: [
          'Azure DevOps organization',
          'Personal Access Token',
        ],
        popularity: 950,
        rating: 4.6,
        downloads: 7800,
        isOfficial: true,
        isPremium: false,
        tags: ['azure', 'devops', 'ci', 'pipeline'],
      },
      {
        id: 'bitbucket',
        platform: CIPlatform.Bitbucket,
        name: 'bitbucket',
        displayName: 'Bitbucket Pipelines',
        description: 'Integrate with Bitbucket Pipelines and pull requests.',
        icon: '🪣',
        category: IntegrationCategory.CI,
        version: '1.0.0',
        author: 'CADDY Team',
        documentation: 'https://docs.caddy.dev/integrations/bitbucket',
        features: [
          'Bitbucket Pipelines integration',
          'Pull request checks',
          'Code Insights (Cloud)',
          'Commit status updates',
          'Webhook support',
        ],
        requirements: [
          'Bitbucket Cloud or Server',
          'App password or Personal Access Token',
        ],
        popularity: 600,
        rating: 4.4,
        downloads: 4200,
        isOfficial: true,
        isPremium: false,
        tags: ['bitbucket', 'ci', 'pull-request', 'pipeline'],
      },
    ];

    setMarketplaceItems(items);
  };

  // Filter marketplace items
  const filteredItems = useMemo(() => {
    return marketplaceItems.filter((item) => {
      const matchesSearch =
        searchQuery === '' ||
        item.displayName.toLowerCase().includes(searchQuery.toLowerCase()) ||
        item.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
        item.tags.some((tag) => tag.toLowerCase().includes(searchQuery.toLowerCase()));

      const matchesCategory =
        selectedCategory === 'all' || item.category === selectedCategory;

      return matchesSearch && matchesCategory;
    });
  }, [marketplaceItems, searchQuery, selectedCategory]);

  // Check if integration is installed
  const isInstalled = (platform: CIPlatform): boolean => {
    return installedIntegrations.some((config) => config.platform === platform);
  };

  // Handle install
  const handleInstall = (item: IntegrationMarketplaceItem) => {
    setSelectedItem(item);
    onInstall?.(item.platform);
  };

  // Handle view details
  const handleViewDetails = (item: IntegrationMarketplaceItem) => {
    setSelectedItem(item);
    setShowDetails(true);
  };

  return (
    <div className="integration-hub">
      <div className="hub-header">
        <h1>Integration Hub</h1>
        <p>Browse and manage CI/CD integrations for CADDY</p>
      </div>

      <Tabs
        value={activeTab}
        onChange={(value) => setActiveTab(value as 'marketplace' | 'installed')}
        tabs={[
          { id: 'marketplace', label: 'Marketplace', icon: '🛒' },
          { id: 'installed', label: `Installed (${installedIntegrations.length})`, icon: '✓' },
        ]}
      />

      {activeTab === 'marketplace' && (
        <div className="marketplace-view">
          <div className="marketplace-filters">
            <Input
              type="search"
              placeholder="Search integrations..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              icon="🔍"
            />

            <Select
              value={selectedCategory}
              onChange={(value) => setSelectedCategory(value as IntegrationCategory | 'all')}
              options={[
                { value: 'all', label: 'All Categories' },
                { value: IntegrationCategory.CI, label: 'Continuous Integration' },
                { value: IntegrationCategory.CD, label: 'Continuous Delivery' },
                { value: IntegrationCategory.Testing, label: 'Testing' },
                { value: IntegrationCategory.Security, label: 'Security' },
                { value: IntegrationCategory.Monitoring, label: 'Monitoring' },
                { value: IntegrationCategory.Notification, label: 'Notification' },
              ]}
            />
          </div>

          <div className="marketplace-grid">
            {filteredItems.map((item) => (
              <IntegrationCard
                key={item.id}
                item={item}
                installed={isInstalled(item.platform)}
                onInstall={() => handleInstall(item)}
                onViewDetails={() => handleViewDetails(item)}
              />
            ))}
          </div>

          {filteredItems.length === 0 && (
            <div className="no-results">
              <p>No integrations found matching your criteria.</p>
            </div>
          )}
        </div>
      )}

      {activeTab === 'installed' && (
        <div className="installed-view">
          <InstalledIntegrationsTable
            integrations={installedIntegrations}
            onConfigure={onConfigure}
            onRemove={onRemove}
          />
        </div>
      )}

      {showDetails && selectedItem && (
        <IntegrationDetailsModal
          item={selectedItem}
          installed={isInstalled(selectedItem.platform)}
          onClose={() => setShowDetails(false)}
          onInstall={() => handleInstall(selectedItem)}
        />
      )}

      <style jsx>{`
        .integration-hub {
          padding: 24px;
          max-width: 1400px;
          margin: 0 auto;
        }

        .hub-header {
          margin-bottom: 32px;
        }

        .hub-header h1 {
          font-size: 32px;
          font-weight: 700;
          margin-bottom: 8px;
        }

        .hub-header p {
          font-size: 16px;
          color: #666;
        }

        .marketplace-filters {
          display: flex;
          gap: 16px;
          margin: 24px 0;
        }

        .marketplace-grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
          gap: 24px;
          margin-top: 24px;
        }

        .no-results {
          text-align: center;
          padding: 64px 24px;
          color: #666;
        }
      `}</style>
    </div>
  );
};

/**
 * Integration card component
 */
interface IntegrationCardProps {
  item: IntegrationMarketplaceItem;
  installed: boolean;
  onInstall: () => void;
  onViewDetails: () => void;
}

const IntegrationCard: React.FC<IntegrationCardProps> = ({
  item,
  installed,
  onInstall,
  onViewDetails,
}) => {
  return (
    <div className={`integration-card ${installed ? 'installed' : ''}`}>
      <div className="card-header">
        <div className="icon">{item.icon}</div>
        <div className="title-area">
          <h3>{item.displayName}</h3>
          {item.isOfficial && <span className="badge official">Official</span>}
          {item.isPremium && <span className="badge premium">Premium</span>}
        </div>
      </div>

      <p className="description">{item.description}</p>

      <div className="stats">
        <div className="stat">
          <span className="label">Rating:</span>
          <span className="value">⭐ {item.rating.toFixed(1)}</span>
        </div>
        <div className="stat">
          <span className="label">Downloads:</span>
          <span className="value">{item.downloads.toLocaleString()}</span>
        </div>
      </div>

      <div className="tags">
        {item.tags.slice(0, 3).map((tag) => (
          <span key={tag} className="tag">
            {tag}
          </span>
        ))}
      </div>

      <div className="actions">
        <Button onClick={onViewDetails} variant="secondary" size="small">
          Details
        </Button>
        {installed ? (
          <Button variant="success" size="small" disabled>
            ✓ Installed
          </Button>
        ) : (
          <Button onClick={onInstall} variant="primary" size="small">
            Install
          </Button>
        )}
      </div>

      <style jsx>{`
        .integration-card {
          background: white;
          border: 1px solid #e0e0e0;
          border-radius: 8px;
          padding: 24px;
          transition: all 0.2s;
        }

        .integration-card:hover {
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
          transform: translateY(-2px);
        }

        .integration-card.installed {
          border-color: #4caf50;
          background: #f1f8f4;
        }

        .card-header {
          display: flex;
          align-items: flex-start;
          gap: 12px;
          margin-bottom: 16px;
        }

        .icon {
          font-size: 40px;
        }

        .title-area {
          flex: 1;
        }

        .title-area h3 {
          font-size: 20px;
          font-weight: 600;
          margin: 0 0 4px 0;
        }

        .badge {
          display: inline-block;
          padding: 2px 8px;
          border-radius: 4px;
          font-size: 12px;
          font-weight: 500;
        }

        .badge.official {
          background: #2196f3;
          color: white;
        }

        .badge.premium {
          background: #ff9800;
          color: white;
        }

        .description {
          color: #666;
          font-size: 14px;
          line-height: 1.5;
          margin-bottom: 16px;
        }

        .stats {
          display: flex;
          gap: 16px;
          margin-bottom: 12px;
          font-size: 13px;
        }

        .stat {
          display: flex;
          gap: 4px;
        }

        .stat .label {
          color: #999;
        }

        .stat .value {
          font-weight: 500;
        }

        .tags {
          display: flex;
          flex-wrap: wrap;
          gap: 6px;
          margin-bottom: 16px;
        }

        .tag {
          padding: 4px 8px;
          background: #f5f5f5;
          border-radius: 4px;
          font-size: 12px;
          color: #666;
        }

        .actions {
          display: flex;
          gap: 8px;
          margin-top: 16px;
        }
      `}</style>
    </div>
  );
};

/**
 * Installed integrations table
 */
interface InstalledIntegrationsTableProps {
  integrations: IntegrationConfig[];
  onConfigure?: (config: IntegrationConfig) => void;
  onRemove?: (id: string) => void;
}

const InstalledIntegrationsTable: React.FC<InstalledIntegrationsTableProps> = ({
  integrations,
  onConfigure,
  onRemove,
}) => {
  const getStatusBadge = (status: IntegrationStatus) => {
    const colors = {
      [IntegrationStatus.Active]: '#4caf50',
      [IntegrationStatus.Configured]: '#2196f3',
      [IntegrationStatus.NotConfigured]: '#ff9800',
      [IntegrationStatus.Error]: '#f44336',
      [IntegrationStatus.Disabled]: '#9e9e9e',
    };

    return (
      <span
        style={{
          padding: '4px 8px',
          borderRadius: '4px',
          fontSize: '12px',
          fontWeight: 500,
          background: colors[status],
          color: 'white',
        }}
      >
        {status}
      </span>
    );
  };

  if (integrations.length === 0) {
    return (
      <div style={{ textAlign: 'center', padding: '64px 24px', color: '#666' }}>
        <p>No integrations installed yet.</p>
        <p>Visit the Marketplace to install your first integration.</p>
      </div>
    );
  }

  return (
    <Table
      columns={[
        {
          key: 'platform',
          header: 'Platform',
          render: (item: IntegrationConfig) => item.platform,
        },
        {
          key: 'name',
          header: 'Name',
          render: (item: IntegrationConfig) => item.name,
        },
        {
          key: 'status',
          header: 'Status',
          render: (item: IntegrationConfig) => getStatusBadge(item.status),
        },
        {
          key: 'enabled',
          header: 'Enabled',
          render: (item: IntegrationConfig) => (item.enabled ? '✓' : '✗'),
        },
        {
          key: 'updatedAt',
          header: 'Last Updated',
          render: (item: IntegrationConfig) => new Date(item.updatedAt).toLocaleDateString(),
        },
        {
          key: 'actions',
          header: 'Actions',
          render: (item: IntegrationConfig) => (
            <div style={{ display: 'flex', gap: '8px' }}>
              <Button
                onClick={() => onConfigure?.(item)}
                variant="secondary"
                size="small"
              >
                Configure
              </Button>
              <Button
                onClick={() => onRemove?.(item.id)}
                variant="danger"
                size="small"
              >
                Remove
              </Button>
            </div>
          ),
        },
      ]}
      data={integrations}
    />
  );
};

/**
 * Integration details modal
 */
interface IntegrationDetailsModalProps {
  item: IntegrationMarketplaceItem;
  installed: boolean;
  onClose: () => void;
  onInstall: () => void;
}

const IntegrationDetailsModal: React.FC<IntegrationDetailsModalProps> = ({
  item,
  installed,
  onClose,
  onInstall,
}) => {
  return (
    <Modal
      isOpen={true}
      onClose={onClose}
      title={item.displayName}
      size="large"
    >
      <div className="details-content">
        <div className="details-header">
          <div className="icon">{item.icon}</div>
          <div>
            <h2>{item.displayName}</h2>
            <p className="author">by {item.author} • v{item.version}</p>
          </div>
        </div>

        <p className="description">{item.description}</p>

        <section>
          <h3>Features</h3>
          <ul>
            {item.features.map((feature, index) => (
              <li key={index}>{feature}</li>
            ))}
          </ul>
        </section>

        <section>
          <h3>Requirements</h3>
          <ul>
            {item.requirements.map((req, index) => (
              <li key={index}>{req}</li>
            ))}
          </ul>
        </section>

        <div className="details-footer">
          <Button onClick={onClose} variant="secondary">
            Close
          </Button>
          {!installed && (
            <Button onClick={onInstall} variant="primary">
              Install Integration
            </Button>
          )}
        </div>
      </div>

      <style jsx>{`
        .details-content {
          padding: 24px;
        }

        .details-header {
          display: flex;
          align-items: center;
          gap: 16px;
          margin-bottom: 24px;
        }

        .details-header .icon {
          font-size: 64px;
        }

        .details-header h2 {
          margin: 0 0 4px 0;
          font-size: 28px;
        }

        .author {
          color: #666;
          margin: 0;
        }

        .description {
          font-size: 16px;
          line-height: 1.6;
          color: #333;
          margin-bottom: 24px;
        }

        section {
          margin-bottom: 24px;
        }

        section h3 {
          font-size: 18px;
          margin-bottom: 12px;
        }

        section ul {
          margin: 0;
          padding-left: 24px;
        }

        section li {
          margin-bottom: 8px;
          line-height: 1.5;
        }

        .details-footer {
          display: flex;
          justify-content: flex-end;
          gap: 12px;
          margin-top: 32px;
          padding-top: 24px;
          border-top: 1px solid #e0e0e0;
        }
      `}</style>
    </Modal>
  );
};

export default IntegrationHub;
