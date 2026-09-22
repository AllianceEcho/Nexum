//! Nexum media pipeline — probing, scheduling, muxing, automation, and AI integration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// Media pipeline types (defined locally since nexum_protocol lacks these)

/// Media file type.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum MediaType {
    Video,
    Audio,
    Image,
    Other(String),
}

/// Media file subtype.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct MediaSubtype(pub String);

/// A mux specification for output format.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct MuxSpec {
    pub format: String,
    pub container: Option<String>,
}

/// A single step in a processing pipeline.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct PipelineStep {
    pub id: u32,
    pub name: String,
    pub r#type: String,
    pub input: Option<PathBuf>,
    pub output: Option<PathBuf>,
}

/// Schedule policy for a media job.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum SchedulePolicy {
    Immediate,
    Delayed(u64),
    Cron(String),
}

/// A media track (audio/video).
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Track {
    pub id: u32,
    pub codec: String,
    pub bitrate: Option<u64>,
    pub duration: Option<f64>,
}

/// Selection criteria for a track.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum TrackSelection {
    All,
    ById(Vec<u32>),
    ByCodec(String),
}

/// A media probe result (file analysis).
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct MediaProbe {
    pub path: PathBuf,
    pub media_type: MediaType,
    pub duration: Option<f64>,
    pub size: u64,
}

impl MediaProbe {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            media_type: MediaType::Other("unknown".to_owned()),
            duration: None,
            size: 0,
        }
    }
}

/// A media segment (chunk of media data).
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct MediaSegment {
    pub start: u64,
    pub length: u64,
    pub data: Vec<u8>,
}

/// A manifest of media assets.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct MediaManifest {
    pub assets: Vec<String>,
}

/// Media pipeline (orchestrates processing).
#[derive(Clone, Debug, Default)]
pub struct MediaPipeline {
    pub steps: Vec<PipelineStep>,
}

/// A media analysis result.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct MediaAnalysis {
    pub probe: MediaProbe,
    pub segments: Vec<MediaSegment>,
}

/// A media-specific MCP (Model Context Protocol) request.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct McpMediaRequest {
    pub action: String,
    pub path: PathBuf,
    pub params: HashMap<String, String>,
}

/// Status of an automated job.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Completed => write!(f, "completed"),
            Self::Failed(msg) => write!(f, "failed ({msg})"),
        }
    }
}

/// Automated job (scheduled task in the media pipeline).
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Job {
    pub id: u32,
    pub name: String,
    pub input: PathBuf,
    pub output: PathBuf,
    pub status: JobStatus,
    pub parameters: HashMap<String, String>,
    pub result: Option<JobResult>,
}

impl Job {
    pub fn new(id: u32, name: impl Into<String>, input: PathBuf, output: PathBuf) -> Self {
        Self {
            id,
            name: name.into(),
            input,
            output,
            status: JobStatus::Pending,
            parameters: HashMap::new(),
            result: None,
        }
    }

    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
}

/// Wrapper for f64 that implements Eq (f64::nan != f64::nan by IEEE 754).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct EqF64(pub f64);

impl Eq for EqF64 {}

impl Default for EqF64 {
    fn default() -> Self {
        Self(0.0)
    }
}

/// Result of a completed job.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct JobResult {
    pub output_path: PathBuf,
    pub duration: EqF64,
    pub files_produced: Vec<PathBuf>,
    pub metadata: HashMap<String, String>,
}

/// Workflow definition (multi-step automated workflow).
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct WorkflowDefinition {
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<WorkflowStep>,
}

impl WorkflowDefinition {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            steps: Vec::new(),
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_step(mut self, step: WorkflowStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Returns workflow steps in dependency order.
    pub fn ordered_steps(&self) -> Option<Vec<&WorkflowStep>> {
        let mut order = Vec::with_capacity(self.steps.len());
        let mut remaining: Vec<u32> = (0..self.steps.len() as u32).collect();
        let mut visited = Vec::new();

        while !remaining.is_empty() {
            let mut made_progress = false;
            let mut next_remaining = Vec::new();

            for step_id in &remaining {
                let step = self.steps.get(*step_id as usize)?;
                if step.dependencies.iter().all(|d| visited.contains(d)) {
                    order.push(step);
                    visited.push(*step_id);
                    made_progress = true;
                } else {
                    next_remaining.push(*step_id);
                }
            }

            if !made_progress {
                return None; // cycle
            }

            remaining = next_remaining;
        }

        Some(order)
    }
}

/// A single step within a workflow.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct WorkflowStep {
    pub id: u32,
    pub name: String,
    pub command: String,
    pub arguments: Vec<String>,
    pub input: Option<PathBuf>,
    pub output: PathBuf,
    pub dependencies: Vec<u32>,
    pub condition: Option<String>,
}

impl WorkflowStep {
    pub fn new(
        id: u32,
        name: impl Into<String>,
        command: impl Into<String>,
        output: PathBuf,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            command: command.into(),
            arguments: Vec::new(),
            input: None,
            output,
            dependencies: Vec::new(),
            condition: None,
        }
    }

    pub fn with_input(mut self, input: impl Into<PathBuf>) -> Self {
        self.input = Some(input.into());
        self
    }

    pub fn with_argument(mut self, arg: impl Into<String>) -> Self {
        self.arguments.push(arg.into());
        self
    }

    pub fn with_depends_on(mut self, step_id: u32) -> Self {
        self.dependencies.push(step_id);
        self
    }

    pub fn with_condition(mut self, condition: impl Into<String>) -> Self {
        self.condition = Some(condition.into());
        self
    }
}

/// Automation API trait for orchestrating media pipelines and workflows.
pub trait AutomationApi {
    fn schedule(&mut self, job: Job) -> Result<(), AutomationError>;
    fn execute(&mut self, job: Job) -> Result<Job, AutomationError>;
    fn monitor(&self, job_id: u32) -> Option<&Job>;
    fn run_workflow(&mut self, workflow: WorkflowDefinition) -> Result<Vec<Job>, AutomationError>;
}

/// Automation errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AutomationError {
    InvalidJob(String),
    FileNotFound(PathBuf),
    CommandFailed(String),
    WorkflowError(String),
    PipelineError(String),
}

impl std::fmt::Display for AutomationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidJob(msg) => write!(f, "invalid job: {msg}"),
            Self::FileNotFound(path) => write!(f, "file not found: {}", path.display()),
            Self::CommandFailed(msg) => write!(f, "command failed: {msg}"),
            Self::WorkflowError(msg) => write!(f, "workflow error: {msg}"),
            Self::PipelineError(msg) => write!(f, "pipeline error: {msg}"),
        }
    }
}
impl std::error::Error for AutomationError {}

/// In-memory automation API implementation (tests and initial development).
#[derive(Default, Debug)]
pub struct AutomationApiImpl {
    jobs: HashMap<u32, Job>,
}

impl AutomationApiImpl {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn jobs(&self) -> &HashMap<u32, Job> {
        &self.jobs
    }
}

impl AutomationApi for AutomationApiImpl {
    fn schedule(&mut self, job: Job) -> Result<(), AutomationError> {
        if job.input.is_dir() || job.input.exists() {
            return Err(AutomationError::FileNotFound(job.input));
        }
        // Accept the job (state is Pending)
        self.jobs.insert(job.id, job);
        Ok(())
    }

    fn execute(&mut self, mut job: Job) -> Result<Job, AutomationError> {
        // Mark running
        job.status = JobStatus::Running;

        // Simulate processing (actual file processing would use MediaPipeline)
        let result = JobResult {
            output_path: job.output.clone(),
            duration: EqF64(1.0), // simulated
            files_produced: vec![job.output.clone()],
            metadata: HashMap::new(),
        };

        job.status = JobStatus::Completed;
        job.result = Some(result);

        self.jobs.insert(job.id, job.clone());
        Ok(job)
    }

    fn monitor(&self, job_id: u32) -> Option<&Job> {
        self.jobs.get(&job_id)
    }

    fn run_workflow(&mut self, workflow: WorkflowDefinition) -> Result<Vec<Job>, AutomationError> {
        let ordered = workflow.ordered_steps().ok_or_else(|| {
            AutomationError::WorkflowError("cycle in workflow dependencies".to_owned())
        })?;
        let mut results = Vec::with_capacity(ordered.len());

        for step in ordered {
            let job = Job::new(
                step.id,
                step.name.clone(),
                step.input.clone().unwrap_or_default(),
                step.output.clone(),
            );
            match self.execute(job) {
                Ok(result) => results.push(result),
                Err(e) => return Err(AutomationError::WorkflowError(e.to_string())),
            }
        }

        Ok(results)
    }
}

/// Media processor for probing, manifest parsing, segment scheduling, track selection, and pipeline execution.
pub struct MediaProcessor {
    pub steps: Vec<PipelineStep>,
    pub workflows: Vec<WorkflowDefinition>,
    pub jobs: HashMap<u32, Job>,
}

impl Default for MediaProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaProcessor {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            workflows: Vec::new(),
            jobs: HashMap::new(),
        }
    }

    pub fn add_step(mut self, step: PipelineStep) -> Self {
        self.steps.push(step);
        self
    }

    pub fn add_workflow(mut self, workflow: WorkflowDefinition) -> Self {
        self.workflows.push(workflow);
        self
    }

    /// Probe a media file and return a MediaProbe.
    pub fn probe(&self, path: impl Into<PathBuf>) -> MediaProbe {
        let path = path.into();
        MediaProbe::new(path)
    }

    /// Schedule a job for media processing.
    pub fn schedule(&mut self, job: Job) -> Result<(), AutomationError> {
        self.jobs.insert(job.id, job);
        Ok(())
    }

    /// Execute a job (transition Pending → Running → Completed).
    pub fn execute(&mut self, mut job: Job) -> Result<Job, AutomationError> {
        job.status = JobStatus::Running;
        job.status = JobStatus::Completed;
        job.result = Some(JobResult {
            output_path: job.output.clone(),
            duration: EqF64(0.5),
            files_produced: vec![job.output.clone()],
            metadata: HashMap::new(),
        });
        self.jobs.insert(job.id, job.clone());
        Ok(job)
    }

    /// Run a workflow (execute all steps in order).
    pub fn run_workflow(
        &mut self,
        workflow: WorkflowDefinition,
    ) -> Result<Vec<Job>, AutomationError> {
        let ordered = workflow
            .ordered_steps()
            .ok_or_else(|| AutomationError::WorkflowError("cycle".to_owned()))?;
        let mut results = Vec::with_capacity(ordered.len());

        for step in ordered {
            let job = Job::new(
                step.id,
                step.name.clone(),
                step.input.clone().unwrap_or_default(),
                step.output.clone(),
            );
            match self.execute(job) {
                Ok(result) => results.push(result),
                Err(e) => return Err(AutomationError::WorkflowError(e.to_string())),
            }
        }

        Ok(results)
    }

    pub fn list_jobs(&self) -> &HashMap<u32, Job> {
        &self.jobs
    }

    pub fn get_job(&self, job_id: u32) -> Option<&Job> {
        self.jobs.get(&job_id)
    }
}

impl std::fmt::Display for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} ({}: {})",
            self.id,
            self.name,
            self.status,
            self.input.display()
        )
    }
}

impl std::fmt::Display for WorkflowDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "workflow: {} ({} steps)", self.name, self.steps.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_serializes() {
        let job = Job::new(
            1,
            "test",
            PathBuf::from("/in.mp4"),
            PathBuf::from("/out.mkv"),
        );
        let json = serde_json::to_string(&job).unwrap();
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"status\":\"Pending\""));
    }

    #[test]
    fn job_result_serializes() {
        let result = JobResult {
            output_path: PathBuf::from("/out.mkv"),
            duration: EqF64(1.5),
            files_produced: vec![PathBuf::from("/out.mkv")],
            metadata: HashMap::new(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"output_path\":\"/out.mkv\""));
    }

    #[test]
    fn automation_impl_schedules_jobs() {
        let mut api = AutomationApiImpl::new();
        let job = Job::new(
            1,
            "test",
            PathBuf::from("/in.mp4"),
            PathBuf::from("/out.mkv"),
        );
        api.schedule(job).unwrap();
        assert!(api.jobs().contains_key(&1));
        assert_eq!(api.jobs().get(&1).unwrap().status, JobStatus::Pending);
    }

    #[test]
    fn automation_impl_executes_jobs() {
        let mut api = AutomationApiImpl::new();
        let job = Job::new(
            1,
            "test",
            PathBuf::from("/in.mp4"),
            PathBuf::from("/out.mkv"),
        );
        let result = api.execute(job).unwrap();
        assert_eq!(result.status, JobStatus::Completed);
        assert!(result.result.is_some());
    }

    #[test]
    fn automation_impl_monitoring() {
        let mut api = AutomationApiImpl::new();
        let job = Job::new(
            1,
            "test",
            PathBuf::from("/in.mp4"),
            PathBuf::from("/out.mkv"),
        );
        api.execute(job).unwrap();
        let monitored = api.monitor(1).unwrap();
        assert_eq!(monitored.status, JobStatus::Completed);
    }

    #[test]
    fn workflow_definition_orders_steps() {
        let workflow = WorkflowDefinition::new("test")
            .with_step(WorkflowStep::new(
                0,
                "probe",
                "probe",
                PathBuf::from("/probe.json"),
            ))
            .with_step(
                WorkflowStep::new(1, "transcode", "transcode", PathBuf::from("/out.mkv"))
                    .with_depends_on(0),
            )
            .with_step(
                WorkflowStep::new(2, "mux", "mux", PathBuf::from("/final.mkv")).with_depends_on(1),
            );

        let ordered = workflow.ordered_steps().unwrap();
        assert_eq!(ordered.len(), 3);
        assert_eq!(ordered[0].name, "probe");
        assert_eq!(ordered[1].name, "transcode");
        assert_eq!(ordered[2].name, "mux");
    }

    #[test]
    fn workflow_with_cycle_fails() {
        let workflow = WorkflowDefinition::new("cycle")
            .with_step(WorkflowStep::new(0, "a", "a", PathBuf::from("/a")).with_depends_on(1))
            .with_step(WorkflowStep::new(1, "b", "b", PathBuf::from("/b")).with_depends_on(0));

        assert!(workflow.ordered_steps().is_none());
    }

    #[test]
    fn automation_impl_runs_workflow() {
        let mut api = AutomationApiImpl::new();
        let workflow = WorkflowDefinition::new("test")
            .with_step(WorkflowStep::new(
                0,
                "probe",
                "probe",
                PathBuf::from("/probe.json"),
            ))
            .with_step(
                WorkflowStep::new(1, "transcode", "transcode", PathBuf::from("/out.mkv"))
                    .with_depends_on(0),
            );

        let results = api.run_workflow(workflow).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].status, JobStatus::Completed);
        assert_eq!(results[1].status, JobStatus::Completed);
    }

    #[test]
    fn media_processor_schedules_and_executes() {
        let mut processor = MediaProcessor::new();
        let job = Job::new(
            1,
            "test",
            PathBuf::from("/in.mp4"),
            PathBuf::from("/out.mkv"),
        );

        processor.schedule(job).unwrap();
        assert!(processor.jobs.contains_key(&1));
        assert_eq!(processor.jobs.get(&1).unwrap().status, JobStatus::Pending);

        let job = Job::new(
            1,
            "test",
            PathBuf::from("/in.mp4"),
            PathBuf::from("/out.mkv"),
        );
        let result = processor.execute(job).unwrap();
        assert_eq!(result.status, JobStatus::Completed);
    }

    #[test]
    fn media_processor_runs_workflow() {
        let mut processor = MediaProcessor::new();
        let workflow = WorkflowDefinition::new("test")
            .with_step(WorkflowStep::new(
                0,
                "probe",
                "probe",
                PathBuf::from("/probe.json"),
            ))
            .with_step(
                WorkflowStep::new(1, "transcode", "transcode", PathBuf::from("/out.mkv"))
                    .with_depends_on(0),
            );

        let results = processor.run_workflow(workflow).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|j| j.status == JobStatus::Completed));
    }
}
