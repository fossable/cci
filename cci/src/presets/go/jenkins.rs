use crate::error::Result;
use crate::platforms::jenkins::models::JenkinsConfig;
use crate::traits::ToJenkins;

use super::GoAppPreset;

impl ToJenkins for GoAppPreset {
    fn to_jenkins(&self) -> Result<JenkinsConfig> {
        use crate::platforms::jenkins::models::*;

        let mut test_steps = Vec::new();

        if self.enable_security_scan {
            test_steps.push("sh 'gosec ./...'".to_string());
        }

        if self.enable_linter {
            test_steps.push("sh 'golangci-lint run'".to_string());
        }

        test_steps.push("sh 'go test -v ./...'".to_string());

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
