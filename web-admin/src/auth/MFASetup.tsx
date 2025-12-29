/**
 * MFA Setup Wizard
 *
 * Multi-step wizard for setting up multi-factor authentication
 */

import React, { useState, useEffect } from 'react';
import { useAuth } from './AuthProvider';
import QRCode from 'qrcode';

type SetupStep = 'choose' | 'totp-setup' | 'totp-verify' | 'recovery-codes' | 'complete';

export const MFASetup: React.FC<{ onComplete?: () => void }> = ({ onComplete }) => {
  const { setupMfa, verifyMfa, mfaStatus } = useAuth();

  const [step, setStep] = useState<SetupStep>('choose');
  const [qrCodeUrl, setQrCodeUrl] = useState<string>('');
  const [secret, setSecret] = useState<string>('');
  const [recoveryCodes, setRecoveryCodes] = useState<string[]>([]);
  const [verificationCode, setVerificationCode] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copiedCodes, setCopiedCodes] = useState(false);

  const handleChooseTOTP = async () => {
    setIsLoading(true);
    setError(null);

    try {
      const setupData = await setupMfa();
      setSecret(setupData.secret);

      // Generate QR code
      const qrData = setupData.qrCode;
      const url = await QRCode.toDataURL(qrData);
      setQrCodeUrl(url);

      setStep('totp-setup');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to setup TOTP');
    } finally {
      setIsLoading(false);
    }
  };

  const handleVerifyTOTP = async () => {
    setIsLoading(true);
    setError(null);

    try {
      await verifyMfa(verificationCode);
      // Recovery codes would be returned from the server
      setRecoveryCodes([
        'ABCD-1234', 'EFGH-5678', 'IJKL-9012',
        'MNOP-3456', 'QRST-7890', 'UVWX-1234',
        'YZAB-5678', 'CDEF-9012', 'GHIJ-3456',
        'KLMN-7890'
      ]);
      setStep('recovery-codes');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Invalid verification code');
    } finally {
      setIsLoading(false);
    }
  };

  const handleCopyRecoveryCodes = () => {
    const text = recoveryCodes.join('\n');
    navigator.clipboard.writeText(text);
    setCopiedCodes(true);
    setTimeout(() => setCopiedCodes(false), 2000);
  };

  const handleDownloadRecoveryCodes = () => {
    const text = recoveryCodes.join('\n');
    const blob = new Blob([text], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'caddy-recovery-codes.txt';
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleComplete = () => {
    setStep('complete');
    setTimeout(() => {
      onComplete?.();
    }, 2000);
  };

  return (
    <div className="max-w-2xl mx-auto p-6">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
        {/* Header */}
        <div className="mb-6">
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
            Multi-Factor Authentication Setup
          </h2>
          <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
            Add an extra layer of security to your account
          </p>
        </div>

        {/* Error Message */}
        {error && (
          <div className="mb-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-red-700 dark:text-red-400 px-4 py-3 rounded">
            <p className="text-sm">{error}</p>
          </div>
        )}

        {/* Step: Choose Method */}
        {step === 'choose' && (
          <div className="space-y-4">
            <h3 className="text-lg font-medium text-gray-900 dark:text-white">
              Choose Authentication Method
            </h3>

            <button
              onClick={handleChooseTOTP}
              disabled={isLoading}
              className="w-full p-4 border-2 border-gray-300 dark:border-gray-600 rounded-lg hover:border-blue-500 transition-colors text-left"
            >
              <div className="flex items-start">
                <div className="text-3xl mr-4">📱</div>
                <div>
                  <h4 className="font-medium text-gray-900 dark:text-white">
                    Authenticator App (TOTP)
                  </h4>
                  <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                    Use an app like Google Authenticator, Authy, or 1Password
                  </p>
                </div>
              </div>
            </button>

            <button
              onClick={() => {/* Handle WebAuthn setup */}}
              disabled={true}
              className="w-full p-4 border-2 border-gray-300 dark:border-gray-600 rounded-lg opacity-50 cursor-not-allowed text-left"
            >
              <div className="flex items-start">
                <div className="text-3xl mr-4">🔐</div>
                <div>
                  <h4 className="font-medium text-gray-900 dark:text-white">
                    Security Key (WebAuthn)
                  </h4>
                  <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                    Use a hardware security key like YubiKey (Coming Soon)
                  </p>
                </div>
              </div>
            </button>
          </div>
        )}

        {/* Step: TOTP Setup */}
        {step === 'totp-setup' && (
          <div className="space-y-6">
            <h3 className="text-lg font-medium text-gray-900 dark:text-white">
              Scan QR Code
            </h3>

            <div className="flex flex-col items-center space-y-4">
              {qrCodeUrl && (
                <img src={qrCodeUrl} alt="QR Code" className="w-64 h-64 border-2 border-gray-200 dark:border-gray-700 rounded" />
              )}

              <div className="w-full">
                <p className="text-sm text-gray-600 dark:text-gray-400 mb-2">
                  Can't scan? Enter this code manually:
                </p>
                <div className="bg-gray-100 dark:bg-gray-700 p-3 rounded font-mono text-center">
                  {secret}
                </div>
              </div>
            </div>

            <button
              onClick={() => setStep('totp-verify')}
              className="w-full py-2 px-4 bg-blue-600 hover:bg-blue-700 text-white rounded-md font-medium"
            >
              Next
            </button>
          </div>
        )}

        {/* Step: TOTP Verification */}
        {step === 'totp-verify' && (
          <div className="space-y-6">
            <h3 className="text-lg font-medium text-gray-900 dark:text-white">
              Verify Authenticator
            </h3>

            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Enter the 6-digit code from your authenticator app
              </label>
              <input
                type="text"
                maxLength={6}
                value={verificationCode}
                onChange={(e) => setVerificationCode(e.target.value.replace(/\D/g, ''))}
                className="w-full px-4 py-3 text-center text-2xl font-mono border border-gray-300 dark:border-gray-600 rounded-md focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
                placeholder="000000"
                autoFocus
              />
            </div>

            <div className="flex space-x-3">
              <button
                onClick={() => setStep('totp-setup')}
                className="flex-1 py-2 px-4 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-900 dark:text-white rounded-md font-medium"
              >
                Back
              </button>
              <button
                onClick={handleVerifyTOTP}
                disabled={verificationCode.length !== 6 || isLoading}
                className="flex-1 py-2 px-4 bg-blue-600 hover:bg-blue-700 text-white rounded-md font-medium disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isLoading ? 'Verifying...' : 'Verify'}
              </button>
            </div>
          </div>
        )}

        {/* Step: Recovery Codes */}
        {step === 'recovery-codes' && (
          <div className="space-y-6">
            <div>
              <h3 className="text-lg font-medium text-gray-900 dark:text-white">
                Save Recovery Codes
              </h3>
              <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
                Store these codes in a safe place. Each code can only be used once.
              </p>
            </div>

            <div className="bg-gray-50 dark:bg-gray-900 p-4 rounded-lg">
              <div className="grid grid-cols-2 gap-2 font-mono text-sm">
                {recoveryCodes.map((code, index) => (
                  <div key={index} className="text-gray-900 dark:text-gray-100">
                    {code}
                  </div>
                ))}
              </div>
            </div>

            <div className="flex space-x-3">
              <button
                onClick={handleCopyRecoveryCodes}
                className="flex-1 py-2 px-4 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-900 dark:text-white rounded-md font-medium"
              >
                {copiedCodes ? '✓ Copied' : '📋 Copy'}
              </button>
              <button
                onClick={handleDownloadRecoveryCodes}
                className="flex-1 py-2 px-4 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-900 dark:text-white rounded-md font-medium"
              >
                💾 Download
              </button>
            </div>

            <button
              onClick={handleComplete}
              className="w-full py-2 px-4 bg-blue-600 hover:bg-blue-700 text-white rounded-md font-medium"
            >
              I've Saved My Codes
            </button>
          </div>
        )}

        {/* Step: Complete */}
        {step === 'complete' && (
          <div className="text-center space-y-4">
            <div className="text-6xl">✅</div>
            <h3 className="text-lg font-medium text-gray-900 dark:text-white">
              MFA Enabled Successfully!
            </h3>
            <p className="text-sm text-gray-600 dark:text-gray-400">
              Your account is now protected with multi-factor authentication
            </p>
          </div>
        )}
      </div>
    </div>
  );
};

export default MFASetup;
