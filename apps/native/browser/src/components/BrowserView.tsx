/**
 * Browser View Component
 * Renders web content using hybrid approach
 */

import { useEffect, useState, useCallback, useRef } from 'react';
import DOMPurify from 'dompurify';
import type { BrowserTab, RenderMode } from '../types';
import type { NativeAppContext } from '../sdk.d';
import { isSafeUrl } from '../utils/url';
import './BrowserView.css';

interface BrowserViewProps {
  tab: BrowserTab;
  context: NativeAppContext;
  onLoadComplete: (title?: string, favicon?: string) => void;
  onError: (error: string) => void;
}

export function BrowserView({ tab, context, onLoadComplete, onError }: BrowserViewProps) {
  const [mode, setMode] = useState<RenderMode>('iframe');
  const [content, setContent] = useState<string>('');
  const [error, setError] = useState<string>('');
  const iframeRef = useRef<HTMLIFrameElement>(null);
  const loadingRef = useRef(false);

  // Load content
  const loadContent = useCallback(async () => {
    if (loadingRef.current) return;
    loadingRef.current = true;

    setError('');
    setContent('');

    // Special URLs
    if (tab.url === 'about:blank') {
      setMode('proxy');
      setContent('<div style="padding: 40px; text-align: center; color: #666;">New Tab</div>');
      onLoadComplete('New Tab');
      loadingRef.current = false;
      return;
    }

    // Safety check
    if (!isSafeUrl(tab.url)) {
      const err = 'Unsafe URL blocked';
      setError(err);
      onError(err);
      loadingRef.current = false;
      return;
    }

    // Try iframe first
    try {
      setMode('iframe');
      // Iframe will handle loading
      loadingRef.current = false;
    } catch (err) {
      console.error('[BrowserView] Iframe failed, falling back to proxy:', err);
      await loadViaProxy();
    }
  }, [tab.url, context.executor, onLoadComplete, onError]);

  // Load via HTTP proxy
  const loadViaProxy = useCallback(async () => {
    try {
      const response = await context.executor.execute('http.get', {
        url: tab.url,
        headers: {
          'User-Agent': 'Mozilla/5.0 (AgentOS Browser/1.0)',
        },
      });

      if (!response || response.status >= 400) {
        throw new Error(`HTTP ${response?.status || 'error'}`);
      }

      const html = response.body || response.data || '';

      // Extract title
      const titleMatch = html.match(/<title[^>]*>([^<]+)<\/title>/i);
      const title = titleMatch ? titleMatch[1] : tab.url;

      // Sanitize HTML
      const sanitized = DOMPurify.sanitize(html, {
        ADD_TAGS: ['style', 'link'],
        ADD_ATTR: ['target', 'rel'],
      });

      setMode('proxy');
      setContent(sanitized);
      onLoadComplete(title);
    } catch (err) {
      const errMsg = `Failed to load page: ${(err as Error).message}`;
      setError(errMsg);
      onError(errMsg);
    } finally {
      loadingRef.current = false;
    }
  }, [tab.url, context.executor, onLoadComplete, onError]);

  // Load on URL change
  useEffect(() => {
    loadContent();
  }, [tab.url]);

  // Handle iframe load
  const handleIframeLoad = useCallback(() => {
    try {
      const iframe = iframeRef.current;
      if (!iframe || !iframe.contentDocument) return;

      const title = iframe.contentDocument.title || tab.url;
      onLoadComplete(title);
    } catch (err) {
      // Cross-origin, fall back to proxy
      console.error('[BrowserView] Iframe cross-origin, switching to proxy');
      loadViaProxy();
    }
  }, [tab.url, onLoadComplete, loadViaProxy]);

  // Handle iframe error
  const handleIframeError = useCallback(() => {
    console.error('[BrowserView] Iframe error, switching to proxy');
    loadViaProxy();
  }, [loadViaProxy]);

  if (error) {
    return (
      <div className="browser-view error">
        <div className="error-content">
          <div className="error-icon">⚠️</div>
          <div className="error-title">Unable to load page</div>
          <div className="error-message">{error}</div>
          <button className="error-retry" onClick={() => loadContent()}>
            Retry
          </button>
        </div>
      </div>
    );
  }

  if (mode === 'iframe') {
    return (
      <div className="browser-view iframe-mode">
        <iframe
          ref={iframeRef}
          src={tab.url}
          className="content-iframe"
          sandbox="allow-same-origin allow-scripts allow-forms allow-popups allow-popups-to-escape-sandbox"
          onLoad={handleIframeLoad}
          onError={handleIframeError}
          title={tab.title}
        />
      </div>
    );
  }

  if (mode === 'proxy') {
    return (
      <div className="browser-view proxy-mode">
        <div
          className="content-proxy"
          dangerouslySetInnerHTML={{ __html: content }}
        />
      </div>
    );
  }

  return <div className="browser-view loading">Loading...</div>;
}

