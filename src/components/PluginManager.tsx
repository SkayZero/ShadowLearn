import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';

interface Plugin {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  enabled: boolean;
  hooks: string[];
}

interface PluginStats {
  total_plugins: number;
  enabled_plugins: number;
  total_hooks: number;
  plugin_directory: string;
}

interface PluginManagerProps {
  isOpen: boolean;
  onClose: () => void;
}

export function PluginManager({ isOpen, onClose }: PluginManagerProps) {
  const [plugins, setPlugins] = useState<Plugin[]>([]);
  const [stats, setStats] = useState<PluginStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [selectedPlugin, setSelectedPlugin] = useState<Plugin | null>(null);

  useEffect(() => {
    if (isOpen) {
      loadPlugins();
    }
  }, [isOpen]);

  const loadPlugins = async () => {
    try {
      setLoading(true);
      const [pluginList, pluginStats] = await Promise.all([
        invoke<Plugin[]>('get_all_plugins'),
        invoke<PluginStats>('get_plugin_stats'),
      ]);
      setPlugins(pluginList);
      setStats(pluginStats);
    } catch (error) {
      console.error('Failed to load plugins:', error);
    } finally {
      setLoading(false);
    }
  };

  const togglePlugin = async (pluginId: string, currentlyEnabled: boolean) => {
    try {
      if (currentlyEnabled) {
        await invoke('disable_plugin', { pluginId });
      } else {
        await invoke('enable_plugin', { pluginId });
      }
      await loadPlugins();
    } catch (error) {
      console.error('Failed to toggle plugin:', error);
    }
  };

  const uninstallPlugin = async (pluginId: string) => {
    if (!confirm(`Are you sure you want to uninstall this plugin?`)) {
      return;
    }

    try {
      await invoke('uninstall_plugin', { pluginId });
      await loadPlugins();
      setSelectedPlugin(null);
    } catch (error) {
      console.error('Failed to uninstall plugin:', error);
    }
  };

  const reloadPlugins = async () => {
    try {
      setLoading(true);
      await invoke('reload_plugins');
      await loadPlugins();
    } catch (error) {
      console.error('Failed to reload plugins:', error);
    }
  };

  if (!isOpen) {
    return null;
  }

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        style={{
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          background: 'rgba(0, 0, 0, 0.6)',
          backdropFilter: 'blur(12px)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          zIndex: 70,
          padding: '24px',
        }}
        onClick={onClose}
      >
        <motion.div
          initial={{ scale: 0.95, y: 20 }}
          animate={{ scale: 1, y: 0 }}
          exit={{ scale: 0.95, y: 20 }}
          onClick={(e) => e.stopPropagation()}
          style={{
            maxWidth: '800px',
            width: '100%',
            maxHeight: '90vh',
            background: 'var(--glass-bg)',
            backdropFilter: 'var(--glass-backdrop)',
            WebkitBackdropFilter: 'var(--glass-backdrop)',
            border: '1px solid var(--glass-border)',
            borderRadius: 'var(--radius-2xl)',
            boxShadow: 'var(--elev-4)',
            overflow: 'hidden',
            display: 'flex',
            flexDirection: 'column',
          }}
        >
          {/* Header */}
          <div
            style={{
              padding: '24px',
              borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
              background: 'linear-gradient(135deg, rgba(139, 92, 246, 0.2), rgba(124, 58, 237, 0.2))',
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <div>
                <h2
                  style={{
                    fontSize: '28px',
                    fontWeight: '700',
                    color: 'var(--text-primary)',
                    margin: 0,
                  }}
                >
                  🔌 Plugin Manager
                </h2>
                {stats && (
                  <p
                    style={{
                      fontSize: '14px',
                      color: 'var(--text-muted)',
                      margin: '4px 0 0 0',
                    }}
                  >
                    {stats.enabled_plugins} of {stats.total_plugins} plugins enabled • {stats.total_hooks} hooks
                  </p>
                )}
              </div>
              <div style={{ display: 'flex', gap: '8px' }}>
                <button
                  onClick={reloadPlugins}
                  style={{
                    background: 'rgba(255, 255, 255, 0.1)',
                    border: 'none',
                    color: 'var(--text-primary)',
                    padding: '8px 16px',
                    borderRadius: '8px',
                    cursor: 'pointer',
                    fontSize: '14px',
                    fontWeight: '600',
                  }}
                >
                  🔄 Reload
                </button>
                <button
                  onClick={onClose}
                  style={{
                    background: 'transparent',
                    border: 'none',
                    color: 'var(--text-muted)',
                    fontSize: '28px',
                    cursor: 'pointer',
                    padding: '0',
                    width: '36px',
                    height: '36px',
                  }}
                >
                  ×
                </button>
              </div>
            </div>
          </div>

          {/* Content */}
          <div style={{ flex: 1, overflowY: 'auto', padding: '24px' }}>
            {loading ? (
              <div style={{ textAlign: 'center', padding: '60px', color: 'var(--text-muted)' }}>
                Loading plugins...
              </div>
            ) : plugins.length === 0 ? (
              <div style={{ textAlign: 'center', padding: '60px' }}>
                <div style={{ fontSize: '48px', marginBottom: '16px' }}>🔌</div>
                <div style={{ fontSize: '18px', color: 'var(--text-primary)', marginBottom: '8px' }}>
                  No plugins installed
                </div>
                <div style={{ fontSize: '14px', color: 'var(--text-muted)' }}>
                  Install plugins to extend ShadowLearn's functionality
                </div>
                {stats && (
                  <div
                    style={{
                      marginTop: '16px',
                      padding: '12px',
                      background: 'rgba(255, 255, 255, 0.05)',
                      borderRadius: '8px',
                      fontSize: '12px',
                      color: 'var(--text-muted)',
                    }}
                  >
                    Plugin directory: {stats.plugin_directory}
                  </div>
                )}
              </div>
            ) : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                {plugins.map((plugin, i) => (
                  <PluginCard
                    key={plugin.id}
                    plugin={plugin}
                    index={i}
                    onToggle={togglePlugin}
                    onUninstall={uninstallPlugin}
                    onSelect={setSelectedPlugin}
                    isSelected={selectedPlugin?.id === plugin.id}
                  />
                ))}
              </div>
            )}
          </div>

          {/* Footer */}
          {stats && (
            <div
              style={{
                padding: '16px 24px',
                borderTop: '1px solid rgba(255, 255, 255, 0.1)',
                background: 'rgba(255, 255, 255, 0.02)',
                fontSize: '12px',
                color: 'var(--text-muted)',
              }}
            >
              📁 {stats.plugin_directory}
            </div>
          )}
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
}

function PluginCard({
  plugin,
  index,
  onToggle,
  onUninstall,
  onSelect,
  isSelected,
}: {
  plugin: Plugin;
  index: number;
  onToggle: (id: string, enabled: boolean) => void;
  onUninstall: (id: string) => void;
  onSelect: (plugin: Plugin) => void;
  isSelected: boolean;
}) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay: index * 0.05 }}
      style={{
        padding: '20px',
        background: isSelected
          ? 'rgba(139, 92, 246, 0.15)'
          : 'rgba(255, 255, 255, 0.05)',
        borderRadius: 'var(--radius-lg)',
        border: isSelected ? '1px solid rgba(139, 92, 246, 0.4)' : 'none',
        cursor: 'pointer',
      }}
      onClick={() => onSelect(plugin)}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'start', marginBottom: '12px' }}>
        <div style={{ flex: 1 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '4px' }}>
            <div
              style={{
                fontSize: '18px',
                fontWeight: '600',
                color: 'var(--text-primary)',
              }}
            >
              {plugin.name}
            </div>
            <div
              style={{
                padding: '2px 8px',
                background: plugin.enabled ? 'rgba(16, 185, 129, 0.2)' : 'rgba(156, 163, 175, 0.2)',
                borderRadius: '4px',
                fontSize: '10px',
                fontWeight: '600',
                color: plugin.enabled ? '#10b981' : '#9ca3af',
                textTransform: 'uppercase',
              }}
            >
              {plugin.enabled ? 'Enabled' : 'Disabled'}
            </div>
          </div>
          <div style={{ fontSize: '13px', color: 'var(--text-muted)', marginBottom: '8px' }}>
            v{plugin.version} by {plugin.author}
          </div>
          <div style={{ fontSize: '14px', color: 'var(--text-secondary)' }}>
            {plugin.description}
          </div>
          {plugin.hooks.length > 0 && (
            <div style={{ marginTop: '12px', display: 'flex', flexWrap: 'wrap', gap: '6px' }}>
              {plugin.hooks.map((hook) => (
                <div
                  key={hook}
                  style={{
                    padding: '4px 8px',
                    background: 'rgba(99, 102, 241, 0.2)',
                    borderRadius: '4px',
                    fontSize: '11px',
                    color: '#6366f1',
                    fontFamily: 'monospace',
                  }}
                >
                  {hook}
                </div>
              ))}
            </div>
          )}
        </div>
        <div style={{ display: 'flex', gap: '8px', marginLeft: '16px' }}>
          <button
            onClick={(e) => {
              e.stopPropagation();
              onToggle(plugin.id, plugin.enabled);
            }}
            style={{
              padding: '8px 16px',
              background: plugin.enabled ? 'rgba(239, 68, 68, 0.2)' : 'rgba(16, 185, 129, 0.2)',
              border: 'none',
              borderRadius: '6px',
              color: plugin.enabled ? '#ef4444' : '#10b981',
              fontSize: '12px',
              fontWeight: '600',
              cursor: 'pointer',
            }}
          >
            {plugin.enabled ? 'Disable' : 'Enable'}
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              onUninstall(plugin.id);
            }}
            style={{
              padding: '8px 12px',
              background: 'rgba(239, 68, 68, 0.1)',
              border: 'none',
              borderRadius: '6px',
              color: '#ef4444',
              fontSize: '12px',
              cursor: 'pointer',
            }}
          >
            🗑️
          </button>
        </div>
      </div>
    </motion.div>
  );
}
