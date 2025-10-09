/**
 * Browser View Component
 * Renders web content using hybrid approach
 */

import { useEffect, useState, useCallback, useRef } from 'react';
import DOMPurify from 'dompurify';
import type { BrowserTab } from '../types';
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
  const [content, setContent] = useState<string>('');
  const [error, setError] = useState<string>('');
  const loadingRef = useRef(false);

  // Load content via HTTP proxy
  const loadContent = useCallback(async () => {
    if (loadingRef.current) return;
    loadingRef.current = true;

    setError('');
    setContent('');

    // Special URLs
    if (tab.url === 'about:blank') {
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

    // Load via HTTP proxy
    await loadViaProxy();
  }, [tab.url, context.executor, onLoadComplete, onError]);

  // Load via browser proxy service
  const loadViaProxy = useCallback(async () => {
    try {
      const response = await context.executor.execute('browser.navigate', {
        url: tab.url,
        session_id: context.appId,
      });

      if (!response) {
        throw new Error('No response from browser proxy');
      }

      const html = response.html || '';
      const title = response.title || tab.url;

      // Sanitize HTML (defense in depth - backend also sanitizes)
      const sanitized = DOMPurify.sanitize(html, {
        ADD_TAGS: ['style', 'link', 'base'],
        ADD_ATTR: ['target', 'rel', 'href', 'src', 'data-original-href', 'data-original-src'],
      });

      setContent(sanitized);
      onLoadComplete(title);
    } catch (err) {
      const errMsg = `Failed to load page: ${(err as Error).message}`;
      setError(errMsg);
      onError(errMsg);
    } finally {
      loadingRef.current = false;
    }
  }, [tab.url, context.executor, context.appId, onLoadComplete, onError]);

  // Load on URL change
  useEffect(() => {
    loadContent();
  }, [tab.url]);

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

  // Handle link clicks to navigate through proxy
  const handleContentClick = useCallback((e: React.MouseEvent) => {
    // Check if clicked element or parent is a link
    let target = e.target as HTMLElement;
    let link: HTMLAnchorElement | null = null;

    // Traverse up to find anchor tag
    for (let i = 0; i < 5 && target; i++) {
      if (target.tagName === 'A') {
        link = target as HTMLAnchorElement;
        break;
      }
      target = target.parentElement as HTMLElement;
    }

    // If we found a link, intercept and navigate through proxy
    if (link && link.href) {
      e.preventDefault();
      const url = link.href;

      // Navigate through proxy
      context.executor.execute('browser.navigate', {
        url,
        session_id: context.appId,
      }).then((response) => {
        if (response && response.html) {
          const sanitized = DOMPurify.sanitize(response.html, {
            ADD_TAGS: ['style', 'link', 'base'],
            ADD_ATTR: ['target', 'rel', 'href', 'src', 'data-original-href', 'data-original-src'],
          });
          setContent(sanitized);
          onLoadComplete(response.title || url);

          // Update tab URL (simplified - would need proper state update)
          console.log('[Browser] Navigated to:', url);
        }
      }).catch((err) => {
        console.error('[Browser] Navigation failed:', err);
      });
    }
  }, [context.executor, context.appId, onLoadComplete]);

  if (content) {
    return (
      <div className="browser-view proxy-mode">
        <div
          className="content-proxy"
          dangerouslySetInnerHTML={{ __html: content }}
          onClick={handleContentClick}
        />
      </div>
    );
  }

  return <div className="browser-view loading">Loading...</div>;
}

