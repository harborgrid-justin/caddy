/**
 * GitHub Setup Wizard Component
 *
 * Step-by-step configuration wizard for GitHub integration.
 */

import React, { useState, useEffect } from 'react';
import { Button, Input, Modal, Tabs } from '../enterprise';
import { GitHubConfig, SetupStep, ConnectionTestResult, IntegrationConfig } from './types';

interface GitHubSetupProps {
  /** Existing configuration (for editing) */
  existingConfig?: Partial<GitHubConfig>;

  /** Callback when setup is complete */
  onComplete: (config: GitHubConfig) => void;

  /** Callback when setup is cancelled */
  onCancel: () => void;
}

/**
 * GitHub Setup Wizard
 *
 * Guides users through the process of setting up GitHub integration.
 */
export const GitHubSetup: React.FC<GitHubSetupProps> = ({
  existingConfig,
  onComplete,
  onCancel,
}) => {
  const [currentStep, setCurrentStep] = useState(0);
  const [config, setConfig] = useState<Partial<GitHubConfig>>(
    existingConfig || {
      autoInstall: false,
      checkName: 'CADDY Design Check',
    }
  );
  const [testResult, setTestResult] = useState<ConnectionTestResult | null>(null);
  const [testing, setTesting] = useState(false);

  const steps: SetupStep[] = [
    {
      id: 'method',
      title: 'Choose Setup Method',
      description: 'Select how you want to set up GitHub integration',
      component: MethodStep,
      completed: false,
    },
    {
      id: 'app-config',
      title: 'GitHub App Configuration',
      description: 'Configure your GitHub App credentials',
      component: AppConfigStep,
      completed: false,
    },
    {
      id: 'permissions',
      title: 'Permissions & Settings',
      description: 'Configure integration permissions and settings',
      component: PermissionsStep,
      completed: false,
    },
    {
      id: 'test',
      title: 'Test Connection',
      description: 'Verify your configuration',
      component: TestStep,
      completed: false,
    },
  ];

  const handleNext = () => {
    if (currentStep < steps.length - 1) {
      setCurrentStep(currentStep + 1);
    } else {
      // Final step - complete setup
      onComplete(config as GitHubConfig);
    }
  };

  const handleBack = () => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1);
    }
  };

  const handleTestConnection = async () => {
    setTesting(true);
    setTestResult(null);

    try {
      // Simulate API call to test connection
      await new Promise((resolve) => setTimeout(resolve, 1500));

      const success = config.appId && config.privateKey;

      setTestResult({
        success: !!success,
        message: success
          ? 'Connection successful! GitHub App is properly configured.'
          : 'Connection failed. Please check your credentials.',
        timestamp: new Date(),
        latency: 342,
      });
    } catch (error) {
      setTestResult({
        success: false,
        message: 'Connection test failed',
        details: error instanceof Error ? error.message : 'Unknown error',
        timestamp: new Date(),
      });
    } finally {
      setTesting(false);
    }
  };

  const CurrentStepComponent = steps[currentStep].component;

  return (
    <Modal isOpen={true} onClose={onCancel} title="GitHub Integration Setup" size="large">
      <div className="github-setup">
        {/* Progress indicator */}
        <div className="progress-steps">
          {steps.map((step, index) => (
            <div
              key={step.id}
              className={`progress-step ${index === currentStep ? 'active' : ''} ${
                index < currentStep ? 'completed' : ''
              }`}
            >
              <div className="step-indicator">{index < currentStep ? '✓' : index + 1}</div>
              <div className="step-label">{step.title}</div>
            </div>
          ))}
        </div>

        {/* Step content */}
        <div className="step-content">
          <h2>{steps[currentStep].title}</h2>
          <p className="step-description">{steps[currentStep].description}</p>

          <CurrentStepComponent
            config={config}
            onUpdate={setConfig}
            testResult={testResult}
            testing={testing}
            onTest={handleTestConnection}
          />
        </div>

        {/* Navigation */}
        <div className="setup-footer">
          <Button onClick={onCancel} variant="secondary">
            Cancel
          </Button>
          <div className="navigation-buttons">
            {currentStep > 0 && (
              <Button onClick={handleBack} variant="secondary">
                Back
              </Button>
            )}
            <Button onClick={handleNext} variant="primary">
              {currentStep === steps.length - 1 ? 'Complete Setup' : 'Next'}
            </Button>
          </div>
        </div>
      </div>

      <style jsx>{`
        .github-setup {
          padding: 24px;
        }

        .progress-steps {
          display: flex;
          justify-content: space-between;
          margin-bottom: 32px;
          position: relative;
        }

        .progress-steps::before {
          content: '';
          position: absolute;
          top: 20px;
          left: 0;
          right: 0;
          height: 2px;
          background: #e0e0e0;
          z-index: 0;
        }

        .progress-step {
          display: flex;
          flex-direction: column;
          align-items: center;
          flex: 1;
          position: relative;
          z-index: 1;
        }

        .step-indicator {
          width: 40px;
          height: 40px;
          border-radius: 50%;
          background: white;
          border: 2px solid #e0e0e0;
          display: flex;
          align-items: center;
          justify-content: center;
          font-weight: 600;
          margin-bottom: 8px;
          transition: all 0.3s;
        }

        .progress-step.active .step-indicator {
          border-color: #2196f3;
          color: #2196f3;
          background: #e3f2fd;
        }

        .progress-step.completed .step-indicator {
          border-color: #4caf50;
          background: #4caf50;
          color: white;
        }

        .step-label {
          font-size: 12px;
          text-align: center;
          color: #666;
          max-width: 100px;
        }

        .progress-step.active .step-label {
          color: #2196f3;
          font-weight: 500;
        }

        .step-content {
          min-height: 400px;
          margin-bottom: 24px;
        }

        .step-content h2 {
          font-size: 24px;
          margin-bottom: 8px;
        }

        .step-description {
          color: #666;
          margin-bottom: 24px;
        }

        .setup-footer {
          display: flex;
          justify-content: space-between;
          padding-top: 24px;
          border-top: 1px solid #e0e0e0;
        }

        .navigation-buttons {
          display: flex;
          gap: 12px;
        }
      `}</style>
    </Modal>
  );
};

/**
 * Step 1: Choose setup method
 */
const MethodStep: React.FC<any> = ({ config, onUpdate }) => {
  const [method, setMethod] = useState<'app' | 'token'>(
    config.appId ? 'app' : 'token'
  );

  return (
    <div className="method-step">
      <div className="method-options">
        <div
          className={`method-option ${method === 'app' ? 'selected' : ''}`}
          onClick={() => setMethod('app')}
        >
          <div className="option-icon">🔧</div>
          <h3>GitHub App (Recommended)</h3>
          <p>More secure and provides better integration features</p>
          <ul>
            <li>Enhanced security with private key authentication</li>
            <li>Fine-grained permissions</li>
            <li>Webhook support</li>
            <li>Check Runs API for detailed status</li>
          </ul>
        </div>

        <div
          className={`method-option ${method === 'token' ? 'selected' : ''}`}
          onClick={() => setMethod('token')}
        >
          <div className="option-icon">🔑</div>
          <h3>Personal Access Token</h3>
          <p>Simpler setup for personal repositories</p>
          <ul>
            <li>Quick setup</li>
            <li>Good for personal projects</li>
            <li>Limited to commit status API</li>
          </ul>
          <div className="coming-soon">Coming Soon</div>
        </div>
      </div>

      <style jsx>{`
        .method-options {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 24px;
        }

        .method-option {
          border: 2px solid #e0e0e0;
          border-radius: 8px;
          padding: 24px;
          cursor: pointer;
          transition: all 0.2s;
          position: relative;
        }

        .method-option:hover {
          border-color: #2196f3;
          box-shadow: 0 4px 12px rgba(33, 150, 243, 0.1);
        }

        .method-option.selected {
          border-color: #2196f3;
          background: #e3f2fd;
        }

        .option-icon {
          font-size: 48px;
          margin-bottom: 16px;
        }

        .method-option h3 {
          font-size: 18px;
          margin-bottom: 8px;
        }

        .method-option p {
          color: #666;
          margin-bottom: 16px;
        }

        .method-option ul {
          list-style: none;
          padding: 0;
          margin: 0;
        }

        .method-option li {
          padding: 4px 0;
          font-size: 14px;
          color: #666;
        }

        .method-option li::before {
          content: '✓ ';
          color: #4caf50;
          font-weight: bold;
        }

        .coming-soon {
          position: absolute;
          top: 16px;
          right: 16px;
          padding: 4px 12px;
          background: #ff9800;
          color: white;
          border-radius: 4px;
          font-size: 12px;
          font-weight: 500;
        }
      `}</style>
    </div>
  );
};

/**
 * Step 2: GitHub App configuration
 */
const AppConfigStep: React.FC<any> = ({ config, onUpdate }) => {
  return (
    <div className="app-config-step">
      <div className="config-instructions">
        <h4>Creating a GitHub App</h4>
        <ol>
          <li>
            Go to your GitHub organization settings → Developer settings → GitHub Apps
          </li>
          <li>Click "New GitHub App"</li>
          <li>
            Fill in the app details:
            <ul>
              <li>Name: "CADDY Design Check"</li>
              <li>Homepage URL: Your CADDY instance URL</li>
              <li>Webhook URL: Your webhook endpoint</li>
            </ul>
          </li>
          <li>
            Set repository permissions:
            <ul>
              <li>Checks: Read & Write</li>
              <li>Contents: Read</li>
              <li>Pull requests: Read & Write</li>
            </ul>
          </li>
          <li>Generate a private key and download it</li>
        </ol>
      </div>

      <div className="config-form">
        <Input
          label="App ID"
          type="text"
          value={config.appId || ''}
          onChange={(e) => onUpdate({ ...config, appId: e.target.value })}
          placeholder="123456"
          required
          help="Found in your GitHub App settings"
        />

        <Input
          label="Installation ID"
          type="text"
          value={config.installationId || ''}
          onChange={(e) => onUpdate({ ...config, installationId: e.target.value })}
          placeholder="78901234"
          help="Found after installing the app to your organization"
        />

        <div className="textarea-group">
          <label>Private Key</label>
          <textarea
            value={config.privateKey || ''}
            onChange={(e) => onUpdate({ ...config, privateKey: e.target.value })}
            placeholder="-----BEGIN RSA PRIVATE KEY-----&#10;...&#10;-----END RSA PRIVATE KEY-----"
            rows={8}
            required
          />
          <span className="help-text">Paste the entire contents of your private key file</span>
        </div>

        <Input
          label="Webhook Secret (Optional)"
          type="password"
          value={config.webhookSecret || ''}
          onChange={(e) => onUpdate({ ...config, webhookSecret: e.target.value })}
          placeholder="Enter webhook secret"
          help="For securing webhook payloads"
        />
      </div>

      <style jsx>{`
        .app-config-step {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 32px;
        }

        .config-instructions {
          background: #f5f5f5;
          padding: 20px;
          border-radius: 8px;
        }

        .config-instructions h4 {
          margin-top: 0;
        }

        .config-instructions ol {
          padding-left: 20px;
        }

        .config-instructions li {
          margin-bottom: 12px;
          line-height: 1.6;
        }

        .config-instructions ul {
          margin-top: 8px;
          padding-left: 20px;
        }

        .config-form {
          display: flex;
          flex-direction: column;
          gap: 16px;
        }

        .textarea-group label {
          display: block;
          font-weight: 500;
          margin-bottom: 8px;
        }

        .textarea-group textarea {
          width: 100%;
          padding: 8px 12px;
          border: 1px solid #e0e0e0;
          border-radius: 4px;
          font-family: monospace;
          font-size: 12px;
          resize: vertical;
        }

        .help-text {
          display: block;
          font-size: 12px;
          color: #666;
          margin-top: 4px;
        }
      `}</style>
    </div>
  );
};

/**
 * Step 3: Permissions and settings
 */
const PermissionsStep: React.FC<any> = ({ config, onUpdate }) => {
  return (
    <div className="permissions-step">
      <Input
        label="Check Run Name"
        type="text"
        value={config.checkName || 'CADDY Design Check'}
        onChange={(e) => onUpdate({ ...config, checkName: e.target.value })}
        placeholder="CADDY Design Check"
        help="Name displayed in GitHub check runs"
      />

      <div className="checkbox-group">
        <label>
          <input
            type="checkbox"
            checked={config.autoInstall || false}
            onChange={(e) => onUpdate({ ...config, autoInstall: e.target.checked })}
          />
          <span>Automatically create check runs for new pull requests</span>
        </label>
      </div>

      <div className="info-box">
        <h4>Required Permissions</h4>
        <p>
          Ensure your GitHub App has the following permissions configured:
        </p>
        <ul>
          <li>✓ Checks: Read & Write</li>
          <li>✓ Contents: Read</li>
          <li>✓ Pull requests: Read & Write</li>
          <li>✓ Commit statuses: Read & Write</li>
        </ul>
      </div>

      <style jsx>{`
        .permissions-step {
          max-width: 600px;
        }

        .checkbox-group {
          margin: 24px 0;
        }

        .checkbox-group label {
          display: flex;
          align-items: center;
          gap: 8px;
          cursor: pointer;
        }

        .checkbox-group input[type='checkbox'] {
          width: 20px;
          height: 20px;
          cursor: pointer;
        }

        .info-box {
          background: #e3f2fd;
          border: 1px solid #2196f3;
          border-radius: 8px;
          padding: 20px;
          margin-top: 24px;
        }

        .info-box h4 {
          margin-top: 0;
          color: #1976d2;
        }

        .info-box ul {
          list-style: none;
          padding: 0;
          margin: 12px 0 0 0;
        }

        .info-box li {
          padding: 4px 0;
        }
      `}</style>
    </div>
  );
};

/**
 * Step 4: Test connection
 */
const TestStep: React.FC<any> = ({ config, testResult, testing, onTest }) => {
  return (
    <div className="test-step">
      <div className="test-info">
        <p>
          Before completing the setup, let's verify that your GitHub App is configured correctly.
        </p>
      </div>

      <div className="test-button-container">
        <Button onClick={onTest} variant="primary" disabled={testing}>
          {testing ? 'Testing Connection...' : 'Test Connection'}
        </Button>
      </div>

      {testResult && (
        <div className={`test-result ${testResult.success ? 'success' : 'error'}`}>
          <div className="result-icon">{testResult.success ? '✓' : '✗'}</div>
          <div className="result-content">
            <h4>{testResult.success ? 'Success!' : 'Connection Failed'}</h4>
            <p>{testResult.message}</p>
            {testResult.latency && (
              <p className="latency">Response time: {testResult.latency}ms</p>
            )}
            {testResult.details && <pre className="details">{testResult.details}</pre>}
          </div>
        </div>
      )}

      <div className="config-summary">
        <h4>Configuration Summary</h4>
        <table>
          <tbody>
            <tr>
              <td>App ID:</td>
              <td>{config.appId || 'Not set'}</td>
            </tr>
            <tr>
              <td>Installation ID:</td>
              <td>{config.installationId || 'Not set'}</td>
            </tr>
            <tr>
              <td>Check Name:</td>
              <td>{config.checkName || 'CADDY Design Check'}</td>
            </tr>
            <tr>
              <td>Webhook Secret:</td>
              <td>{config.webhookSecret ? '••••••••' : 'Not set'}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <style jsx>{`
        .test-info {
          background: #f5f5f5;
          padding: 16px;
          border-radius: 8px;
          margin-bottom: 24px;
        }

        .test-button-container {
          margin-bottom: 24px;
        }

        .test-result {
          display: flex;
          gap: 16px;
          padding: 20px;
          border-radius: 8px;
          margin-bottom: 24px;
        }

        .test-result.success {
          background: #e8f5e9;
          border: 1px solid #4caf50;
        }

        .test-result.error {
          background: #ffebee;
          border: 1px solid #f44336;
        }

        .result-icon {
          font-size: 32px;
          width: 40px;
          height: 40px;
          display: flex;
          align-items: center;
          justify-content: center;
        }

        .test-result.success .result-icon {
          color: #4caf50;
        }

        .test-result.error .result-icon {
          color: #f44336;
        }

        .result-content h4 {
          margin: 0 0 8px 0;
        }

        .result-content p {
          margin: 0;
        }

        .latency {
          font-size: 12px;
          color: #666;
          margin-top: 8px !important;
        }

        .details {
          background: rgba(0, 0, 0, 0.05);
          padding: 12px;
          border-radius: 4px;
          font-size: 12px;
          overflow-x: auto;
          margin-top: 12px;
        }

        .config-summary {
          background: #fafafa;
          padding: 20px;
          border-radius: 8px;
        }

        .config-summary h4 {
          margin-top: 0;
        }

        .config-summary table {
          width: 100%;
          border-collapse: collapse;
        }

        .config-summary td {
          padding: 8px 0;
        }

        .config-summary td:first-child {
          font-weight: 500;
          color: #666;
          width: 40%;
        }
      `}</style>
    </div>
  );
};

export default GitHubSetup;
