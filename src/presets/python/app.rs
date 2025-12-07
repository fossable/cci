use crate::error::Result;
use crate::platforms::circleci::models::CircleCIConfig;
use crate::platforms::github::models::{
    GitHubJob, GitHubStep, GitHubTriggerConfig, GitHubTriggers, GitHubWorkflow,
};
use crate::platforms::gitlab::models::GitLabCI;
use crate::platforms::jenkins::models::JenkinsConfig;
use crate::traits::{Detectable, PresetInfo, ToCircleCI, ToGitHub, ToGitLab, ToJenkins};
use std::collections::BTreeMap;

/// Linter tool options for Python
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PythonLinterTool {
    Flake8,
    Ruff,
}

impl PythonLinterTool {
    pub fn name(&self) -> &'static str {
        match self {
            PythonLinterTool::Flake8 => "flake8",
            PythonLinterTool::Ruff => "ruff",
        }
    }

    pub fn check_command(&self) -> &'static str {
        match self {
            PythonLinterTool::Flake8 => "flake8 .",
            PythonLinterTool::Ruff => "ruff check .",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            PythonLinterTool::Flake8 => PythonLinterTool::Ruff,
            PythonLinterTool::Ruff => PythonLinterTool::Flake8,
        }
    }
}

/// Formatter tool options for Python
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PythonFormatterTool {
    Black,
    Ruff,
}

impl PythonFormatterTool {
    pub fn name(&self) -> &'static str {
        match self {
            PythonFormatterTool::Black => "black",
            PythonFormatterTool::Ruff => "ruff",
        }
    }

    pub fn check_command(&self) -> &'static str {
        match self {
            PythonFormatterTool::Black => "black --check .",
            PythonFormatterTool::Ruff => "ruff format --check .",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            PythonFormatterTool::Black => PythonFormatterTool::Ruff,
            PythonFormatterTool::Ruff => PythonFormatterTool::Black,
        }
    }
}

/// Preset for Python application projects
#[derive(Debug, Clone)]
pub struct PythonAppPreset {
    python_version: String,
    enable_linter: bool,
    linter_tool: PythonLinterTool,
    enable_type_check: bool,
    enable_formatter_check: bool,
    formatter_tool: PythonFormatterTool,
}

impl PythonAppPreset {
    /// Create a new builder for PythonAppPreset
    pub fn builder() -> PythonAppPresetBuilder {
        PythonAppPresetBuilder::default()
    }
}

/// Builder for PythonAppPreset
pub struct PythonAppPresetBuilder {
    python_version: Option<String>,
    enable_linter: bool,
    linter_tool: PythonLinterTool,
    enable_type_check: bool,
    enable_formatter_check: bool,
    formatter_tool: PythonFormatterTool,
}

impl Default for PythonAppPresetBuilder {
    fn default() -> Self {
        Self {
            python_version: None,
            enable_linter: false,
            linter_tool: PythonLinterTool::Flake8,
            enable_type_check: false,
            enable_formatter_check: false,
            formatter_tool: PythonFormatterTool::Black,
        }
    }
}

impl PythonAppPresetBuilder {
    /// Set the Python version
    pub fn python_version(mut self, version: impl Into<String>) -> Self {
        self.python_version = Some(version.into());
        self
    }

    /// Enable or disable linting
    pub fn linter(mut self, enable: bool) -> Self {
        self.enable_linter = enable;
        self
    }

    /// Set the linter tool
    pub fn linter_tool(mut self, tool: PythonLinterTool) -> Self {
        self.linter_tool = tool;
        self
    }

    /// Enable or disable type checking with mypy
    pub fn type_check(mut self, enable: bool) -> Self {
        self.enable_type_check = enable;
        self
    }

    /// Enable or disable formatter checking
    pub fn formatter_check(mut self, enable: bool) -> Self {
        self.enable_formatter_check = enable;
        self
    }

    /// Set the formatter tool
    pub fn formatter_tool(mut self, tool: PythonFormatterTool) -> Self {
        self.formatter_tool = tool;
        self
    }

    /// Build the PythonAppPreset
    pub fn build(self) -> PythonAppPreset {
        PythonAppPreset {
            python_version: self.python_version.unwrap_or_else(|| "3.11".to_string()),
            enable_linter: self.enable_linter,
            linter_tool: self.linter_tool,
            enable_type_check: self.enable_type_check,
            enable_formatter_check: self.enable_formatter_check,
            formatter_tool: self.formatter_tool,
        }
    }
}

impl ToGitHub for PythonAppPreset {
    fn to_github(&self) -> Result<GitHubWorkflow> {
        let mut jobs = BTreeMap::new();

        // Test job (always present)
        let test_steps = vec![
            GitHubStep {
                name: Some("Checkout code".to_string()),
                uses: Some("actions/checkout@v4".to_string()),
                run: None,
                with: None,
                env: None,
            },
            GitHubStep {
                name: Some("Setup Python".to_string()),
                uses: Some("actions/setup-python@v5".to_string()),
                run: None,
                with: Some(BTreeMap::from([(
                    "python-version".to_string(),
                    serde_yaml::Value::String(self.python_version.clone()),
                )])),
                env: None,
            },
            GitHubStep {
                name: Some("Install dependencies".to_string()),
                uses: None,
                run: Some("pip install -r requirements.txt".to_string()),
                with: None,
                env: None,
            },
            GitHubStep {
                name: Some("Run tests".to_string()),
                uses: None,
                run: Some("pytest".to_string()),
                with: None,
                env: None,
            },
        ];

        jobs.insert(
            "python/test".to_string(),
            GitHubJob {
                runs_on: "ubuntu-latest".to_string(),
                steps: test_steps,
                needs: None,
                timeout_minutes: Some(30),
                continue_on_error: None,
            },
        );

        // Lint job (optional)
        if self.enable_linter {
            let linter_name = self.linter_tool.name();
            let linter_cmd = self.linter_tool.check_command();

            jobs.insert(
                "python/lint".to_string(),
                GitHubJob {
                    runs_on: "ubuntu-latest".to_string(),
                    steps: vec![
                        GitHubStep {
                            name: Some("Checkout code".to_string()),
                            uses: Some("actions/checkout@v4".to_string()),
                            run: None,
                            with: None,
                            env: None,
                        },
                        GitHubStep {
                            name: Some("Setup Python".to_string()),
                            uses: Some("actions/setup-python@v5".to_string()),
                            run: None,
                            with: Some(BTreeMap::from([(
                                "python-version".to_string(),
                                serde_yaml::Value::String(self.python_version.clone()),
                            )])),
                            env: None,
                        },
                        GitHubStep {
                            name: Some(format!("Install {}", linter_name)),
                            uses: None,
                            run: Some(format!("pip install {}", linter_name)),
                            with: None,
                            env: None,
                        },
                        GitHubStep {
                            name: Some(format!("Run {}", linter_name)),
                            uses: None,
                            run: Some(linter_cmd.to_string()),
                            with: None,
                            env: None,
                        },
                    ],
                    needs: None,
                    timeout_minutes: Some(15),
                    continue_on_error: None,
                },
            );
        }

        // Type check job (optional)
        if self.enable_type_check {
            jobs.insert(
                "python/type-check".to_string(),
                GitHubJob {
                    runs_on: "ubuntu-latest".to_string(),
                    steps: vec![
                        GitHubStep {
                            name: Some("Checkout code".to_string()),
                            uses: Some("actions/checkout@v4".to_string()),
                            run: None,
                            with: None,
                            env: None,
                        },
                        GitHubStep {
                            name: Some("Setup Python".to_string()),
                            uses: Some("actions/setup-python@v5".to_string()),
                            run: None,
                            with: Some(BTreeMap::from([(
                                "python-version".to_string(),
                                serde_yaml::Value::String(self.python_version.clone()),
                            )])),
                            env: None,
                        },
                        GitHubStep {
                            name: Some("Install mypy".to_string()),
                            uses: None,
                            run: Some("pip install mypy".to_string()),
                            with: None,
                            env: None,
                        },
                        GitHubStep {
                            name: Some("Run mypy".to_string()),
                            uses: None,
                            run: Some("mypy .".to_string()),
                            with: None,
                            env: None,
                        },
                    ],
                    needs: None,
                    timeout_minutes: Some(15),
                    continue_on_error: None,
                },
            );
        }

        Ok(GitHubWorkflow {
            name: "CI".to_string(),
            on: GitHubTriggers::Detailed(BTreeMap::from([
                (
                    "push".to_string(),
                    GitHubTriggerConfig {
                        branches: Some(vec!["main".to_string(), "master".to_string()]),
                        tags: None,
                    },
                ),
                (
                    "pull_request".to_string(),
                    GitHubTriggerConfig {
                        branches: Some(vec!["main".to_string(), "master".to_string()]),
                        tags: None,
                    },
                ),
            ])),
            env: None,
            jobs,
        })
    }
}

impl ToGitLab for PythonAppPreset {
    fn to_gitlab(&self) -> Result<GitLabCI> {
        use crate::platforms::gitlab::models::*;
        use std::collections::BTreeMap;

        let mut jobs = BTreeMap::new();

        let mut script = vec!["pip install -r requirements.txt".to_string(), "pytest".to_string()];

        if self.enable_linter {
            script.insert(1, format!("pip install {}", self.linter_tool.name()));
            script.insert(2, self.linter_tool.check_command().to_string());
        }

        if self.enable_type_check {
            script.insert(1, "pip install mypy".to_string());
            script.insert(2, "mypy .".to_string());
        }

        if self.enable_formatter_check {
            script.insert(1, format!("pip install {}", self.formatter_tool.name()));
            script.insert(2, self.formatter_tool.check_command().to_string());
        }

        jobs.insert(
            "python/test".to_string(),
            GitLabJob {
                stage: "test".to_string(),
                image: Some(format!("python:{}", self.python_version)),
                script,
                before_script: None,
                after_script: None,
                needs: None,
                cache: None,
                artifacts: None,
                only: None,
                timeout: None,
            },
        );

        Ok(GitLabCI {
            stages: Some(vec!["test".to_string()]),
            variables: None,
            cache: None,
            jobs,
        })
    }
}

impl ToCircleCI for PythonAppPreset {
    fn to_circleci(&self) -> Result<CircleCIConfig> {
        use crate::platforms::circleci::models::*;
        use std::collections::BTreeMap;

        let mut steps = vec![
            CircleCIStep::Simple("checkout".to_string()),
            CircleCIStep::Command {
                run: CircleCIRun::Detailed {
                    name: "Install dependencies".to_string(),
                    command: "pip install -r requirements.txt".to_string(),
                },
            },
        ];

        if self.enable_linter {
            steps.push(CircleCIStep::Command {
                run: CircleCIRun::Detailed {
                    name: format!("Install {}", self.linter_tool.name()),
                    command: format!("pip install {}", self.linter_tool.name()),
                },
            });
            steps.push(CircleCIStep::Command {
                run: CircleCIRun::Detailed {
                    name: "Lint".to_string(),
                    command: self.linter_tool.check_command().to_string(),
                },
            });
        }

        if self.enable_type_check {
            steps.push(CircleCIStep::Command {
                run: CircleCIRun::Detailed {
                    name: "Install mypy".to_string(),
                    command: "pip install mypy".to_string(),
                },
            });
            steps.push(CircleCIStep::Command {
                run: CircleCIRun::Detailed {
                    name: "Type check".to_string(),
                    command: "mypy .".to_string(),
                },
            });
        }

        if self.enable_formatter_check {
            steps.push(CircleCIStep::Command {
                run: CircleCIRun::Detailed {
                    name: format!("Install {}", self.formatter_tool.name()),
                    command: format!("pip install {}", self.formatter_tool.name()),
                },
            });
            steps.push(CircleCIStep::Command {
                run: CircleCIRun::Detailed {
                    name: "Format check".to_string(),
                    command: self.formatter_tool.check_command().to_string(),
                },
            });
        }

        steps.push(CircleCIStep::Command {
            run: CircleCIRun::Detailed {
                name: "Run tests".to_string(),
                command: "pytest".to_string(),
            },
        });

        let mut jobs = BTreeMap::new();
        jobs.insert(
            "python/test".to_string(),
            CircleCIJob {
                docker: vec![CircleCIDocker {
                    image: format!("python:{}", self.python_version),
                }],
                steps,
                environment: None,
            },
        );

        Ok(CircleCIConfig {
            version: "2.1".to_string(),
            orbs: None,
            jobs,
            workflows: BTreeMap::from([(
                "main".to_string(),
                CircleCIWorkflow {
                    jobs: vec![CircleCIWorkflowJob::Simple("python/test".to_string())],
                },
            )]),
        })
    }
}

impl ToJenkins for PythonAppPreset {
    fn to_jenkins(&self) -> Result<JenkinsConfig> {
        use crate::platforms::jenkins::models::*;

        let mut test_steps = vec!["sh 'pip install -r requirements.txt'".to_string()];

        if self.enable_type_check {
            test_steps.push("sh 'mypy .'".to_string());
        }

        if self.enable_linter {
            test_steps.push("sh 'flake8 .'".to_string());
        }

        test_steps.push("sh 'pytest'".to_string());

        Ok(JenkinsConfig {
            agent: "any".to_string(),
            environment: vec![],
            stages: vec![JenkinsStage {
                name: "Test".to_string(),
                steps: test_steps,
            }],
        })
    }
}

impl Detectable for PythonAppPreset {
    fn matches_github(&self, workflow: &GitHubWorkflow) -> bool {
        let has_python_setup = workflow.jobs.values().any(|job| {
            job.steps.iter().any(|step| {
                step.uses
                    .as_ref()
                    .map(|u| u.contains("setup-python"))
                    .unwrap_or(false)
            })
        });

        let has_pytest = workflow.jobs.values().any(|job| {
            job.steps.iter().any(|step| {
                step.run
                    .as_ref()
                    .map(|r| r.contains("pytest"))
                    .unwrap_or(false)
            })
        });

        has_python_setup && has_pytest
    }

    fn matches_gitlab(&self, _config: &GitLabCI) -> bool {
        false
    }

    fn matches_circleci(&self, _config: &CircleCIConfig) -> bool {
        false
    }

    fn matches_jenkins(&self, _config: &JenkinsConfig) -> bool {
        false
    }
}

impl PresetInfo for PythonAppPreset {
    fn name(&self) -> &str {
        "python-app"
    }

    fn description(&self) -> &str {
        "CI pipeline for Python applications with pytest, linting, and type checking"
    }
}
