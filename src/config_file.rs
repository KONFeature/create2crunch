use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

use crate::RunConfig;

// Config file structure
#[derive(Deserialize)]
pub struct ConfigFile {
    pub bin_folder: String,
    pub factory_address: String,
    pub calling_address: String,
    pub gpu_device: Option<u8>,
    pub targets: Vec<Target>,
}

// Config smart contract target structure
#[derive(Deserialize, Clone)]
pub struct Target {
    pub name: String,
    pub placeholder_name: Option<String>,
    #[serde(default)]
    pub stop_thresholds: Option<StopThresholds>,
}

// Config smart contract target stop thresholds (which requirements do we need to stop the crunching and move on)
#[derive(Deserialize, Clone)]
pub struct StopThresholds {
    pub leading_zeroes: Option<u8>,
    pub total_zeroes: Option<u8>,
}

impl ConfigFile {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let config_content = fs::read_to_string(path)?;
        let config: ConfigFile = toml::from_str(&config_content)?;

        // Check if bin folder exists
        if !Path::new(&config.bin_folder).exists() {
            return Err(format!("Bin folder '{}' does not exist", &config.bin_folder).into());
        }

        // Iterate over each target
        for target in &config.targets {
            // Construct the path to the bin file
            let bin_file_path = Path::new(&config.bin_folder).join(&target.name);

            // Check if the bin file exists
            if !bin_file_path.exists() {
                return Err(
                    format!("Bin file '{}' does not exist", bin_file_path.display()).into(),
                );
            }
        }

        // Stop points validation
        config.validate_stop_points()?;

        // Placeholder validation
        config.validate_placeholders()?;

        Ok(config)
    }

    pub fn validate_placeholders(&self) -> Result<(), String> {
        let placeholder_pattern = regex::Regex::new(r"\$\{(\w+)\}").unwrap();

        for target in &self.targets {
            // Construct the path to the bin file
            let bin_file_path = Path::new(&self.bin_folder).join(&target.name);

            let bin_content = std::fs::read_to_string(&bin_file_path)
                .map_err(|e| format!("Failed to read bin file {}", e))?;

            // Ensure the placeholder referenced in the bin file exists in the target configuration
            for caps in placeholder_pattern.captures_iter(&bin_content) {
                let placeholder = &caps[1];
                // Ensure the placeholder exists in some target configuration
                if !self
                    .targets
                    .iter()
                    .any(|target| target.placeholder_name.as_deref() == Some(placeholder))
                {
                    return Err(format!(
                        "Missing placeholder '{}' in the target configuration",
                        placeholder
                    ));
                }

                // Ensure that the placeholder isn't the one of the target itself (otherwise it would create a circular dependency)
                if target.placeholder_name.as_deref() == Some(placeholder) {
                    return Err(format!(
                        "Circular dependency detected for placeholder '{}'",
                        placeholder
                    ));
                }
            }
        }
        Ok(())
    }

    /// Ensure every target has at least one stop point (leading zeroes or total zeroes)
    fn validate_stop_points(&self) -> Result<(), String> {
        for target in &self.targets {
            if let Some(stop_thresholds) = &target.stop_thresholds {
                if stop_thresholds.leading_zeroes.is_none()
                    && stop_thresholds.total_zeroes.is_none()
                {
                    return Err(format!(
                        "Target '{}' does not have any stop points defined",
                        target.name
                    ));
                }
            } else {
                return Err(format!(
                    "Target '{}' does not have any stop points defined",
                    target.name
                ));
            }
        }
        Ok(())
    }
}
