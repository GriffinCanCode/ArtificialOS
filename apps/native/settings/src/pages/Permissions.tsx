/**
 * Permissions Settings Page
 */

import { useState, useEffect } from 'react';
import './Page.css';

interface PermissionsPageProps {
  executor: any;
  state: any;
}

export function PermissionsPage({ executor }: PermissionsPageProps) {
  const [permissions, setPermissions] = useState<any[]>([]);
  const [resources, setResources] = useState<any[]>([]);
  const [auditLog, setAuditLog] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadPermissions();
    loadResources();
    loadAuditLog();
  }, []);

  const loadPermissions = async () => {
    try {
      const result = await executor.execute('permissions.list', {});
      if (result?.permissions) {
        setPermissions(result.permissions);
      }
      setLoading(false);
    } catch (error) {
      console.error('Failed to load permissions:', error);
      setLoading(false);
    }
  };

  const loadResources = async () => {
    try {
      const result = await executor.execute('permissions.resources', {});
      if (result?.resources) {
        setResources(result.resources);
      }
    } catch (error) {
      console.error('Failed to load resources:', error);
    }
  };

  const loadAuditLog = async () => {
    try {
      const result = await executor.execute('permissions.audit', { limit: 20 });
      if (result?.audit_log) {
        setAuditLog(result.audit_log);
      }
    } catch (error) {
      console.error('Failed to load audit log:', error);
    }
  };

  if (loading) {
    return <div className="settings-page loading">Loading...</div>;
  }

  return (
    <div className="settings-page">
      <div className="page-header">
        <h1>Permissions Management</h1>
        <p className="page-description">Control app access to system resources</p>
      </div>

      <div className="settings-section">
        <h2 className="section-title">Available Resources</h2>
        <div className="resource-grid">
          {resources.map((resource) => (
            <div key={resource.type} className="resource-card">
              <div className="resource-type">{resource.type}</div>
              <div className="resource-description">{resource.description}</div>
              <div className="resource-actions">
                {resource.actions?.map((action: string) => (
                  <span key={action} className="action-badge">{action}</span>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>

      {permissions.length > 0 && (
        <div className="settings-section">
          <h2 className="section-title">Granted Permissions ({permissions.length})</h2>
          <div className="permission-list">
            {permissions.map((perm, idx) => (
              <div key={idx} className="permission-item">
                <div className="permission-info">
                  <div className="permission-app">{perm.app_id}</div>
                  <div className="permission-resource">{perm.resource}</div>
                  <div className="permission-actions">
                    {perm.actions?.join(', ')}
                  </div>
                </div>
                <button
                  className="btn-secondary"
                  onClick={async () => {
                    try {
                      await executor.execute('permissions.revoke', {
                        app_id: perm.app_id,
                        resource: perm.resource,
                      });
                      loadPermissions();
                    } catch (error) {
                      console.error('Failed to revoke permission:', error);
                    }
                  }}
                >
                  Revoke
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      {auditLog.length > 0 && (
        <div className="settings-section">
          <h2 className="section-title">Recent Activity</h2>
          <div className="audit-log">
            {auditLog.map((entry, idx) => (
              <div key={idx} className={`audit-entry ${entry.allowed ? 'allowed' : 'denied'}`}>
                <div className="audit-timestamp">
                  {new Date(entry.timestamp * 1000).toLocaleString()}
                </div>
                <div className="audit-info">
                  <span className="audit-app">{entry.app_id}</span>
                  {' → '}
                  <span className="audit-resource">{entry.resource}</span>
                  {' '}
                  <span className="audit-action">({entry.action})</span>
                </div>
                <div className={`audit-result ${entry.allowed ? 'allowed' : 'denied'}`}>
                  {entry.allowed ? '✓ Allowed' : '✗ Denied'}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

