/**
 * CADDY v0.4.0 - File Preview Component
 * Preview for images, PDFs, videos, documents, and more
 */

import React, { useState, useEffect, useCallback, useRef } from 'react';
import { FileItem, FilePreviewData, PreviewConfig } from './types';

interface FilePreviewProps {
  file: FileItem;
  tenantId: string;
  config?: Partial<PreviewConfig>;
  onClose: () => void;
  onDownload?: () => void;
  onShare?: () => void;
  onDelete?: () => void;
  onPrevious?: () => void;
  onNext?: () => void;
  className?: string;
}

const DEFAULT_CONFIG: PreviewConfig = {
  maxSize: 100 * 1024 * 1024, // 100MB
  supportedTypes: [
    'image/jpeg', 'image/png', 'image/gif', 'image/webp', 'image/svg+xml',
    'video/mp4', 'video/webm', 'video/ogg',
    'audio/mpeg', 'audio/wav', 'audio/ogg',
    'application/pdf',
    'text/plain', 'text/html', 'text/css', 'text/javascript',
    'application/json', 'application/xml',
  ],
  enableAnnotations: true,
  enableComments: true,
  enableDownload: true,
  enablePrint: true,
  enableShare: true,
};

export const FilePreview: React.FC<FilePreviewProps> = ({
  file,
  tenantId,
  config: userConfig = {},
  onClose,
  onDownload,
  onShare,
  onDelete,
  onPrevious,
  onNext,
  className = '',
}) => {
  const config = { ...DEFAULT_CONFIG, ...userConfig };
  const [previewData, setPreviewData] = useState<FilePreviewData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [currentPage, setCurrentPage] = useState(1);
  const [zoom, setZoom] = useState(100);
  const [rotation, setRotation] = useState(0);
  const contentRef = useRef<HTMLDivElement>(null);
  const videoRef = useRef<HTMLVideoElement>(null);
  const audioRef = useRef<HTMLAudioElement>(null);

  // Load preview
  useEffect(() => {
    loadPreview();
  }, [file.id]);

  const loadPreview = async () => {
    setLoading(true);
    setError(null);

    try {
      // Check if file type is supported
      if (!config.supportedTypes.includes(file.mimeType)) {
        setPreviewData({
          file,
          url: '',
          type: 'unsupported',
        });
        setLoading(false);
        return;
      }

      // Check file size
      if (file.size > config.maxSize) {
        throw new Error(`File too large for preview (max ${formatBytes(config.maxSize)})`);
      }

      // Get preview URL
      const token = localStorage.getItem('token');
      const url = `/api/v1/tenants/${tenantId}/files/${file.id}/content?token=${token}`;

      // Determine preview type
      const type = getPreviewType(file.mimeType);

      // For text files, fetch content
      let content: string | undefined;
      if (type === 'text' || type === 'code') {
        const response = await fetch(url);
        if (!response.ok) throw new Error('Failed to load file content');
        content = await response.text();
      }

      setPreviewData({
        file,
        url,
        type,
        content,
        pages: type === 'pdf' ? await getPdfPageCount(url) : undefined,
        currentPage: 1,
      });
      setLoading(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load preview');
      setLoading(false);
    }
  };

  const getPreviewType = (mimeType: string): FilePreviewData['type'] => {
    if (mimeType.startsWith('image/')) return 'image';
    if (mimeType.startsWith('video/')) return 'video';
    if (mimeType.startsWith('audio/')) return 'audio';
    if (mimeType === 'application/pdf') return 'pdf';
    if (mimeType.startsWith('text/') || mimeType.includes('json') || mimeType.includes('xml')) {
      return mimeType.includes('javascript') || mimeType.includes('json') || mimeType.includes('xml')
        ? 'code'
        : 'text';
    }
    return 'document';
  };

  const getPdfPageCount = async (url: string): Promise<number> => {
    // This would use a PDF library like pdf.js
    // For now, return a placeholder
    return 1;
  };

  // Keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      switch (e.key) {
        case 'Escape':
          onClose();
          break;
        case 'ArrowLeft':
          if (e.ctrlKey || e.metaKey) {
            onPrevious?.();
          } else if (previewData?.type === 'pdf' && currentPage > 1) {
            setCurrentPage(p => p - 1);
          }
          break;
        case 'ArrowRight':
          if (e.ctrlKey || e.metaKey) {
            onNext?.();
          } else if (previewData?.type === 'pdf' && previewData.pages && currentPage < previewData.pages) {
            setCurrentPage(p => p + 1);
          }
          break;
        case '+':
        case '=':
          if (e.ctrlKey || e.metaKey) {
            e.preventDefault();
            setZoom(z => Math.min(z + 10, 200));
          }
          break;
        case '-':
          if (e.ctrlKey || e.metaKey) {
            e.preventDefault();
            setZoom(z => Math.max(z - 10, 50));
          }
          break;
        case '0':
          if (e.ctrlKey || e.metaKey) {
            e.preventDefault();
            setZoom(100);
            setRotation(0);
          }
          break;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [onClose, onPrevious, onNext, currentPage, previewData]);

  const handleDownload = useCallback(() => {
    if (previewData?.url) {
      const a = document.createElement('a');
      a.href = previewData.url;
      a.download = file.name;
      a.click();
    }
    onDownload?.();
  }, [previewData, file.name, onDownload]);

  const handlePrint = useCallback(() => {
    if (previewData?.type === 'image' || previewData?.type === 'pdf') {
      window.print();
    }
  }, [previewData]);

  const handleRotate = useCallback(() => {
    setRotation(r => (r + 90) % 360);
  }, []);

  const handleZoomIn = useCallback(() => {
    setZoom(z => Math.min(z + 10, 200));
  }, []);

  const handleZoomOut = useCallback(() => {
    setZoom(z => Math.max(z - 10, 50));
  }, []);

  const handleZoomReset = useCallback(() => {
    setZoom(100);
    setRotation(0);
  }, []);

  const renderPreviewContent = () => {
    if (loading) {
      return (
        <div className="preview-loading">
          <div className="spinner"></div>
          <div>Loading preview...</div>
        </div>
      );
    }

    if (error) {
      return (
        <div className="preview-error">
          <div className="error-icon">⚠️</div>
          <div className="error-message">{error}</div>
          {config.enableDownload && (
            <button onClick={handleDownload} className="btn btn-primary">
              Download File
            </button>
          )}
        </div>
      );
    }

    if (!previewData) {
      return null;
    }

    switch (previewData.type) {
      case 'image':
        return (
          <div className="preview-image-container">
            <img
              src={previewData.url}
              alt={file.name}
              style={{
                transform: `scale(${zoom / 100}) rotate(${rotation}deg)`,
                maxWidth: '100%',
                maxHeight: '100%',
              }}
            />
          </div>
        );

      case 'video':
        return (
          <div className="preview-video-container">
            <video
              ref={videoRef}
              src={previewData.url}
              controls
              style={{ maxWidth: '100%', maxHeight: '100%' }}
            >
              Your browser does not support the video tag.
            </video>
          </div>
        );

      case 'audio':
        return (
          <div className="preview-audio-container">
            <div className="audio-info">
              <div className="audio-icon">🎵</div>
              <div className="audio-name">{file.name}</div>
              <div className="audio-size">{formatBytes(file.size)}</div>
            </div>
            <audio
              ref={audioRef}
              src={previewData.url}
              controls
              style={{ width: '100%', maxWidth: '600px' }}
            >
              Your browser does not support the audio tag.
            </audio>
          </div>
        );

      case 'pdf':
        return (
          <div className="preview-pdf-container">
            <div className="pdf-toolbar">
              <button
                onClick={() => setCurrentPage(p => Math.max(1, p - 1))}
                disabled={currentPage === 1}
              >
                Previous
              </button>
              <span className="page-info">
                Page {currentPage} of {previewData.pages || 1}
              </span>
              <button
                onClick={() => setCurrentPage(p => Math.min(previewData.pages || 1, p + 1))}
                disabled={currentPage === (previewData.pages || 1)}
              >
                Next
              </button>
            </div>
            <iframe
              src={`${previewData.url}#page=${currentPage}&zoom=${zoom}`}
              style={{ width: '100%', height: '100%', border: 'none' }}
              title={file.name}
            />
          </div>
        );

      case 'text':
      case 'code':
        return (
          <div className="preview-text-container">
            <pre className={previewData.type === 'code' ? 'code-preview' : 'text-preview'}>
              <code>{previewData.content}</code>
            </pre>
          </div>
        );

      case 'unsupported':
        return (
          <div className="preview-unsupported">
            <div className="unsupported-icon">📄</div>
            <div className="unsupported-message">
              Preview not available for this file type
            </div>
            <div className="file-info">
              <div className="info-item">
                <strong>Name:</strong> {file.name}
              </div>
              <div className="info-item">
                <strong>Type:</strong> {file.mimeType}
              </div>
              <div className="info-item">
                <strong>Size:</strong> {formatBytes(file.size)}
              </div>
              <div className="info-item">
                <strong>Modified:</strong> {new Date(file.modifiedAt).toLocaleString()}
              </div>
            </div>
            {config.enableDownload && (
              <button onClick={handleDownload} className="btn btn-primary">
                Download File
              </button>
            )}
          </div>
        );

      default:
        return null;
    }
  };

  return (
    <div className={`file-preview-modal ${className}`}>
      <div className="preview-overlay" onClick={onClose}></div>
      <div className="preview-container">
        {/* Header */}
        <div className="preview-header">
          <div className="preview-title">
            <span className="preview-icon">{getFileIcon(file)}</span>
            <span className="preview-filename">{file.name}</span>
          </div>
          <div className="preview-actions">
            {onPrevious && (
              <button onClick={onPrevious} className="btn btn-icon" title="Previous (Ctrl+←)">
                ←
              </button>
            )}
            {onNext && (
              <button onClick={onNext} className="btn btn-icon" title="Next (Ctrl+→)">
                →
              </button>
            )}
            {(previewData?.type === 'image' || previewData?.type === 'pdf') && (
              <>
                <button onClick={handleZoomOut} className="btn btn-icon" title="Zoom out (Ctrl+-)">
                  −
                </button>
                <span className="zoom-level">{zoom}%</span>
                <button onClick={handleZoomIn} className="btn btn-icon" title="Zoom in (Ctrl++)">
                  +
                </button>
                <button onClick={handleZoomReset} className="btn btn-icon" title="Reset (Ctrl+0)">
                  Reset
                </button>
              </>
            )}
            {previewData?.type === 'image' && (
              <button onClick={handleRotate} className="btn btn-icon" title="Rotate">
                ↻
              </button>
            )}
            {config.enableDownload && (
              <button onClick={handleDownload} className="btn btn-icon" title="Download">
                ⬇
              </button>
            )}
            {config.enablePrint && (previewData?.type === 'image' || previewData?.type === 'pdf') && (
              <button onClick={handlePrint} className="btn btn-icon" title="Print">
                🖨️
              </button>
            )}
            {config.enableShare && onShare && (
              <button onClick={onShare} className="btn btn-icon" title="Share">
                🔗
              </button>
            )}
            {onDelete && (
              <button onClick={onDelete} className="btn btn-icon btn-danger" title="Delete">
                🗑️
              </button>
            )}
            <button onClick={onClose} className="btn btn-icon" title="Close (Esc)">
              ✕
            </button>
          </div>
        </div>

        {/* Content */}
        <div className="preview-content" ref={contentRef}>
          {renderPreviewContent()}
        </div>

        {/* Footer */}
        <div className="preview-footer">
          <div className="preview-meta">
            <span>{formatBytes(file.size)}</span>
            <span>•</span>
            <span>{new Date(file.modifiedAt).toLocaleString()}</span>
            <span>•</span>
            <span>Modified by {file.modifiedBy}</span>
          </div>
        </div>
      </div>
    </div>
  );
};

function getFileIcon(file: FileItem): string {
  if (file.type === 'folder') return '📁';

  const ext = file.name.split('.').pop()?.toLowerCase() || '';
  const iconMap: Record<string, string> = {
    jpg: '🖼️', jpeg: '🖼️', png: '🖼️', gif: '🖼️', svg: '🖼️', webp: '🖼️',
    pdf: '📕', doc: '📘', docx: '📘', xls: '📗', xlsx: '📗', ppt: '📙', pptx: '📙',
    txt: '📄', md: '📝',
    zip: '📦', rar: '📦', '7z': '📦',
    mp4: '🎬', mov: '🎬', avi: '🎬',
    mp3: '🎵', wav: '🎵', flac: '🎵',
  };

  return iconMap[ext] || '📄';
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}

export default FilePreview;
