use cci_macros::{Preset, PresetEnum};
use crate::error::Result;
use crate::platforms::circleci::models::CircleCIConfig;
use crate::platforms::github::models::{
    GitHubJob, GitHubStep, GitHubTriggerConfig, GitHubTriggers, GitHubWorkflow,
};
use crate::platforms::gitlab::models::GitLabCI;
use crate::platforms::jenkins::models::JenkinsConfig;
use crate::traits::{Detectable, PresetInfo, ToCircleCI, ToGitea, ToGitHub, ToGitLab, ToJenkins};
use std::collections::BTreeMap;

/// Linter tool options for Python
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, PresetEnum)]
#[preset_enum(default = "Flake8")]
#[serde(rename_all = "lowercase")]
pub enum PythonLinter {
    #[preset_variant(id = "flake8", display = "Flake8")]
    Flake8,
    #[preset_variant(id = "ruff", display = "Ruff")]
    Ruff,
}

impl PythonLinter {
    pub fn name(&self) -> &'static str {
        match self {
            PythonLinter::Flake8 => "flake8",
            PythonLinter::Ruff => "ruff",
        }
    }

    pub fn check_command(&self) -> &'static str {
        match self {
            PythonLinter::Flake8 => "flake8 .",
            PythonLinter::Ruff => "ruff check .",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            PythonLinter::Flake8 => PythonLinter::Ruff,
            PythonLinter::Ruff => PythonLinter::Flake8,
        }
    }
}

/// Formatter tool options for Python
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, PresetEnum)]
#[preset_enum(default = "Black")]
#[serde(rename_all = "lowercase")]
pub enum PythonFormatter {
    #[preset_variant(id = "black", display = "Black")]
    Black,
    #[preset_variant(id = "ruff", display = "Ruff")]
    Ruff,
}

impl PythonFormatter {
    pub fn name(&self) -> &'static str {
        match self {
            PythonFormatter::Black => "black",
            PythonFormatter::Ruff => "ruff",
        }
    }

    pub fn check_command(&self) -> &'static str {
        match self {
            PythonFormatter::Black => "black --check .",
            PythonFormatter::Ruff => "ruff format --check .",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            PythonFormatter::Black => PythonFormatter::Ruff,
            PythonFormatter::Ruff => PythonFormatter::Black,
        }
    }
}

/// Preset for Python application projects
#[derive(Debug, Clone, Preset)]
#[preset(
    id = "python-app",
    name = "Python App",
    description = "CI pipeline for Python applications with pytest, linting, and type checking",
    matches = "PythonApp | PythonLibrary"
)]
pub struct PythonAppPreset {
    #[preset_field(
        ron_field = "version",
        default = "\"3.11\".to_string()",
        hidden = true
    )]
    python_version: String,

    #[preset_field(
        ron_field = "linter",
        feature = "linting",
        feature_display = "Linting",
        feature_description = "Code quality checks with configurable tools",
        display = "Linter",
        description = "Choose linter tool (None, Flake8, or Ruff)",
        default = "None"
    )]
    linter: Option<PythonLinter>,

    #[preset_field(
        id = "type_check",
        feature = "testing",
        feature_display = "Testing",
        feature_description = "Test execution and type checking",
        display = "Type Checking",
        description = "Enable mypy static type checking",
        default = "true"
    )]
    enable_type_check: bool,

    #[preset_field(
        ron_field = "formatter",
        feature = "formatting",
        feature_display = "Formatting",
        feature_description = "Code formatting checks",
        display = "Formatter",
        description = "Choose formatter tool (None, Black, or Ruff)",
        default = "None"
    )]
    formatter: Option<PythonFormatter>,
}

impl PythonAppPreset {
    /// Constant default instance for registry initialization
    pub const DEFAULT: Self = Self {
        python_version: String::new(),
        linter: None,
        enable_type_check: false,
        formatter: None,
    };
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
        if let Some(linter) = &self.linter {
            let linter_name = linter.name();
            let linter_cmd = linter.check_command();

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

impl ToGitea for PythonAppPreset {
    fn to_gitea(&self) -> Result<crate::platforms::gitea::models::GiteaWorkflow> {
        // Gitea Actions uses the same workflow format as GitHub Actions
        self.to_github()
    }
}

impl ToGitLab for PythonAppPreset {
    fn to_gitlab(&self) -> Result<GitLabCI> {
        use crate::platforms::gitlab::models::*;
        use std::collections::BTreeMap;

        let mut jobs = BTreeMap::new();

        let mut script = vec!["pip install -r requirements.txt".to_string(), "pytest".to_string()];

        if let Some(linter) = &self.linter {
            script.insert(1, format!("pip install {}", linter.name()));
            script.insert(2, linter.check_command().to_string());
        }

        if self.enable_type_check {
            script.insert(1, "pip install mypy".to_string());
            script.insert(2, "mypy .".to_string());
        }

        if let Some(formatter) = &self.formatter {
            script.insert(1, format!("pip install {}", formatter.name()));
            script.insert(2, formatter.check_command().to_string());
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

        if let Some(linter) = &self.linter {
            steps.push(CircleCIStep::Command {
                run: CircleCIRun::Detailed {
                    name: format!("Install {}", linter.name()),
                    command: format!("pip install {}", linter.name()),
                },
            });
            steps.push(CircleCIStep::Command {
                run: CircleCIRun::Detailed {
                    name: "Lint".to_string(),
                    command: linter.check_command().to_string(),
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

        if let Some(formatter) = &self.formatter {
            steps.push(CircleCIStep::Command {
                run: CircleCIRun::Detailed {
                    name: format!("Install {}", formatter.name()),
                    command: format!("pip install {}", formatter.name()),
                },
            });
            steps.push(CircleCIStep::Command {
                run: CircleCIRun::Detailed {
                    name: "Format check".to_string(),
                    command: formatter.check_command().to_string(),
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

        if let Some(linter) = &self.linter {
            test_steps.push(format!("sh '{}'", linter.check_command()));
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

    fn matches_gitea(&self, workflow: &crate::platforms::gitea::models::GiteaWorkflow) -> bool {
        // Gitea Actions uses the same workflow format as GitHub Actions
        self.matches_github(workflow)
    }

    fn matches_gitlab(&self, _config: &GitLabCI) -> bool{
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

// EditorPreset implementation is now auto-generated by #[derive(Preset)]
