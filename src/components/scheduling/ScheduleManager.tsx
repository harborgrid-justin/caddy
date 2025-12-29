/**
 * Schedule Manager Component
 *
 * Provides a comprehensive UI for managing scheduled jobs including:
 * - Viewing active, pending, and completed jobs
 * - Creating new scheduled jobs
 * - Editing existing schedules
 * - Cancelling jobs
 * - Viewing job execution history
 */

import React, { useState, useEffect, useCallback } from 'react';
import {
  Job,
  JobStatus,
  JobPriority,
  JobSchedule,
  ScheduleFormState,
  CreateJobRequest,
  QueueStats,
  JobProgress,
} from './types';

interface ScheduleManagerProps {
  apiBaseUrl?: string;
  onJobCreated?: (job: Job) => void;
  onJobCancelled?: (jobId: string) => void;
}

export const ScheduleManager: React.FC<ScheduleManagerProps> = ({
  apiBaseUrl = '/api/scheduling',
  onJobCreated,
  onJobCancelled,
}) => {
  const [jobs, setJobs] = useState<Job[]>([]);
  const [selectedJob, setSelectedJob] = useState<Job | null>(null);
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [filterStatus, setFilterStatus] = useState<JobStatus | 'all'>('all');
  const [queueStats, setQueueStats] = useState<QueueStats[]>([]);
  const [jobProgress, setJobProgress] = useState<Map<string, JobProgress>>(new Map());

  const [formState, setFormState] = useState<ScheduleFormState>({
    name: '',
    job_type: 'accessibility-scan',
    schedule_type: 'cron',
    cron_expression: '0 0 * * *',
    priority: JobPriority.Normal,
    payload: '{}',
    max_retries: 3,
    timeout_seconds: 300,
    tags: [],
  });

  // Fetch jobs
  const fetchJobs = useCallback(async () => {
    try {
      setLoading(true);
      const response = await fetch(`${apiBaseUrl}/jobs`);
      if (!response.ok) throw new Error('Failed to fetch jobs');
      const data = await response.json();
      setJobs(data);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  }, [apiBaseUrl]);

  // Fetch queue statistics
  const fetchQueueStats = useCallback(async () => {
    try {
      const response = await fetch(`${apiBaseUrl}/queues/stats`);
      if (!response.ok) throw new Error('Failed to fetch queue stats');
      const data = await response.json();
      setQueueStats(data);
    } catch (err) {
      console.error('Error fetching queue stats:', err);
    }
  }, [apiBaseUrl]);

  // Fetch job progress
  const fetchJobProgress = useCallback(async (jobId: string) => {
    try {
      const response = await fetch(`${apiBaseUrl}/jobs/${jobId}/progress`);
      if (!response.ok) return;
      const data: JobProgress = await response.json();
      setJobProgress((prev) => new Map(prev).set(jobId, data));
    } catch (err) {
      console.error('Error fetching job progress:', err);
    }
  }, [apiBaseUrl]);

  useEffect(() => {
    fetchJobs();
    fetchQueueStats();

    // Poll for updates every 5 seconds
    const interval = setInterval(() => {
      fetchJobs();
      fetchQueueStats();
    }, 5000);

    return () => clearInterval(interval);
  }, [fetchJobs, fetchQueueStats]);

  // Poll progress for running jobs
  useEffect(() => {
    const runningJobs = jobs.filter((j) => j.status === JobStatus.Running);
    runningJobs.forEach((job) => fetchJobProgress(job.id));
  }, [jobs, fetchJobProgress]);

  // Create job
  const handleCreateJob = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      let schedule: JobSchedule;
      switch (formState.schedule_type) {
        case 'once':
          schedule = {
            type: 'Once',
            timestamp: formState.once_timestamp || new Date().toISOString(),
          };
          break;
        case 'cron':
          schedule = {
            type: 'Cron',
            expression: formState.cron_expression || '0 0 * * *',
          };
          break;
        case 'interval':
          schedule = {
            type: 'Interval',
            duration: formState.interval_duration || 3600,
            start: formState.interval_start,
          };
          break;
        default:
          throw new Error('Invalid schedule type');
      }

      const payload = formState.payload ? JSON.parse(formState.payload) : {};
      const tags = Object.fromEntries(
        formState.tags.map((t) => [t.key, t.value])
      );

      const request: CreateJobRequest = {
        name: formState.name,
        job_type: formState.job_type,
        schedule,
        priority: formState.priority,
        payload,
        max_retries: formState.max_retries,
        timeout_seconds: formState.timeout_seconds,
        tags,
      };

      const response = await fetch(`${apiBaseUrl}/jobs`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(request),
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.message || 'Failed to create job');
      }

      const newJob: Job = await response.json();
      setJobs([...jobs, newJob]);
      setShowCreateForm(false);
      resetForm();

      if (onJobCreated) {
        onJobCreated(newJob);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  // Cancel job
  const handleCancelJob = async (jobId: string) => {
    if (!confirm('Are you sure you want to cancel this job?')) return;

    try {
      const response = await fetch(`${apiBaseUrl}/jobs/${jobId}/cancel`, {
        method: 'POST',
      });

      if (!response.ok) throw new Error('Failed to cancel job');

      setJobs(jobs.map((j) => (j.id === jobId ? { ...j, status: JobStatus.Cancelled } : j)));

      if (onJobCancelled) {
        onJobCancelled(jobId);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    }
  };

  // Reset form
  const resetForm = () => {
    setFormState({
      name: '',
      job_type: 'accessibility-scan',
      schedule_type: 'cron',
      cron_expression: '0 0 * * *',
      priority: JobPriority.Normal,
      payload: '{}',
      max_retries: 3,
      timeout_seconds: 300,
      tags: [],
    });
  };

  // Format date
  const formatDate = (dateStr?: string) => {
    if (!dateStr) return 'N/A';
    return new Date(dateStr).toLocaleString();
  };

  // Get status color
  const getStatusColor = (status: JobStatus): string => {
    switch (status) {
      case JobStatus.Completed:
        return 'text-green-600 bg-green-100';
      case JobStatus.Running:
        return 'text-blue-600 bg-blue-100';
      case JobStatus.Failed:
        return 'text-red-600 bg-red-100';
      case JobStatus.Cancelled:
        return 'text-gray-600 bg-gray-100';
      case JobStatus.Scheduled:
        return 'text-purple-600 bg-purple-100';
      case JobStatus.Retrying:
        return 'text-yellow-600 bg-yellow-100';
      default:
        return 'text-gray-600 bg-gray-100';
    }
  };

  // Get priority label
  const getPriorityLabel = (priority: JobPriority): string => {
    return JobPriority[priority];
  };

  // Filter jobs
  const filteredJobs = filterStatus === 'all'
    ? jobs
    : jobs.filter((j) => j.status === filterStatus);

  return (
    <div className="schedule-manager p-6 bg-gray-50 min-h-screen">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-6">
          <h1 className="text-3xl font-bold text-gray-900">Schedule Manager</h1>
          <p className="mt-2 text-gray-600">
            Manage scheduled jobs, cron tasks, and recurring scans
          </p>
        </div>

        {/* Error Display */}
        {error && (
          <div className="mb-4 p-4 bg-red-50 border border-red-200 rounded-lg">
            <p className="text-red-800">{error}</p>
          </div>
        )}

        {/* Queue Statistics */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
          {queueStats.map((stat) => (
            <div key={stat.queue_name} className="bg-white p-4 rounded-lg shadow">
              <h3 className="text-sm font-medium text-gray-500 uppercase">
                {stat.queue_name}
              </h3>
              <div className="mt-2 flex justify-between">
                <div>
                  <p className="text-2xl font-bold text-gray-900">{stat.pending_jobs}</p>
                  <p className="text-xs text-gray-500">Pending</p>
                </div>
                <div>
                  <p className="text-2xl font-bold text-red-600">{stat.failed_jobs}</p>
                  <p className="text-xs text-gray-500">Failed</p>
                </div>
              </div>
            </div>
          ))}
        </div>

        {/* Controls */}
        <div className="mb-6 flex justify-between items-center">
          <div className="flex gap-2">
            <button
              onClick={() => setShowCreateForm(!showCreateForm)}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition"
            >
              {showCreateForm ? 'Cancel' : 'Create New Job'}
            </button>
            <button
              onClick={fetchJobs}
              disabled={loading}
              className="px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 transition disabled:opacity-50"
            >
              {loading ? 'Loading...' : 'Refresh'}
            </button>
          </div>

          <div className="flex gap-2 items-center">
            <label className="text-sm text-gray-600">Filter:</label>
            <select
              value={filterStatus}
              onChange={(e) => setFilterStatus(e.target.value as JobStatus | 'all')}
              className="px-3 py-2 border border-gray-300 rounded-lg"
            >
              <option value="all">All Jobs</option>
              {Object.values(JobStatus).map((status) => (
                <option key={status} value={status}>
                  {status}
                </option>
              ))}
            </select>
          </div>
        </div>

        {/* Create Form */}
        {showCreateForm && (
          <div className="mb-6 bg-white p-6 rounded-lg shadow">
            <h2 className="text-xl font-bold mb-4">Create New Job</h2>
            <form onSubmit={handleCreateJob} className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Job Name
                  </label>
                  <input
                    type="text"
                    required
                    value={formState.name}
                    onChange={(e) => setFormState({ ...formState, name: e.target.value })}
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                    placeholder="Daily accessibility scan"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Job Type
                  </label>
                  <select
                    value={formState.job_type}
                    onChange={(e) => setFormState({ ...formState, job_type: e.target.value })}
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                  >
                    <option value="accessibility-scan">Accessibility Scan</option>
                    <option value="performance-test">Performance Test</option>
                    <option value="content-analysis">Content Analysis</option>
                    <option value="backup">Backup</option>
                    <option value="report-generation">Report Generation</option>
                  </select>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Schedule Type
                  </label>
                  <select
                    value={formState.schedule_type}
                    onChange={(e) =>
                      setFormState({
                        ...formState,
                        schedule_type: e.target.value as 'once' | 'cron' | 'interval',
                      })
                    }
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                  >
                    <option value="cron">Cron Expression</option>
                    <option value="interval">Fixed Interval</option>
                    <option value="once">One-Time</option>
                  </select>
                </div>

                {formState.schedule_type === 'cron' && (
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Cron Expression
                    </label>
                    <input
                      type="text"
                      value={formState.cron_expression}
                      onChange={(e) =>
                        setFormState({ ...formState, cron_expression: e.target.value })
                      }
                      className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                      placeholder="0 0 * * *"
                    />
                    <p className="text-xs text-gray-500 mt-1">
                      e.g., "0 0 * * *" = daily at midnight
                    </p>
                  </div>
                )}

                {formState.schedule_type === 'interval' && (
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Interval (seconds)
                    </label>
                    <input
                      type="number"
                      value={formState.interval_duration}
                      onChange={(e) =>
                        setFormState({
                          ...formState,
                          interval_duration: parseInt(e.target.value),
                        })
                      }
                      className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                      placeholder="3600"
                    />
                  </div>
                )}

                {formState.schedule_type === 'once' && (
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Execution Time
                    </label>
                    <input
                      type="datetime-local"
                      value={formState.once_timestamp?.slice(0, 16)}
                      onChange={(e) =>
                        setFormState({
                          ...formState,
                          once_timestamp: new Date(e.target.value).toISOString(),
                        })
                      }
                      className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                    />
                  </div>
                )}

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Priority
                  </label>
                  <select
                    value={formState.priority}
                    onChange={(e) =>
                      setFormState({ ...formState, priority: parseInt(e.target.value) })
                    }
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                  >
                    <option value={JobPriority.Low}>Low</option>
                    <option value={JobPriority.Normal}>Normal</option>
                    <option value={JobPriority.High}>High</option>
                    <option value={JobPriority.Critical}>Critical</option>
                  </select>
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Payload (JSON)
                </label>
                <textarea
                  value={formState.payload}
                  onChange={(e) => setFormState({ ...formState, payload: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg font-mono text-sm"
                  rows={4}
                  placeholder='{"url": "https://example.com", "options": {}}'
                />
              </div>

              <div className="flex gap-4">
                <button
                  type="submit"
                  disabled={loading}
                  className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition disabled:opacity-50"
                >
                  {loading ? 'Creating...' : 'Create Job'}
                </button>
                <button
                  type="button"
                  onClick={() => setShowCreateForm(false)}
                  className="px-6 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 transition"
                >
                  Cancel
                </button>
              </div>
            </form>
          </div>
        )}

        {/* Jobs List */}
        <div className="bg-white rounded-lg shadow overflow-hidden">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Job Name
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Type
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Status
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Priority
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Next Run
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Last Run
                </th>
                <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Actions
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {filteredJobs.length === 0 ? (
                <tr>
                  <td colSpan={7} className="px-6 py-8 text-center text-gray-500">
                    No jobs found
                  </td>
                </tr>
              ) : (
                filteredJobs.map((job) => (
                  <tr key={job.id} className="hover:bg-gray-50">
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="text-sm font-medium text-gray-900">{job.name}</div>
                      <div className="text-xs text-gray-500">{job.id.slice(0, 8)}</div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {job.job_type}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span
                        className={`px-2 py-1 inline-flex text-xs leading-5 font-semibold rounded-full ${getStatusColor(
                          job.status
                        )}`}
                      >
                        {job.status}
                      </span>
                      {job.status === JobStatus.Running && jobProgress.has(job.id) && (
                        <div className="mt-1">
                          <div className="text-xs text-gray-500">
                            {jobProgress.get(job.id)!.percentage.toFixed(1)}%
                          </div>
                          <div className="w-full bg-gray-200 rounded-full h-1.5 mt-1">
                            <div
                              className="bg-blue-600 h-1.5 rounded-full"
                              style={{ width: `${jobProgress.get(job.id)!.percentage}%` }}
                            />
                          </div>
                        </div>
                      )}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {getPriorityLabel(job.priority)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {formatDate(job.next_run)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {formatDate(job.last_run)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                      <button
                        onClick={() => setSelectedJob(job)}
                        className="text-blue-600 hover:text-blue-900 mr-3"
                      >
                        Details
                      </button>
                      {job.status !== JobStatus.Completed &&
                        job.status !== JobStatus.Cancelled && (
                          <button
                            onClick={() => handleCancelJob(job.id)}
                            className="text-red-600 hover:text-red-900"
                          >
                            Cancel
                          </button>
                        )}
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>

        {/* Job Details Modal */}
        {selectedJob && (
          <div
            className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
            onClick={() => setSelectedJob(null)}
          >
            <div
              className="bg-white rounded-lg p-6 max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto"
              onClick={(e) => e.stopPropagation()}
            >
              <h2 className="text-2xl font-bold mb-4">{selectedJob.name}</h2>

              <div className="space-y-3">
                <div>
                  <span className="font-medium">ID:</span> {selectedJob.id}
                </div>
                <div>
                  <span className="font-medium">Type:</span> {selectedJob.job_type}
                </div>
                <div>
                  <span className="font-medium">Status:</span>{' '}
                  <span
                    className={`px-2 py-1 inline-flex text-xs leading-5 font-semibold rounded-full ${getStatusColor(
                      selectedJob.status
                    )}`}
                  >
                    {selectedJob.status}
                  </span>
                </div>
                <div>
                  <span className="font-medium">Priority:</span>{' '}
                  {getPriorityLabel(selectedJob.priority)}
                </div>
                <div>
                  <span className="font-medium">Created:</span>{' '}
                  {formatDate(selectedJob.created_at)}
                </div>
                <div>
                  <span className="font-medium">Next Run:</span>{' '}
                  {formatDate(selectedJob.next_run)}
                </div>
                <div>
                  <span className="font-medium">Last Run:</span>{' '}
                  {formatDate(selectedJob.last_run)}
                </div>
                <div>
                  <span className="font-medium">Retries:</span> {selectedJob.retry_count} /{' '}
                  {selectedJob.max_retries}
                </div>
                {selectedJob.last_error && (
                  <div>
                    <span className="font-medium">Last Error:</span>
                    <pre className="mt-1 p-2 bg-red-50 text-red-800 text-sm rounded">
                      {selectedJob.last_error}
                    </pre>
                  </div>
                )}
                <div>
                  <span className="font-medium">Payload:</span>
                  <pre className="mt-1 p-2 bg-gray-50 text-sm rounded overflow-x-auto">
                    {JSON.stringify(selectedJob.payload, null, 2)}
                  </pre>
                </div>
              </div>

              <button
                onClick={() => setSelectedJob(null)}
                className="mt-6 px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 transition"
              >
                Close
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default ScheduleManager;
