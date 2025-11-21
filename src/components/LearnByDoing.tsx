import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Workflow {
  id: string;
  title: string;
  description: string;
  start_time: number;
  end_time?: number;
  duration_minutes: number;
  app_name: string;
  tags: string[];
  actions: WorkflowAction[];
}

interface WorkflowAction {
  timestamp: number;
  action_type: ActionType;
  app_name: string;
  description: string;
  screenshot_path?: string;
  metadata: any;
}

type ActionType =
  | { type: 'app_switch'; from_app: string; to_app: string }
  | { type: 'keyboard_input'; keys: string }
  | { type: 'mouse_click'; x: number; y: number; button: string }
  | { type: 'tool_used'; tool_name: string }
  | { type: 'file_operation'; operation: string; file_path: string }
  | { type: 'command'; command: string }
  | { type: 'comment'; text: string };

interface Tutorial {
  id: string;
  workflow_id: string;
  title: string;
  description: string;
  difficulty: string;
  estimated_duration: number;
  tags: string[];
  steps: TutorialStep[];
}

interface TutorialStep {
  step_number: number;
  title: string;
  description: string;
  screenshot_path?: string;
  code_snippet?: string;
  tips: string[];
}

interface RecordingState {
  is_recording: boolean;
  current_workflow_id?: string;
  actions_recorded: number;
  recording_duration_seconds: number;
}

export function LearnByDoing() {
  const [isRecording, setIsRecording] = useState(false);
  const [recordingTitle, setRecordingTitle] = useState('');
  const [appName, setAppName] = useState('');
  const [workflows, setWorkflows] = useState<Workflow[]>([]);
  const [tutorials, setTutorials] = useState<Tutorial[]>([]);
  const [markdown, setMarkdown] = useState<string>('');
  const [error, setError] = useState<string>('');

  // Add step form
  const [stepTitle, setStepTitle] = useState('');
  const [stepDescription, setStepDescription] = useState('');
  const [stepCommand, setStepCommand] = useState('');

  useEffect(() => {
    loadData();
    const interval = setInterval(loadData, 2000);
    return () => clearInterval(interval);
  }, []);

  const loadData = async () => {
    try {
      const state = await invoke<RecordingState>('get_recording_state');
      setIsRecording(state.is_recording);

      const wf = await invoke<Workflow[]>('get_all_workflows');
      setWorkflows(wf);

      const tut = await invoke<Tutorial[]>('get_all_tutorials');
      setTutorials(tut);
    } catch (err) {
      console.error('Error loading data:', err);
    }
  };

  const startRecording = async () => {
    if (!recordingTitle.trim()) {
      setError('Please enter a workflow title');
      return;
    }
    if (!appName.trim()) {
      setError('Please enter an app name');
      return;
    }

    try {
      await invoke<string>('start_workflow_recording', {
        title: recordingTitle,
        appName: appName,
      });
      setError('✅ Recording started!');
      setTimeout(() => setError(''), 2000);
      setRecordingTitle('');
      setAppName('');
      await loadData();
    } catch (err) {
      setError(`Failed to start recording: ${err}`);
    }
  };

  const stopRecording = async () => {
    try {
      await invoke<Workflow>('stop_workflow_recording');
      setError('✅ Recording stopped!');
      setTimeout(() => setError(''), 2000);
      await loadData();
    } catch (err) {
      setError(`Failed to stop recording: ${err}`);
    }
  };

  const addStep = async () => {
    if (!stepTitle.trim()) {
      setError('Please enter a step title');
      return;
    }

    try {
      // Add as comment (which becomes step title)
      await invoke('add_workflow_comment', {
        comment: stepTitle,
      });

      // If there's a command, add it
      if (stepCommand.trim()) {
        // We'll add the command description to the comment
        await invoke('add_workflow_comment', {
          comment: `Command: ${stepCommand}`,
        });
      }

      // If there's a description, add it
      if (stepDescription.trim()) {
        await invoke('add_workflow_comment', {
          comment: `Description: ${stepDescription}`,
        });
      }

      setStepTitle('');
      setStepDescription('');
      setStepCommand('');
      setError('✅ Step added!');
      setTimeout(() => setError(''), 1500);
    } catch (err) {
      setError(`Failed to add step: ${err}`);
    }
  };

  const generateTutorial = async (workflowId: string) => {
    try {
      await invoke<Tutorial>('generate_workflow_tutorial', {
        workflowId,
      });
      setError('✅ Tutorial generated!');
      setTimeout(() => setError(''), 2000);
      await loadData();
    } catch (err) {
      setError(`Failed to generate tutorial: ${err}`);
    }
  };

  const exportMarkdown = async (tutorialId: string) => {
    try {
      const md = await invoke<string>('export_tutorial_as_markdown', {
        tutorialId,
      });
      setMarkdown(md);
      setError('');
    } catch (err) {
      setError(`Failed to export markdown: ${err}`);
    }
  };

  return (
    <div style={{
      padding: '20px',
      maxWidth: '1400px',
      margin: '0 auto',
      fontFamily: 'system-ui, -apple-system, sans-serif',
      color: '#fff',
      backgroundColor: '#0a0a0a',
      minHeight: '100vh',
    }}>
      <div style={{ marginBottom: '30px' }}>
        <h1 style={{ margin: '0 0 10px 0', fontSize: '32px', fontWeight: 'bold' }}>
          📚 Learn by Doing
        </h1>
        <p style={{ margin: 0, color: '#888', fontSize: '16px' }}>
          Record your workflow → Generate tutorial automatically
        </p>
      </div>

      {error && (
        <div style={{
          padding: '12px 16px',
          marginBottom: '20px',
          backgroundColor: error.startsWith('✅') ? '#065f46' : '#7f1d1d',
          borderRadius: '6px',
          fontSize: '14px',
        }}>
          {error}
        </div>
      )}

      {/* Recording Controls */}
      <div style={{
        backgroundColor: '#1a1a1a',
        padding: '24px',
        borderRadius: '12px',
        marginBottom: '30px',
        border: isRecording ? '2px solid #ef4444' : '1px solid #2a2a2a',
      }}>
        <h2 style={{ margin: '0 0 20px 0', fontSize: '20px', display: 'flex', alignItems: 'center', gap: '10px' }}>
          {isRecording ? (
            <>
              <span style={{
                width: '12px',
                height: '12px',
                backgroundColor: '#ef4444',
                borderRadius: '50%',
                animation: 'pulse 1.5s ease-in-out infinite',
              }}></span>
              Recording...
            </>
          ) : (
            'Start New Recording'
          )}
        </h2>

        {!isRecording ? (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
            <input
              type="text"
              placeholder="Workflow title (e.g., 'Setup Tailwind CSS')"
              value={recordingTitle}
              onChange={(e) => setRecordingTitle(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && startRecording()}
              style={{
                padding: '12px 16px',
                backgroundColor: '#2a2a2a',
                border: '1px solid #3a3a3a',
                borderRadius: '6px',
                color: '#fff',
                fontSize: '14px',
              }}
            />
            <input
              type="text"
              placeholder="App name (e.g., 'VS Code', 'Terminal')"
              value={appName}
              onChange={(e) => setAppName(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && startRecording()}
              style={{
                padding: '12px 16px',
                backgroundColor: '#2a2a2a',
                border: '1px solid #3a3a3a',
                borderRadius: '6px',
                color: '#fff',
                fontSize: '14px',
              }}
            />
            <button
              onClick={startRecording}
              style={{
                padding: '12px 24px',
                backgroundColor: '#10b981',
                border: 'none',
                borderRadius: '6px',
                color: '#fff',
                fontSize: '14px',
                cursor: 'pointer',
                fontWeight: '600',
                transition: 'all 0.2s',
              }}
              onMouseEnter={(e) => e.currentTarget.style.backgroundColor = '#059669'}
              onMouseLeave={(e) => e.currentTarget.style.backgroundColor = '#10b981'}
            >
              🎬 Start Recording
            </button>
          </div>
        ) : (
          <div>
            {/* Add Step Form */}
            <div style={{
              backgroundColor: '#2a2a2a',
              padding: '20px',
              borderRadius: '8px',
              marginBottom: '20px',
            }}>
              <h3 style={{ margin: '0 0 15px 0', fontSize: '16px', color: '#10b981' }}>
                Add Step to Recording
              </h3>
              <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
                <input
                  type="text"
                  placeholder="Step title (e.g., 'Install Tailwind')"
                  value={stepTitle}
                  onChange={(e) => setStepTitle(e.target.value)}
                  style={{
                    padding: '10px 14px',
                    backgroundColor: '#1a1a1a',
                    border: '1px solid #3a3a3a',
                    borderRadius: '4px',
                    color: '#fff',
                    fontSize: '13px',
                  }}
                />
                <textarea
                  placeholder="Description (optional)"
                  value={stepDescription}
                  onChange={(e) => setStepDescription(e.target.value)}
                  rows={2}
                  style={{
                    padding: '10px 14px',
                    backgroundColor: '#1a1a1a',
                    border: '1px solid #3a3a3a',
                    borderRadius: '4px',
                    color: '#fff',
                    fontSize: '13px',
                    fontFamily: 'inherit',
                    resize: 'vertical',
                  }}
                />
                <input
                  type="text"
                  placeholder="Command (optional, e.g., 'npm install -D tailwindcss')"
                  value={stepCommand}
                  onChange={(e) => setStepCommand(e.target.value)}
                  style={{
                    padding: '10px 14px',
                    backgroundColor: '#1a1a1a',
                    border: '1px solid #3a3a3a',
                    borderRadius: '4px',
                    color: '#fff',
                    fontSize: '13px',
                    fontFamily: 'monospace',
                  }}
                />
                <button
                  onClick={addStep}
                  style={{
                    padding: '10px 20px',
                    backgroundColor: '#3b82f6',
                    border: 'none',
                    borderRadius: '4px',
                    color: '#fff',
                    fontSize: '13px',
                    cursor: 'pointer',
                    fontWeight: '500',
                  }}
                >
                  ➕ Add Step
                </button>
              </div>
            </div>

            <button
              onClick={stopRecording}
              style={{
                padding: '12px 24px',
                backgroundColor: '#ef4444',
                border: 'none',
                borderRadius: '6px',
                color: '#fff',
                fontSize: '14px',
                cursor: 'pointer',
                fontWeight: '600',
                width: '100%',
              }}
            >
              ⏹️ Stop Recording
            </button>
          </div>
        )}
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '30px', marginBottom: '30px' }}>
        {/* Workflows */}
        <div>
          <h2 style={{ margin: '0 0 15px 0', fontSize: '18px' }}>
            Recorded Workflows ({workflows.length})
          </h2>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
            {workflows.length === 0 ? (
              <p style={{ color: '#666', fontSize: '14px' }}>
                No workflows yet. Start recording above!
              </p>
            ) : (
              workflows.map((wf) => (
                <div
                  key={wf.id}
                  style={{
                    backgroundColor: '#1a1a1a',
                    padding: '16px',
                    borderRadius: '8px',
                    border: '1px solid #2a2a2a',
                  }}
                >
                  <h3 style={{ margin: '0 0 8px 0', fontSize: '15px', fontWeight: '600' }}>
                    {wf.title}
                  </h3>
                  <p style={{ margin: '0 0 12px 0', color: '#888', fontSize: '13px' }}>
                    {wf.app_name} • {wf.actions.length} actions • {wf.duration_minutes} min
                  </p>
                  <button
                    onClick={() => generateTutorial(wf.id)}
                    style={{
                      padding: '8px 16px',
                      backgroundColor: '#8b5cf6',
                      border: 'none',
                      borderRadius: '4px',
                      color: '#fff',
                      fontSize: '12px',
                      cursor: 'pointer',
                      fontWeight: '500',
                    }}
                  >
                    ✨ Generate Tutorial
                  </button>
                </div>
              ))
            )}
          </div>
        </div>

        {/* Tutorials */}
        <div>
          <h2 style={{ margin: '0 0 15px 0', fontSize: '18px' }}>
            Generated Tutorials ({tutorials.length})
          </h2>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
            {tutorials.length === 0 ? (
              <p style={{ color: '#666', fontSize: '14px' }}>
                No tutorials yet. Generate one from a workflow!
              </p>
            ) : (
              tutorials.map((tut) => (
                <div
                  key={tut.id}
                  style={{
                    backgroundColor: '#1a1a1a',
                    padding: '16px',
                    borderRadius: '8px',
                    border: '1px solid #2a2a2a',
                  }}
                >
                  <h3 style={{ margin: '0 0 8px 0', fontSize: '15px', fontWeight: '600' }}>
                    {tut.title}
                  </h3>
                  <p style={{ margin: '0 0 12px 0', color: '#888', fontSize: '13px' }}>
                    {tut.difficulty} • ~{tut.estimated_duration} min • {tut.steps.length} steps
                  </p>
                  <button
                    onClick={() => exportMarkdown(tut.id)}
                    style={{
                      padding: '8px 16px',
                      backgroundColor: '#06b6d4',
                      border: 'none',
                      borderRadius: '4px',
                      color: '#fff',
                      fontSize: '12px',
                      cursor: 'pointer',
                      fontWeight: '500',
                    }}
                  >
                    📄 Export Markdown
                  </button>
                </div>
              ))
            )}
          </div>
        </div>
      </div>

      {/* Markdown Preview */}
      {markdown && (
        <div style={{ marginTop: '30px' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '15px' }}>
            <h2 style={{ margin: 0, fontSize: '18px' }}>Exported Markdown</h2>
            <button
              onClick={() => {
                navigator.clipboard.writeText(markdown);
                setError('✅ Copied to clipboard!');
                setTimeout(() => setError(''), 2000);
              }}
              style={{
                padding: '8px 16px',
                backgroundColor: '#10b981',
                border: 'none',
                borderRadius: '4px',
                color: '#fff',
                fontSize: '12px',
                cursor: 'pointer',
                fontWeight: '500',
              }}
            >
              📋 Copy to Clipboard
            </button>
          </div>
          <div style={{
            backgroundColor: '#1a1a1a',
            padding: '20px',
            borderRadius: '8px',
            border: '1px solid #2a2a2a',
            overflow: 'auto',
            maxHeight: '600px',
          }}>
            <pre style={{
              margin: 0,
              whiteSpace: 'pre-wrap',
              wordWrap: 'break-word',
              fontSize: '13px',
              lineHeight: '1.6',
              color: '#e5e5e5',
              fontFamily: 'ui-monospace, monospace',
            }}>
              {markdown}
            </pre>
          </div>
        </div>
      )}

      <style>{`
        @keyframes pulse {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.5; }
        }
      `}</style>
    </div>
  );
}
