/**
 * Learn by Doing Module
 * Records workflows and generates tutorials automatically
 */

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use tracing::{info, warn};

pub mod exporter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowAction {
    pub timestamp: i64,
    pub action_type: ActionType,
    pub app_name: String,
    pub description: String,
    pub screenshot_path: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionType {
    AppSwitch { from_app: String, to_app: String },
    KeyboardInput { keys: String },
    MouseClick { x: i32, y: i32, button: String },
    ToolUsed { tool_name: String },
    FileOperation { operation: String, file_path: String },
    Command { command: String },
    Comment { text: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub title: String,
    pub description: String,
    pub start_time: i64,
    pub end_time: Option<i64>,
    pub duration_minutes: u32,
    pub actions: Vec<WorkflowAction>,
    pub tags: Vec<String>,
    pub app_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tutorial {
    pub id: String,
    pub workflow_id: String,
    pub title: String,
    pub description: String,
    pub steps: Vec<TutorialStep>,
    pub estimated_duration: u32,
    pub difficulty: String,
    pub tags: Vec<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialStep {
    pub step_number: usize,
    pub title: String,
    pub description: String,
    pub screenshot_path: Option<String>,
    pub code_snippet: Option<String>,
    pub tips: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingState {
    pub is_recording: boolean,
    pub current_workflow_id: Option<String>,
    pub actions_recorded: usize,
    pub recording_duration_seconds: u32,
}

pub struct LearnManager {
    is_recording: bool,
    current_workflow: Option<Workflow>,
    workflows: Vec<Workflow>,
    tutorials: Vec<Tutorial>,
    action_buffer: VecDeque<WorkflowAction>,
    max_workflows: usize,
}

impl LearnManager {
    pub fn new(max_workflows: usize) -> Self {
        Self {
            is_recording: false,
            current_workflow: None,
            workflows: Vec::new(),
            tutorials: Vec::new(),
            action_buffer: VecDeque::new(),
            max_workflows,
        }
    }

    pub fn start_recording(&mut self, title: &str, app_name: &str) -> Result<String, String> {
        if self.is_recording {
            return Err("Already recording a workflow".to_string());
        }

        let now = Utc::now().timestamp();
        let workflow = Workflow {
            id: format!("workflow_{}", now),
            title: title.to_string(),
            description: String::new(),
            start_time: now,
            end_time: None,
            duration_minutes: 0,
            actions: Vec::new(),
            tags: Vec::new(),
            app_name: app_name.to_string(),
        };

        let workflow_id = workflow.id.clone();
        self.current_workflow = Some(workflow);
        self.is_recording = true;
        self.action_buffer.clear();

        info!("🎥 Started recording workflow: {}", title);
        Ok(workflow_id)
    }

    pub fn stop_recording(&mut self) -> Result<Workflow, String> {
        if !self.is_recording {
            return Err("Not currently recording".to_string());
        }

        let mut workflow = self.current_workflow.take()
            .ok_or("No active workflow".to_string())?;

        let now = Utc::now().timestamp();
        workflow.end_time = Some(now);
        workflow.duration_minutes = ((now - workflow.start_time) / 60) as u32;

        // Add buffered actions
        workflow.actions = self.action_buffer.drain(..).collect();

        // Trim old workflows if needed
        if self.workflows.len() >= self.max_workflows {
            self.workflows.remove(0);
        }

        self.workflows.push(workflow.clone());
        self.is_recording = false;

        info!("⏹️ Stopped recording. {} actions recorded", workflow.actions.len());
        Ok(workflow)
    }

    pub fn record_action(&mut self, action: WorkflowAction) {
        if !self.is_recording {
            return;
        }

        self.action_buffer.push_back(action);

        // Limit buffer size
        if self.action_buffer.len() > 1000 {
            self.action_buffer.pop_front();
        }
    }

    pub fn add_comment(&mut self, comment: &str) -> Result<(), String> {
        if !self.is_recording {
            return Err("Not recording".to_string());
        }

        let action = WorkflowAction {
            timestamp: Utc::now().timestamp(),
            action_type: ActionType::Comment { text: comment.to_string() },
            app_name: self.current_workflow.as_ref().map(|w| w.app_name.clone()).unwrap_or_default(),
            description: format!("💬 {}", comment),
            screenshot_path: None,
            metadata: serde_json::json!({"comment": comment}),
        };

        self.record_action(action);
        Ok(())
    }

    pub fn generate_tutorial(&mut self, workflow_id: &str) -> Result<Tutorial, String> {
        let workflow = self.workflows.iter()
            .find(|w| w.id == workflow_id)
            .ok_or("Workflow not found".to_string())?
            .clone();

        // Group actions into logical steps
        let steps = self.group_into_steps(&workflow.actions);

        let tutorial = Tutorial {
            id: format!("tutorial_{}", Utc::now().timestamp()),
            workflow_id: workflow.id.clone(),
            title: workflow.title.clone(),
            description: format!("Learn how to {}", workflow.title.to_lowercase()),
            steps,
            estimated_duration: workflow.duration_minutes,
            difficulty: self.calculate_difficulty(&workflow),
            tags: workflow.tags.clone(),
            created_at: Utc::now().timestamp(),
        };

        self.tutorials.push(tutorial.clone());

        info!("📚 Generated tutorial: {}", tutorial.title);
        Ok(tutorial)
    }

    fn group_into_steps(&self, actions: &[WorkflowAction]) -> Vec<TutorialStep> {
        let mut steps = Vec::new();
        let mut current_step_actions = Vec::new();
        let mut step_number = 1;

        for (i, action) in actions.iter().enumerate() {
            current_step_actions.push(action.clone());

            // Create a new step when we encounter a comment or every N actions
            let should_create_step = matches!(action.action_type, ActionType::Comment { .. })
                || current_step_actions.len() >= 10
                || i == actions.len() - 1;

            if should_create_step && !current_step_actions.is_empty() {
                let step = self.create_step(step_number, &current_step_actions);
                steps.push(step);
                step_number += 1;
                current_step_actions.clear();
            }
        }

        steps
    }

    fn create_step(&self, step_number: usize, actions: &[WorkflowAction]) -> TutorialStep {
        // Use comment as title if available
        let title = actions.iter()
            .find_map(|a| match &a.action_type {
                ActionType::Comment { text } => Some(text.clone()),
                _ => None,
            })
            .unwrap_or_else(|| format!("Step {}", step_number));

        // Generate description from actions
        let description = self.describe_actions(actions);

        // Find screenshot
        let screenshot_path = actions.iter()
            .find_map(|a| a.screenshot_path.clone());

        // Extract code snippets
        let code_snippet = actions.iter()
            .find_map(|a| match &a.action_type {
                ActionType::Command { command } => Some(command.clone()),
                _ => None,
            });

        // Generate tips
        let tips = self.generate_tips(actions);

        TutorialStep {
            step_number,
            title,
            description,
            screenshot_path,
            code_snippet,
            tips,
        }
    }

    fn describe_actions(&self, actions: &[WorkflowAction]) -> String {
        let mut descriptions = Vec::new();

        for action in actions {
            let desc = match &action.action_type {
                ActionType::AppSwitch { to_app, .. } => format!("Open {}", to_app),
                ActionType::ToolUsed { tool_name } => format!("Use {}", tool_name),
                ActionType::FileOperation { operation, .. } => format!("{} file", operation),
                ActionType::Command { command } => format!("Run: `{}`", command),
                ActionType::Comment { .. } => continue, // Skip comments
                _ => continue,
            };

            if !descriptions.contains(&desc) {
                descriptions.push(desc);
            }
        }

        descriptions.join(", ")
    }

    fn generate_tips(&self, actions: &[WorkflowAction]) -> Vec<String> {
        let mut tips = Vec::new();

        // Look for keyboard shortcuts
        if actions.iter().any(|a| matches!(a.action_type, ActionType::KeyboardInput { .. })) {
            tips.push("💡 Use keyboard shortcuts to work faster".to_string());
        }

        // Look for repetitive actions
        if actions.len() > 15 {
            tips.push("⚡ Consider automating repetitive tasks".to_string());
        }

        tips
    }

    fn calculate_difficulty(&self, workflow: &Workflow) -> String {
        let action_count = workflow.actions.len();

        if action_count < 10 {
            "Beginner".to_string()
        } else if action_count < 30 {
            "Intermediate".to_string()
        } else {
            "Advanced".to_string()
        }
    }

    pub fn get_recording_state(&self) -> RecordingState {
        if let Some(ref workflow) = self.current_workflow {
            let duration = (Utc::now().timestamp() - workflow.start_time) as u32;
            RecordingState {
                is_recording: true,
                current_workflow_id: Some(workflow.id.clone()),
                actions_recorded: self.action_buffer.len(),
                recording_duration_seconds: duration,
            }
        } else {
            RecordingState {
                is_recording: false,
                current_workflow_id: None,
                actions_recorded: 0,
                recording_duration_seconds: 0,
            }
        }
    }

    pub fn get_workflows(&self) -> Vec<Workflow> {
        self.workflows.clone()
    }

    pub fn get_tutorials(&self) -> Vec<Tutorial> {
        self.tutorials.clone()
    }

    pub fn export_tutorial_markdown(&self, tutorial_id: &str) -> Result<String, String> {
        let tutorial = self.tutorials.iter()
            .find(|t| t.id == tutorial_id)
            .ok_or("Tutorial not found".to_string())?;

        Ok(exporter::export_markdown(tutorial))
    }
}

// Tauri Commands
#[tauri::command]
pub async fn start_workflow_recording(
    title: String,
    app_name: String,
    learn_manager: State<'_, Arc<Mutex<LearnManager>>>,
) -> Result<String, String> {
    let mut manager = learn_manager.lock().await;
    manager.start_recording(&title, &app_name)
}

#[tauri::command]
pub async fn stop_workflow_recording(
    learn_manager: State<'_, Arc<Mutex<LearnManager>>>,
) -> Result<Workflow, String> {
    let mut manager = learn_manager.lock().await;
    manager.stop_recording()
}

#[tauri::command]
pub async fn add_workflow_comment(
    comment: String,
    learn_manager: State<'_, Arc<Mutex<LearnManager>>>,
) -> Result<(), String> {
    let mut manager = learn_manager.lock().await;
    manager.add_comment(&comment)
}

#[tauri::command]
pub async fn generate_workflow_tutorial(
    workflow_id: String,
    learn_manager: State<'_, Arc<Mutex<LearnManager>>>,
) -> Result<Tutorial, String> {
    let mut manager = learn_manager.lock().await;
    manager.generate_tutorial(&workflow_id)
}

#[tauri::command]
pub async fn get_recording_state(
    learn_manager: State<'_, Arc<Mutex<LearnManager>>>,
) -> Result<RecordingState, String> {
    let manager = learn_manager.lock().await;
    Ok(manager.get_recording_state())
}

#[tauri::command]
pub async fn get_all_workflows(
    learn_manager: State<'_, Arc<Mutex<LearnManager>>>,
) -> Result<Vec<Workflow>, String> {
    let manager = learn_manager.lock().await;
    Ok(manager.get_workflows())
}

#[tauri::command]
pub async fn get_all_tutorials(
    learn_manager: State<'_, Arc<Mutex<LearnManager>>>,
) -> Result<Vec<Tutorial>, String> {
    let manager = learn_manager.lock().await;
    Ok(manager.get_tutorials())
}

#[tauri::command]
pub async fn export_tutorial_as_markdown(
    tutorial_id: String,
    learn_manager: State<'_, Arc<Mutex<LearnManager>>>,
) -> Result<String, String> {
    let manager = learn_manager.lock().await;
    manager.export_tutorial_markdown(&tutorial_id)
}
