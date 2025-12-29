/**
 * MFA Setup Component
 * Multi-Factor Authentication enrollment wizard
 */

import React, { useState, useEffect } from 'react';
import { Button, Input, Modal, Tabs } from '../enterprise';
import type {
  MfaMethod,
  MfaEnrollment,
  TotpConfig,
  MFASetupProps,
  BackupCode,
} from './types';
import QRCode from 'qrcode';

const MFA_METHODS: Array<{ value: MfaMethod; label: string; description: string; icon: string }> = [
  {
    value: 'totp',
    label: 'Authenticator App',
    description: 'Use Google Authenticator, Authy, or similar app',
    icon: '📱',
  },
  {
    value: 'sms',
    label: 'SMS',
    description: 'Receive codes via text message',
    icon: '💬',
  },
  {
    value: 'email',
    label: 'Email',
    description: 'Receive codes via email',
    icon: '📧',
  },
  {
    value: 'fido2',
    label: 'Security Key',
    description: 'Use a hardware security key (YubiKey, etc.)',
    icon: '🔑',
  },
];

export const MFASetup: React.FC<MFASetupProps> = ({
  userId,
  onComplete,
  onCancel,
  availableMethods = ['totp', 'sms', 'email', 'fido2'],
}) => {
  const [selectedMethod, setSelectedMethod] = useState<MfaMethod | null>(null);
  const [currentStep, setCurrentStep] = useState<'select' | 'configure' | 'verify' | 'backup'>('select');
  const [enrollments, setEnrollments] = useState<MfaEnrollment[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadEnrollments();
  }, [userId]);

  const loadEnrollments = async () => {
    try {
      const response = await fetch(`/api/mfa/enrollments?user_id=${userId}`);
      const data = await response.json();
      setEnrollments(data.enrollments || []);
    } catch (error) {
      console.error('Failed to load enrollments:', error);
    }
  };

  const handleMethodSelect = (method: MfaMethod) => {
    setSelectedMethod(method);
    setCurrentStep('configure');
  };

  const handleBack = () => {
    if (currentStep === 'configure') {
      setCurrentStep('select');
      setSelectedMethod(null);
    } else if (currentStep === 'verify') {
      setCurrentStep('configure');
    } else if (currentStep === 'backup') {
      setCurrentStep('verify');
    }
  };

  const renderMethodSelection = () => (
    <div className="space-y-4">
      <div className="text-center mb-6">
        <h3 className="text-xl font-bold mb-2">Choose Authentication Method</h3>
        <p className="text-gray-600">
          Select how you'd like to verify your identity
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {MFA_METHODS.filter((m) => availableMethods.includes(m.value)).map(
          (method) => {
            const isEnrolled = enrollments.some(
              (e) => e.method === method.value && e.enabled
            );

            return (
              <button
                key={method.value}
                onClick={() => !isEnrolled && handleMethodSelect(method.value)}
                disabled={isEnrolled}
                className={`p-6 border-2 rounded-lg text-left transition-all ${
                  isEnrolled
                    ? 'border-green-300 bg-green-50 cursor-not-allowed'
                    : 'border-gray-200 hover:border-blue-500 hover:shadow-md'
                }`}
              >
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center mb-2">
                      <span className="text-3xl mr-3">{method.icon}</span>
                      <h4 className="font-semibold">{method.label}</h4>
                    </div>
                    <p className="text-sm text-gray-600">{method.description}</p>
                  </div>
                  {isEnrolled && (
                    <span className="text-green-600 text-sm font-medium">
                      ✓ Enrolled
                    </span>
                  )}
                </div>
              </button>
            );
          }
        )}
      </div>

      {enrollments.length > 0 && (
        <div className="mt-6 p-4 bg-blue-50 rounded">
          <p className="text-sm text-blue-800">
            <strong>Tip:</strong> For maximum security, consider enabling multiple
            MFA methods as backup options.
          </p>
        </div>
      )}
    </div>
  );

  return (
    <div className="max-w-2xl mx-auto p-6">
      <div className="bg-white rounded-lg shadow-lg p-6">
        {currentStep === 'select' && renderMethodSelection()}

        {currentStep !== 'select' && selectedMethod === 'totp' && (
          <TOTPSetup
            userId={userId}
            onComplete={(enrollment) => {
              setCurrentStep('backup');
            }}
            onBack={handleBack}
          />
        )}

        {currentStep !== 'select' && selectedMethod === 'sms' && (
          <SMSSetup
            userId={userId}
            onComplete={onComplete}
            onBack={handleBack}
          />
        )}

        {currentStep !== 'select' && selectedMethod === 'email' && (
          <EmailSetup
            userId={userId}
            onComplete={onComplete}
            onBack={handleBack}
          />
        )}

        {currentStep !== 'select' && selectedMethod === 'fido2' && (
          <FIDO2Setup
            userId={userId}
            onComplete={onComplete}
            onBack={handleBack}
          />
        )}

        {currentStep === 'backup' && (
          <BackupCodesDisplay
            userId={userId}
            onComplete={onComplete}
            onBack={handleBack}
          />
        )}

        {currentStep === 'select' && onCancel && (
          <div className="flex justify-end mt-6">
            <Button variant="ghost" onClick={onCancel}>
              Cancel
            </Button>
          </div>
        )}
      </div>
    </div>
  );
};

// TOTP Setup Component
interface SetupComponentProps {
  userId: string;
  onComplete: (enrollment?: MfaEnrollment) => void;
  onBack: () => void;
}

const TOTPSetup: React.FC<SetupComponentProps> = ({ userId, onComplete, onBack }) => {
  const [totpConfig, setTotpConfig] = useState<TotpConfig | null>(null);
  const [qrCodeUrl, setQrCodeUrl] = useState<string>('');
  const [verificationCode, setVerificationCode] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    enrollTotp();
  }, []);

  const enrollTotp = async () => {
    setLoading(true);
    try {
      const response = await fetch('/api/mfa/enroll/totp', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ user_id: userId }),
      });

      const data = await response.json();
      setTotpConfig(data.config);

      // Generate QR code
      const qrUrl = await QRCode.toDataURL(data.config.provisioning_url);
      setQrCodeUrl(qrUrl);
    } catch (err) {
      setError('Failed to initialize TOTP setup');
    } finally {
      setLoading(false);
    }
  };

  const handleVerify = async () => {
    if (verificationCode.length !== 6) {
      setError('Please enter a 6-digit code');
      return;
    }

    setLoading(true);
    setError('');

    try {
      const response = await fetch('/api/mfa/verify/totp', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          user_id: userId,
          code: verificationCode,
        }),
      });

      const result = await response.json();

      if (result.success) {
        onComplete();
      } else {
        setError(result.error || 'Invalid code. Please try again.');
      }
    } catch (err) {
      setError('Verification failed. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="text-center">
        <h3 className="text-xl font-bold mb-2">Set Up Authenticator App</h3>
        <p className="text-gray-600">Scan the QR code with your authenticator app</p>
      </div>

      {loading && <div className="text-center">Loading...</div>}

      {!loading && totpConfig && (
        <>
          <div className="flex justify-center">
            <div className="p-4 bg-white border-2 rounded-lg">
              <img src={qrCodeUrl} alt="QR Code" className="w-48 h-48" />
            </div>
          </div>

          <div className="text-center">
            <p className="text-sm text-gray-600 mb-2">
              Can't scan? Enter this code manually:
            </p>
            <code className="bg-gray-100 px-3 py-1 rounded font-mono text-sm">
              {totpConfig.secret}
            </code>
          </div>

          <div className="border-t pt-4">
            <Input
              label="Verification Code"
              value={verificationCode}
              onChange={(e) => setVerificationCode(e.target.value.replace(/\D/g, ''))}
              placeholder="Enter 6-digit code"
              maxLength={6}
              error={error}
              helperText="Enter the code from your authenticator app"
            />
          </div>

          <div className="flex justify-between">
            <Button variant="secondary" onClick={onBack}>
              Back
            </Button>
            <Button
              onClick={handleVerify}
              disabled={verificationCode.length !== 6 || loading}
              loading={loading}
            >
              Verify and Continue
            </Button>
          </div>
        </>
      )}
    </div>
  );
};

// SMS Setup Component
const SMSSetup: React.FC<SetupComponentProps> = ({ userId, onComplete, onBack }) => {
  const [phoneNumber, setPhoneNumber] = useState('');
  const [countryCode, setCountryCode] = useState('+1');
  const [verificationCode, setVerificationCode] = useState('');
  const [codeSent, setCodeSent] = useState(false);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleSendCode = async () => {
    if (!phoneNumber.trim()) {
      setError('Please enter your phone number');
      return;
    }

    setLoading(true);
    setError('');

    try {
      const response = await fetch('/api/mfa/enroll/sms', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          user_id: userId,
          phone_number: phoneNumber,
          country_code: countryCode,
        }),
      });

      if (response.ok) {
        setCodeSent(true);
      } else {
        setError('Failed to send verification code');
      }
    } catch (err) {
      setError('Failed to send verification code');
    } finally {
      setLoading(false);
    }
  };

  const handleVerify = async () => {
    setLoading(true);
    setError('');

    try {
      const response = await fetch('/api/mfa/verify/sms', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          user_id: userId,
          code: verificationCode,
        }),
      });

      const result = await response.json();

      if (result.success) {
        onComplete();
      } else {
        setError(result.error || 'Invalid code');
      }
    } catch (err) {
      setError('Verification failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="text-center">
        <h3 className="text-xl font-bold mb-2">Set Up SMS Verification</h3>
        <p className="text-gray-600">
          {codeSent
            ? 'Enter the code sent to your phone'
            : 'Enter your phone number to receive verification codes'}
        </p>
      </div>

      {!codeSent ? (
        <>
          <div className="flex space-x-2">
            <select
              className="w-24 p-2 border rounded"
              value={countryCode}
              onChange={(e) => setCountryCode(e.target.value)}
            >
              <option value="+1">+1 (US)</option>
              <option value="+44">+44 (UK)</option>
              <option value="+86">+86 (CN)</option>
              {/* Add more country codes */}
            </select>
            <Input
              value={phoneNumber}
              onChange={(e) => setPhoneNumber(e.target.value)}
              placeholder="Phone number"
              error={error}
            />
          </div>

          <div className="flex justify-between">
            <Button variant="secondary" onClick={onBack}>
              Back
            </Button>
            <Button onClick={handleSendCode} loading={loading}>
              Send Code
            </Button>
          </div>
        </>
      ) : (
        <>
          <Input
            label="Verification Code"
            value={verificationCode}
            onChange={(e) => setVerificationCode(e.target.value)}
            placeholder="Enter code"
            error={error}
          />

          <div className="flex justify-between">
            <Button variant="secondary" onClick={() => setCodeSent(false)}>
              Change Number
            </Button>
            <Button onClick={handleVerify} loading={loading}>
              Verify
            </Button>
          </div>
        </>
      )}
    </div>
  );
};

// Email Setup Component
const EmailSetup: React.FC<SetupComponentProps> = ({ userId, onComplete, onBack }) => {
  const [email, setEmail] = useState('');
  const [verificationCode, setVerificationCode] = useState('');
  const [codeSent, setCodeSent] = useState(false);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleSendCode = async () => {
    setLoading(true);
    try {
      await fetch('/api/mfa/enroll/email', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ user_id: userId, email }),
      });
      setCodeSent(true);
    } catch (err) {
      setError('Failed to send code');
    } finally {
      setLoading(false);
    }
  };

  const handleVerify = async () => {
    setLoading(true);
    try {
      const response = await fetch('/api/mfa/verify/email', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ user_id: userId, code: verificationCode }),
      });

      const result = await response.json();
      if (result.success) {
        onComplete();
      } else {
        setError('Invalid code');
      }
    } catch (err) {
      setError('Verification failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="text-center">
        <h3 className="text-xl font-bold mb-2">Set Up Email Verification</h3>
      </div>

      {!codeSent ? (
        <>
          <Input
            label="Email Address"
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            error={error}
          />
          <div className="flex justify-between">
            <Button variant="secondary" onClick={onBack}>Back</Button>
            <Button onClick={handleSendCode} loading={loading}>Send Code</Button>
          </div>
        </>
      ) : (
        <>
          <Input
            label="Verification Code"
            value={verificationCode}
            onChange={(e) => setVerificationCode(e.target.value)}
            error={error}
          />
          <div className="flex justify-between">
            <Button variant="secondary" onClick={onBack}>Back</Button>
            <Button onClick={handleVerify} loading={loading}>Verify</Button>
          </div>
        </>
      )}
    </div>
  );
};

// FIDO2 Setup Component
const FIDO2Setup: React.FC<SetupComponentProps> = ({ userId, onComplete, onBack }) => {
  const [deviceName, setDeviceName] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleRegister = async () => {
    setLoading(true);
    try {
      // In production, use WebAuthn API
      // const publicKey = await navigator.credentials.create({ publicKey: options });
      onComplete();
    } catch (err) {
      setError('Failed to register security key');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="text-center">
        <h3 className="text-xl font-bold mb-2">Register Security Key</h3>
        <p className="text-gray-600">Insert your security key and follow the prompts</p>
      </div>

      <Input
        label="Device Name"
        value={deviceName}
        onChange={(e) => setDeviceName(e.target.value)}
        placeholder="e.g., YubiKey 5"
        error={error}
      />

      <div className="flex justify-between">
        <Button variant="secondary" onClick={onBack}>Back</Button>
        <Button onClick={handleRegister} loading={loading}>Register Key</Button>
      </div>
    </div>
  );
};

// Backup Codes Display Component
const BackupCodesDisplay: React.FC<SetupComponentProps> = ({ userId, onComplete, onBack }) => {
  const [backupCodes, setBackupCodes] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    generateBackupCodes();
  }, []);

  const generateBackupCodes = async () => {
    try {
      const response = await fetch('/api/mfa/backup-codes', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ user_id: userId, count: 10 }),
      });

      const data = await response.json();
      setBackupCodes(data.codes || []);
    } catch (err) {
      console.error('Failed to generate backup codes');
    } finally {
      setLoading(false);
    }
  };

  const handleDownload = () => {
    const content = backupCodes.join('\n');
    const blob = new Blob([content], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'caddy-backup-codes.txt';
    a.click();
  };

  return (
    <div className="space-y-6">
      <div className="text-center">
        <h3 className="text-xl font-bold mb-2">Save Your Backup Codes</h3>
        <p className="text-gray-600">
          Store these codes safely. Each code can only be used once.
        </p>
      </div>

      {loading ? (
        <div className="text-center">Generating codes...</div>
      ) : (
        <>
          <div className="bg-gray-50 p-4 rounded border-2 border-yellow-300">
            <div className="grid grid-cols-2 gap-2 font-mono text-sm">
              {backupCodes.map((code, index) => (
                <div key={index} className="p-2 bg-white rounded">
                  {code}
                </div>
              ))}
            </div>
          </div>

          <div className="bg-yellow-50 border border-yellow-200 p-4 rounded">
            <p className="text-sm text-yellow-800">
              <strong>⚠ Important:</strong> Save these codes in a secure location.
              If you lose access to your authentication method, you'll need these
              codes to regain access to your account.
            </p>
          </div>

          <div className="flex justify-between">
            <Button variant="secondary" onClick={handleDownload}>
              Download Codes
            </Button>
            <Button onClick={() => onComplete()}>
              I've Saved My Codes
            </Button>
          </div>
        </>
      )}
    </div>
  );
};
