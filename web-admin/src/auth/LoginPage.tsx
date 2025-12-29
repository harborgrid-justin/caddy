/**
 * Enterprise Login Page
 *
 * Multi-provider authentication login page with SSO support
 */

import React, { useState } from 'react';
import { useAuth } from './AuthProvider';

interface LoginPageProps {
  onSuccess?: () => void;
  enableOAuth?: boolean;
  enableSAML?: boolean;
  oauthProviders?: Array<{ id: string; name: string; icon?: string }>;
  samlProviders?: Array<{ id: string; name: string; icon?: string }>;
}

export const LoginPage: React.FC<LoginPageProps> = ({
  onSuccess,
  enableOAuth = true,
  enableSAML = true,
  oauthProviders = [
    { id: 'google', name: 'Google', icon: '🔷' },
    { id: 'azure', name: 'Microsoft', icon: '🔷' },
    { id: 'okta', name: 'Okta', icon: '🔷' },
  ],
  samlProviders = [
    { id: 'enterprise', name: 'Enterprise SSO', icon: '🔐' },
  ],
}) => {
  const { login, loginWithOAuth, loginWithSAML } = useAuth();

  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showPassword, setShowPassword] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setIsLoading(true);

    try {
      await login(username, password);
      onSuccess?.();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Login failed');
    } finally {
      setIsLoading(false);
    }
  };

  const handleOAuthLogin = async (provider: string) => {
    setError(null);
    setIsLoading(true);

    try {
      await loginWithOAuth(provider);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'OAuth login failed');
      setIsLoading(false);
    }
  };

  const handleSAMLLogin = async (provider: string) => {
    setError(null);
    setIsLoading(true);

    try {
      await loginWithSAML(provider);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'SAML login failed');
      setIsLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-100 dark:bg-gray-900">
      <div className="max-w-md w-full space-y-8 bg-white dark:bg-gray-800 p-8 rounded-lg shadow-lg">
        {/* Header */}
        <div className="text-center">
          <h2 className="text-3xl font-bold text-gray-900 dark:text-white">
            CADDY Enterprise
          </h2>
          <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
            Sign in to your account
          </p>
        </div>

        {/* Error Message */}
        {error && (
          <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-red-700 dark:text-red-400 px-4 py-3 rounded">
            <p className="text-sm">{error}</p>
          </div>
        )}

        {/* Login Form */}
        <form className="mt-8 space-y-6" onSubmit={handleSubmit}>
          <div className="space-y-4">
            <div>
              <label htmlFor="username" className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Username or Email
              </label>
              <input
                id="username"
                name="username"
                type="text"
                required
                autoComplete="username"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                disabled={isLoading}
                className="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
                placeholder="Enter your username"
              />
            </div>

            <div>
              <label htmlFor="password" className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Password
              </label>
              <div className="mt-1 relative">
                <input
                  id="password"
                  name="password"
                  type={showPassword ? 'text' : 'password'}
                  required
                  autoComplete="current-password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  disabled={isLoading}
                  className="block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
                  placeholder="Enter your password"
                />
                <button
                  type="button"
                  onClick={() => setShowPassword(!showPassword)}
                  className="absolute inset-y-0 right-0 pr-3 flex items-center text-gray-400 hover:text-gray-600"
                >
                  {showPassword ? '👁️' : '👁️‍🗨️'}
                </button>
              </div>
            </div>
          </div>

          <div className="flex items-center justify-between">
            <div className="flex items-center">
              <input
                id="remember-me"
                name="remember-me"
                type="checkbox"
                className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
              />
              <label htmlFor="remember-me" className="ml-2 block text-sm text-gray-700 dark:text-gray-300">
                Remember me
              </label>
            </div>

            <div className="text-sm">
              <a href="#" className="font-medium text-blue-600 hover:text-blue-500">
                Forgot password?
              </a>
            </div>
          </div>

          <button
            type="submit"
            disabled={isLoading}
            className="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isLoading ? 'Signing in...' : 'Sign in'}
          </button>
        </form>

        {/* SSO Options */}
        {(enableOAuth || enableSAML) && (
          <>
            <div className="relative">
              <div className="absolute inset-0 flex items-center">
                <div className="w-full border-t border-gray-300 dark:border-gray-600"></div>
              </div>
              <div className="relative flex justify-center text-sm">
                <span className="px-2 bg-white dark:bg-gray-800 text-gray-500">
                  Or continue with
                </span>
              </div>
            </div>

            <div className="space-y-3">
              {/* OAuth Providers */}
              {enableOAuth && oauthProviders.map((provider) => (
                <button
                  key={provider.id}
                  onClick={() => handleOAuthLogin(provider.id)}
                  disabled={isLoading}
                  className="w-full flex items-center justify-center px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600 disabled:opacity-50"
                >
                  <span className="mr-2">{provider.icon}</span>
                  {provider.name}
                </button>
              ))}

              {/* SAML Providers */}
              {enableSAML && samlProviders.map((provider) => (
                <button
                  key={provider.id}
                  onClick={() => handleSAMLLogin(provider.id)}
                  disabled={isLoading}
                  className="w-full flex items-center justify-center px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600 disabled:opacity-50"
                >
                  <span className="mr-2">{provider.icon}</span>
                  {provider.name}
                </button>
              ))}
            </div>
          </>
        )}

        {/* Footer */}
        <div className="text-center text-sm text-gray-600 dark:text-gray-400">
          <p>
            Don't have an account?{' '}
            <a href="#" className="font-medium text-blue-600 hover:text-blue-500">
              Contact your administrator
            </a>
          </p>
        </div>
      </div>
    </div>
  );
};

export default LoginPage;
