import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useProductivity } from '../hooks/useProductivity';
import type { Insight, FlowSession } from '../hooks/useProductivity';

interface ProductivityDashboardProps {
  isOpen: boolean;
  onClose: () => void;
}

export function ProductivityDashboard({ isOpen, onClose }: ProductivityDashboardProps) {
  const { metrics, loading, refresh } = useProductivity(true, 30000);
  const [selectedTab, setSelectedTab] = useState<'overview' | 'trends' | 'insights'>('overview');

  useEffect(() => {
    if (isOpen) {
      refresh();
    }
  }, [isOpen]);

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
            maxWidth: '900px',
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
              background: 'linear-gradient(135deg, rgba(99, 102, 241, 0.2), rgba(139, 92, 246, 0.2))',
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
                  📊 Productivity Dashboard
                </h2>
                <p
                  style={{
                    fontSize: '14px',
                    color: 'var(--text-muted)',
                    margin: '4px 0 0 0',
                  }}
                >
                  Track your productivity, flow, and insights
                </p>
              </div>
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
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  transition: 'color 0.2s',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.color = 'var(--text-primary)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.color = 'var(--text-muted)';
                }}
              >
                ×
              </button>
            </div>

            {/* Tabs */}
            <div style={{ marginTop: '16px', display: 'flex', gap: '8px' }}>
              {(['overview', 'trends', 'insights'] as const).map((tab) => (
                <button
                  key={tab}
                  onClick={() => setSelectedTab(tab)}
                  style={{
                    padding: '8px 16px',
                    background: selectedTab === tab ? 'rgba(255, 255, 255, 0.2)' : 'transparent',
                    border: 'none',
                    borderRadius: '8px',
                    color: selectedTab === tab ? 'var(--text-primary)' : 'var(--text-secondary)',
                    fontWeight: selectedTab === tab ? '600' : '400',
                    cursor: 'pointer',
                    transition: 'all 0.2s',
                    fontSize: '14px',
                    textTransform: 'capitalize',
                  }}
                >
                  {tab}
                </button>
              ))}
            </div>
          </div>

          {/* Content */}
          <div style={{ flex: 1, overflowY: 'auto', padding: '24px' }}>
            {loading ? (
              <div style={{ textAlign: 'center', padding: '60px', color: 'var(--text-muted)' }}>
                <div style={{ fontSize: '18px' }}>Loading metrics...</div>
              </div>
            ) : metrics ? (
              <>
                {selectedTab === 'overview' && <OverviewTab metrics={metrics} />}
                {selectedTab === 'trends' && <TrendsTab metrics={metrics} />}
                {selectedTab === 'insights' && <InsightsTab insights={metrics.insights} />}
              </>
            ) : (
              <div style={{ textAlign: 'center', padding: '60px', color: 'var(--text-muted)' }}>
                No data available
              </div>
            )}
          </div>
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
}

// Overview Tab
function OverviewTab({ metrics }: { metrics: any }) {
  const { today, week, top_productive_hours, flow_sessions } = metrics;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
      {/* Today's Stats */}
      <div>
        <h3
          style={{
            fontSize: '18px',
            fontWeight: '600',
            color: 'var(--text-primary)',
            marginBottom: '12px',
          }}
        >
          Today's Performance
        </h3>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))', gap: '12px' }}>
          <MetricCard
            icon="💡"
            value={today.suggestions_shown}
            label="Suggestions"
            color="rgba(251, 191, 36, 0.3)"
          />
          <MetricCard
            icon="✅"
            value={today.suggestions_accepted}
            label="Accepted"
            color="rgba(16, 185, 129, 0.3)"
          />
          <MetricCard
            icon="🎯"
            value={`${Math.round(today.acceptance_rate)}%`}
            label="Rate"
            color="rgba(99, 102, 241, 0.3)"
          />
          <MetricCard
            icon="⚡"
            value={`${today.time_saved_minutes}min`}
            label="Saved"
            color="rgba(139, 92, 246, 0.3)"
          />
          <MetricCard
            icon="🧘"
            value={`${Math.round(today.flow_time_minutes / 60)}h`}
            label="Flow"
            color="rgba(14, 165, 233, 0.3)"
          />
          <MetricCard
            icon="⚠️"
            value={today.interruptions}
            label="Interruptions"
            color="rgba(239, 68, 68, 0.3)"
          />
        </div>
      </div>

      {/* Week Summary */}
      <div>
        <h3
          style={{
            fontSize: '18px',
            fontWeight: '600',
            color: 'var(--text-primary)',
            marginBottom: '12px',
          }}
        >
          This Week
        </h3>
        <div
          style={{
            padding: '20px',
            background: 'linear-gradient(135deg, rgba(139, 92, 246, 0.15), rgba(99, 102, 241, 0.15))',
            borderRadius: 'var(--radius-lg)',
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(120px, 1fr))',
            gap: '16px',
          }}
        >
          <WeekStat label="Total Suggestions" value={week.total_suggestions} />
          <WeekStat label="Accepted" value={week.total_accepted} />
          <WeekStat label="Time Saved" value={`${week.total_time_saved}min`} />
          <WeekStat label="Flow Time" value={`${Math.round(week.total_flow_time / 60)}h`} />
        </div>
      </div>

      {/* Top Productive Hours */}
      {top_productive_hours && top_productive_hours.length > 0 && (
        <div>
          <h3
            style={{
              fontSize: '18px',
              fontWeight: '600',
              color: 'var(--text-primary)',
              marginBottom: '12px',
            }}
          >
            Best Hours to Work
          </h3>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
            {top_productive_hours.slice(0, 5).map((hour: any, i: number) => (
              <motion.div
                key={hour.hour}
                initial={{ opacity: 0, x: -10 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: i * 0.05 }}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  padding: '12px 16px',
                  background: 'rgba(255, 255, 255, 0.05)',
                  borderRadius: '8px',
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                  <div
                    style={{
                      fontSize: '20px',
                      fontWeight: '700',
                      color: 'var(--accent-primary)',
                    }}
                  >
                    {hour.hour}:00
                  </div>
                  <div>
                    <div style={{ fontSize: '13px', color: 'var(--text-secondary)' }}>
                      {hour.flow_sessions} flow sessions
                    </div>
                  </div>
                </div>
                <div style={{ fontSize: '14px', color: 'var(--text-muted)', fontWeight: '600' }}>
                  Score: {Math.round(hour.productivity_score)}
                </div>
              </motion.div>
            ))}
          </div>
        </div>
      )}

      {/* Recent Flow Sessions */}
      {flow_sessions && flow_sessions.length > 0 && (
        <div>
          <h3
            style={{
              fontSize: '18px',
              fontWeight: '600',
              color: 'var(--text-primary)',
              marginBottom: '12px',
            }}
          >
            Recent Flow Sessions
          </h3>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
            {flow_sessions.slice(0, 5).map((session: FlowSession, i: number) => (
              <FlowSessionCard key={i} session={session} index={i} />
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

// Trends Tab
function TrendsTab({ metrics }: { metrics: any }) {
  const { trends, week } = metrics;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
      {/* Week Progress */}
      <div>
        <h3
          style={{
            fontSize: '18px',
            fontWeight: '600',
            color: 'var(--text-primary)',
            marginBottom: '12px',
          }}
        >
          Weekly Progress
        </h3>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
          {week.daily_breakdown.map((day: any, i: number) => (
            <DayProgressBar key={i} day={day} isToday={i === 0} />
          ))}
        </div>
      </div>

      {/* Trend Summary */}
      <div>
        <h3
          style={{
            fontSize: '18px',
            fontWeight: '600',
            color: 'var(--text-primary)',
            marginBottom: '12px',
          }}
        >
          30-Day Trends
        </h3>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))', gap: '16px' }}>
          <TrendCard
            title="Acceptance Rate"
            data={trends.acceptance_rate_trend}
            color="#10b981"
            suffix="%"
          />
          <TrendCard
            title="Flow Time"
            data={trends.flow_time_trend}
            color="#6366f1"
            suffix="min"
          />
          <TrendCard
            title="Productivity Score"
            data={trends.productivity_score_trend}
            color="#8b5cf6"
            suffix=""
          />
        </div>
      </div>
    </div>
  );
}

// Insights Tab
function InsightsTab({ insights }: { insights: Insight[] }) {
  const categoryIcons: Record<string, string> = {
    Achievement: '🏆',
    Pattern: '🔍',
    Improvement: '📈',
    Warning: '⚠️',
  };

  const impactColors: Record<string, string> = {
    High: 'rgba(239, 68, 68, 0.2)',
    Medium: 'rgba(251, 191, 36, 0.2)',
    Low: 'rgba(156, 163, 175, 0.2)',
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
      {insights.length === 0 ? (
        <div style={{ textAlign: 'center', padding: '60px', color: 'var(--text-muted)' }}>
          <div style={{ fontSize: '48px', marginBottom: '16px' }}>🔍</div>
          <div style={{ fontSize: '16px' }}>No insights yet</div>
          <div style={{ fontSize: '14px', marginTop: '8px' }}>
            Keep using ShadowLearn to generate insights!
          </div>
        </div>
      ) : (
        insights.map((insight, i) => (
          <motion.div
            key={insight.id}
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: i * 0.05 }}
            style={{
              padding: '20px',
              background: impactColors[insight.impact],
              borderRadius: 'var(--radius-lg)',
              border: '1px solid rgba(255, 255, 255, 0.1)',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'start', gap: '16px' }}>
              <div style={{ fontSize: '32px' }}>{categoryIcons[insight.category]}</div>
              <div style={{ flex: 1 }}>
                <div
                  style={{
                    fontSize: '16px',
                    fontWeight: '600',
                    color: 'var(--text-primary)',
                    marginBottom: '4px',
                  }}
                >
                  {insight.title}
                </div>
                <div
                  style={{
                    fontSize: '14px',
                    color: 'var(--text-secondary)',
                    marginBottom: insight.action ? '12px' : '0',
                  }}
                >
                  {insight.description}
                </div>
                {insight.action && (
                  <div
                    style={{
                      padding: '10px 14px',
                      background: 'rgba(255, 255, 255, 0.1)',
                      borderRadius: '6px',
                      fontSize: '13px',
                      color: 'var(--text-primary)',
                      fontWeight: '500',
                    }}
                  >
                    💡 {insight.action}
                  </div>
                )}
              </div>
              <div
                style={{
                  padding: '4px 10px',
                  background: 'rgba(255, 255, 255, 0.1)',
                  borderRadius: '12px',
                  fontSize: '11px',
                  fontWeight: '600',
                  color: 'var(--text-muted)',
                  textTransform: 'uppercase',
                }}
              >
                {insight.impact}
              </div>
            </div>
          </motion.div>
        ))
      )}
    </div>
  );
}

// Helper Components
function MetricCard({
  icon,
  value,
  label,
  color,
}: {
  icon: string;
  value: string | number;
  label: string;
  color: string;
}) {
  return (
    <div
      style={{
        padding: '16px',
        background: color,
        borderRadius: 'var(--radius-lg)',
        textAlign: 'center',
      }}
    >
      <div style={{ fontSize: '24px', marginBottom: '8px' }}>{icon}</div>
      <div
        style={{
          fontSize: '24px',
          fontWeight: '700',
          color: 'var(--text-primary)',
        }}
      >
        {value}
      </div>
      <div
        style={{
          fontSize: '11px',
          color: 'var(--text-muted)',
          marginTop: '4px',
          textTransform: 'uppercase',
          fontWeight: '600',
        }}
      >
        {label}
      </div>
    </div>
  );
}

function WeekStat({ label, value }: { label: string; value: string | number }) {
  return (
    <div style={{ textAlign: 'center' }}>
      <div
        style={{
          fontSize: '24px',
          fontWeight: '700',
          color: 'var(--text-primary)',
        }}
      >
        {value}
      </div>
      <div
        style={{
          fontSize: '12px',
          color: 'var(--text-muted)',
          marginTop: '4px',
        }}
      >
        {label}
      </div>
    </div>
  );
}

function FlowSessionCard({ session, index }: { session: FlowSession; index: number }) {
  const formatTime = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' });
  };

  return (
    <motion.div
      initial={{ opacity: 0, x: -10 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ delay: index * 0.05 }}
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '14px 16px',
        background: `linear-gradient(90deg, rgba(14, 165, 233, ${session.quality_score * 0.2}), transparent)`,
        borderRadius: '8px',
        border: '1px solid rgba(255, 255, 255, 0.1)',
      }}
    >
      <div>
        <div style={{ fontSize: '14px', color: 'var(--text-primary)', fontWeight: '600' }}>
          {session.app_name}
        </div>
        <div style={{ fontSize: '12px', color: 'var(--text-muted)', marginTop: '2px' }}>
          {formatTime(session.start_time)} - {formatTime(session.end_time)}
        </div>
      </div>
      <div style={{ textAlign: 'right' }}>
        <div style={{ fontSize: '16px', color: 'var(--accent-primary)', fontWeight: '700' }}>
          {session.duration_minutes}min
        </div>
        <div style={{ fontSize: '11px', color: 'var(--text-muted)' }}>
          Quality: {Math.round(session.quality_score * 100)}%
        </div>
      </div>
    </motion.div>
  );
}

function DayProgressBar({ day, isToday }: { day: any; isToday: boolean }) {
  const date = new Date(day.date);
  const dayName = date.toLocaleDateString('en-US', { weekday: 'short' });

  return (
    <div
      style={{
        padding: '12px 16px',
        background: isToday ? 'rgba(99, 102, 241, 0.15)' : 'rgba(255, 255, 255, 0.05)',
        borderRadius: '8px',
        border: isToday ? '1px solid rgba(99, 102, 241, 0.4)' : 'none',
      }}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '8px' }}>
        <div style={{ fontSize: '13px', color: 'var(--text-primary)', fontWeight: '600' }}>
          {dayName} {isToday && '(Today)'}
        </div>
        <div style={{ fontSize: '12px', color: 'var(--text-muted)' }}>
          {day.suggestions_accepted}/{day.suggestions_shown} accepted
        </div>
      </div>
      <div
        style={{
          width: '100%',
          height: '6px',
          background: 'rgba(255, 255, 255, 0.1)',
          borderRadius: '3px',
          overflow: 'hidden',
        }}
      >
        <div
          style={{
            width: `${day.acceptance_rate}%`,
            height: '100%',
            background: `linear-gradient(90deg, #10b981, #6366f1)`,
            borderRadius: '3px',
            transition: 'width 0.3s ease',
          }}
        />
      </div>
    </div>
  );
}

function TrendCard({
  title,
  data,
  color,
  suffix,
}: {
  title: string;
  data: any[];
  color: string;
  suffix: string;
}) {
  const recentData = data.slice(-7);
  const maxValue = Math.max(...recentData.map((d) => d.value), 1);
  const latestValue = recentData[recentData.length - 1]?.value || 0;

  return (
    <div
      style={{
        padding: '20px',
        background: 'rgba(255, 255, 255, 0.05)',
        borderRadius: 'var(--radius-lg)',
      }}
    >
      <div style={{ marginBottom: '12px' }}>
        <div style={{ fontSize: '13px', color: 'var(--text-muted)', marginBottom: '4px' }}>
          {title}
        </div>
        <div style={{ fontSize: '28px', fontWeight: '700', color }}>
          {Math.round(latestValue)}
          {suffix}
        </div>
      </div>
      <div style={{ display: 'flex', alignItems: 'flex-end', gap: '4px', height: '60px' }}>
        {recentData.map((point, i) => (
          <div
            key={i}
            style={{
              flex: 1,
              height: `${(point.value / maxValue) * 100}%`,
              background: color,
              borderRadius: '2px',
              minHeight: '4px',
            }}
          />
        ))}
      </div>
    </div>
  );
}
