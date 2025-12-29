/**
 * CI Configuration Generator Component
 *
 * Generates CI/CD configuration files for various platforms.
 */

import React, { useState, useMemo } from 'react';
import { Button, Input, Select, Tabs, Modal } from '../enterprise';
import { CIPlatform, CIConfigTemplate, CIConfigVariable } from './types';

interface CIConfigGeneratorProps {
  /** Selected platform */
  platform?: CIPlatform;

  /** Callback when config is generated */
  onGenerate?: (config: string, filename: string) => void;

  /** Show as modal */
  isModal?: boolean;

  /** Modal close handler */
  onClose?: () => void;
}

/**
 * CI Config Generator
 *
 * Interactive tool to generate CI/CD configuration files with customizable options.
 */
export const CIConfigGenerator: React.FC<CIConfigGeneratorProps> = ({
  platform: initialPlatform,
  onGenerate,
  isModal = false,
  onClose,
}) => {
  const [selectedPlatform, setSelectedPlatform] = useState<CIPlatform>(
    initialPlatform || CIPlatform.GitHub
  );
  const [variables, setVariables] = useState<Record<string, string>>({});
  const [selectedTemplate, setSelectedTemplate] = useState<string>('default');

  // Available templates for each platform
  const templates = useMemo(() => getTemplates(), []);

  // Get templates for selected platform
  const platformTemplates = useMemo(
    () => templates.filter((t) => t.platform === selectedPlatform),
    [templates, selectedPlatform]
  );

  // Get current template
  const currentTemplate = useMemo(
    () => platformTemplates.find((t) => t.name === selectedTemplate) || platformTemplates[0],
    [platformTemplates, selectedTemplate]
  );

  // Generate config with variable substitution
  const generatedConfig = useMemo(() => {
    if (!currentTemplate) return '';

    let config = currentTemplate.content;

    // Substitute variables
    currentTemplate.variables.forEach((variable) => {
      const value = variables[variable.key] || variable.defaultValue || '';
      const pattern = new RegExp(`\\{\\{${variable.key}\\}\\}`, 'g');
      config = config.replace(pattern, value);
    });

    return config;
  }, [currentTemplate, variables]);

  // Handle variable change
  const handleVariableChange = (key: string, value: string) => {
    setVariables({ ...variables, [key]: value });
  };

  // Handle download
  const handleDownload = () => {
    const blob = new Blob([generatedConfig], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = currentTemplate?.filename || 'config.yml';
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  // Handle copy
  const handleCopy = () => {
    navigator.clipboard.writeText(generatedConfig);
    alert('Configuration copied to clipboard!');
  };

  const content = (
    <div className="ci-config-generator">
      <div className="generator-header">
        <h2>CI/CD Configuration Generator</h2>
        <p>Generate configuration files for your CI/CD platform</p>
      </div>

      <div className="generator-content">
        <div className="configuration-panel">
          <section>
            <h3>Platform</h3>
            <Select
              value={selectedPlatform}
              onChange={(value) => {
                setSelectedPlatform(value as CIPlatform);
                setSelectedTemplate('default');
                setVariables({});
              }}
              options={[
                { value: CIPlatform.GitHub, label: 'GitHub Actions' },
                { value: CIPlatform.GitLab, label: 'GitLab CI/CD' },
                { value: CIPlatform.Jenkins, label: 'Jenkins Pipeline' },
                { value: CIPlatform.AzureDevOps, label: 'Azure Pipelines' },
                { value: CIPlatform.Bitbucket, label: 'Bitbucket Pipelines' },
              ]}
            />
          </section>

          <section>
            <h3>Template</h3>
            <Select
              value={selectedTemplate}
              onChange={(value) => {
                setSelectedTemplate(value);
                setVariables({});
              }}
              options={platformTemplates.map((t) => ({
                value: t.name,
                label: t.name,
              }))}
            />
            <p className="template-description">{currentTemplate?.description}</p>
          </section>

          {currentTemplate && currentTemplate.variables.length > 0 && (
            <section>
              <h3>Configuration</h3>
              <div className="variables-form">
                {currentTemplate.variables.map((variable) => (
                  <div key={variable.key} className="variable-field">
                    <Input
                      label={variable.description}
                      type={variable.type === 'secret' ? 'password' : 'text'}
                      value={variables[variable.key] || variable.defaultValue || ''}
                      onChange={(e) => handleVariableChange(variable.key, e.target.value)}
                      placeholder={variable.defaultValue}
                      required={variable.required}
                    />
                  </div>
                ))}
              </div>
            </section>
          )}

          <section>
            <h3>File Information</h3>
            <div className="file-info">
              <div className="info-row">
                <span className="label">Filename:</span>
                <span className="value">{currentTemplate?.filename}</span>
              </div>
              <div className="info-row">
                <span className="label">Platform:</span>
                <span className="value">{selectedPlatform}</span>
              </div>
            </div>
          </section>
        </div>

        <div className="preview-panel">
          <div className="preview-header">
            <h3>Preview</h3>
            <div className="preview-actions">
              <Button onClick={handleCopy} variant="secondary" size="small">
                Copy
              </Button>
              <Button onClick={handleDownload} variant="primary" size="small">
                Download
              </Button>
            </div>
          </div>
          <pre className="config-preview">
            <code>{generatedConfig}</code>
          </pre>
        </div>
      </div>

      <style jsx>{`
        .ci-config-generator {
          padding: 24px;
        }

        .generator-header h2 {
          margin: 0 0 8px 0;
          font-size: 24px;
        }

        .generator-header p {
          color: #666;
          margin: 0 0 24px 0;
        }

        .generator-content {
          display: grid;
          grid-template-columns: 400px 1fr;
          gap: 24px;
          min-height: 600px;
        }

        .configuration-panel {
          display: flex;
          flex-direction: column;
          gap: 24px;
        }

        .configuration-panel section {
          background: #fafafa;
          padding: 20px;
          border-radius: 8px;
        }

        .configuration-panel h3 {
          margin: 0 0 12px 0;
          font-size: 16px;
          font-weight: 600;
        }

        .template-description {
          font-size: 14px;
          color: #666;
          margin: 8px 0 0 0;
          line-height: 1.5;
        }

        .variables-form {
          display: flex;
          flex-direction: column;
          gap: 12px;
        }

        .file-info {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .info-row {
          display: flex;
          justify-content: space-between;
          font-size: 14px;
        }

        .info-row .label {
          color: #666;
          font-weight: 500;
        }

        .info-row .value {
          font-family: monospace;
          background: white;
          padding: 2px 8px;
          border-radius: 4px;
        }

        .preview-panel {
          display: flex;
          flex-direction: column;
          background: #1e1e1e;
          border-radius: 8px;
          overflow: hidden;
        }

        .preview-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 16px 20px;
          background: #2d2d2d;
          border-bottom: 1px solid #3d3d3d;
        }

        .preview-header h3 {
          margin: 0;
          font-size: 16px;
          color: #fff;
        }

        .preview-actions {
          display: flex;
          gap: 8px;
        }

        .config-preview {
          flex: 1;
          margin: 0;
          padding: 20px;
          overflow: auto;
          font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
          font-size: 13px;
          line-height: 1.6;
          color: #d4d4d4;
          background: #1e1e1e;
        }

        .config-preview code {
          color: #d4d4d4;
        }

        @media (max-width: 1024px) {
          .generator-content {
            grid-template-columns: 1fr;
          }

          .preview-panel {
            min-height: 400px;
          }
        }
      `}</style>
    </div>
  );

  if (isModal) {
    return (
      <Modal isOpen={true} onClose={onClose || (() => {})} title="" size="xlarge">
        {content}
      </Modal>
    );
  }

  return content;
};

/**
 * Get all available templates
 */
function getTemplates(): CIConfigTemplate[] {
  return [
    // GitHub Actions
    {
      platform: CIPlatform.GitHub,
      name: 'default',
      description: 'Standard GitHub Actions workflow for pull requests and pushes',
      filename: '.github/workflows/caddy-check.yml',
      content: `name: CADDY Design Check

on:
  pull_request:
    branches: [ {{main_branch}} ]
  push:
    branches: [ {{main_branch}} ]

jobs:
  caddy-check:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Run CADDY Design Check
        run: |
          caddy-cli scan --format github-actions
        env:
          GITHUB_TOKEN: \${{ secrets.GITHUB_TOKEN }}

      - name: Upload Results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: caddy-reports
          path: target/caddy-reports/
`,
      variables: [
        {
          key: 'main_branch',
          description: 'Main branch name',
          defaultValue: 'main',
          required: true,
          type: 'string',
        },
      ],
    },

    // GitLab CI/CD
    {
      platform: CIPlatform.GitLab,
      name: 'default',
      description: 'Standard GitLab CI/CD pipeline configuration',
      filename: '.gitlab-ci.yml',
      content: `stages:
  - test

caddy_check:
  stage: test
  image: {{docker_image}}
  script:
    - caddy-cli scan --format gitlab-ci
  artifacts:
    reports:
      junit: target/caddy-reports/junit.xml
    paths:
      - target/caddy-reports/
    expire_in: 1 week
  rules:
    - if: '$CI_PIPELINE_SOURCE == "merge_request_event"'
    - if: '$CI_COMMIT_BRANCH == "{{main_branch}}"'
`,
      variables: [
        {
          key: 'docker_image',
          description: 'Docker image to use',
          defaultValue: 'rust:latest',
          required: true,
          type: 'string',
        },
        {
          key: 'main_branch',
          description: 'Main branch name',
          defaultValue: 'main',
          required: true,
          type: 'string',
        },
      ],
    },

    // Jenkins Pipeline
    {
      platform: CIPlatform.Jenkins,
      name: 'default',
      description: 'Declarative Jenkins Pipeline',
      filename: 'Jenkinsfile',
      content: `pipeline {
    agent any

    stages {
        stage('CADDY Design Check') {
            steps {
                sh 'caddy-cli scan --format jenkins --output target/caddy-reports/junit.xml'
            }
        }
    }

    post {
        always {
            junit 'target/caddy-reports/junit.xml'
            publishHTML([
                allowMissing: false,
                alwaysLinkToLastBuild: true,
                keepAll: true,
                reportDir: 'target/caddy-reports',
                reportFiles: 'report.html',
                reportName: 'CADDY Design Check Report'
            ])
        }
        success {
            echo 'CADDY check passed!'
        }
        failure {
            echo 'CADDY check failed!'
        }
    }
}
`,
      variables: [],
    },

    // Azure Pipelines
    {
      platform: CIPlatform.AzureDevOps,
      name: 'default',
      description: 'Standard Azure Pipelines configuration',
      filename: 'azure-pipelines.yml',
      content: `trigger:
  - {{main_branch}}

pr:
  - {{main_branch}}

pool:
  vmImage: '{{vm_image}}'

steps:
  - script: |
      caddy-cli scan --format azure-devops
    displayName: 'CADDY Design Check'
    continueOnError: false

  - task: PublishTestResults@2
    displayName: 'Publish CADDY Test Results'
    condition: always()
    inputs:
      testResultsFormat: 'JUnit'
      testResultsFiles: 'target/caddy-reports/junit.xml'
      testRunTitle: 'CADDY Design Check'

  - task: PublishBuildArtifacts@1
    displayName: 'Publish CADDY Reports'
    condition: always()
    inputs:
      pathToPublish: 'target/caddy-reports'
      artifactName: 'caddy-reports'
`,
      variables: [
        {
          key: 'main_branch',
          description: 'Main branch name',
          defaultValue: 'main',
          required: true,
          type: 'string',
        },
        {
          key: 'vm_image',
          description: 'VM image to use',
          defaultValue: 'ubuntu-latest',
          required: true,
          type: 'string',
        },
      ],
    },

    // Bitbucket Pipelines
    {
      platform: CIPlatform.Bitbucket,
      name: 'default',
      description: 'Standard Bitbucket Pipelines configuration',
      filename: 'bitbucket-pipelines.yml',
      content: `image: {{docker_image}}

pipelines:
  default:
    - step:
        name: CADDY Design Check
        script:
          - caddy-cli scan --format bitbucket
        after-script:
          - pipe: atlassian/bitbucket-upload-file:0.3.2
            variables:
              FILENAME: 'target/caddy-reports/report.html'
              ARTIFACT_NAME: 'caddy-report'

  pull-requests:
    '**':
      - step:
          name: CADDY PR Check
          script:
            - caddy-cli scan --format bitbucket --pr $BITBUCKET_PR_ID
`,
      variables: [
        {
          key: 'docker_image',
          description: 'Docker image to use',
          defaultValue: 'rust:latest',
          required: true,
          type: 'string',
        },
      ],
    },
  ];
}

export default CIConfigGenerator;
