use create2crunch::CliArgsConfig;
use create2crunch::ConfigFile;
use std::collections::HashMap;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!(
            "Usage: {} [config_file] or provide necessary arguments",
            args[0]
        );
        process::exit(1);
    }

    // Check if the first argument is a config file
    let config_arg = &args[1];
    let config_path = std::path::Path::new(config_arg);

    // Check if the config file exists
    if config_path.exists() {
        run_with_config_file(&args);
    } else {
        run_with_args(&args);
    }
}

// Run create2crunch with args
fn run_with_args(args: &[String]) {
    // Parse the args
    let config = CliArgsConfig::new(args).unwrap_or_else(|err| {
        eprintln!("Failed parsing arguments: {err}");
        process::exit(1);
    });

    if config.gpu_device == 255 {
        if let Err(e) = create2crunch::cpu(config.to_run_config()) {
            eprintln!("CPU application error: {e}");
            process::exit(1);
        }
    } else if let Err(e) = create2crunch::gpu(config.to_run_config(), config.gpu_device) {
        eprintln!("GPU application error: {e}");
        process::exit(1);
    }
}

// Run create2crunch with config file
fn run_with_config_file(args: &[String]) {
    let config_arg = &args[1];

    // Load the configuration file
    let config_file = ConfigFile::new(config_arg).unwrap_or_else(|err| {
        eprintln!("Failed to load configuration file: {}", err);
        process::exit(1);
    });

    // Process the configuration
    if let Err(e) = process_config(config_file) {
        eprintln!("Configuration processing error: {e}");
        process::exit(1);
    }
}

fn process_config(config_file: ConfigFile) -> Result<(), Box<dyn std::error::Error>> {
    // Map to store computed addresses by placeholder name
    let mut computed_addresses: HashMap<String, String> = HashMap::new();

    // Targets that are yet to be processed
    let mut remaining_targets = config_file.targets.clone();

    // Loop until all targets are processed or no progress is made
    while !remaining_targets.is_empty() {
        let initial_len = remaining_targets.len();
        let mut processed_indices = Vec::new();

        for (i, target) in remaining_targets.iter().enumerate() {
            // Check if all placeholders in this target can be filled
            let can_process = if let Some(placeholder_name) = &target.placeholder_name {
                // Check if the placeholder value is available
                computed_addresses.contains_key(placeholder_name)
            } else {
                true // No placeholder, can process
            };

            // if can_process {
            //     // Construct RunConfig
            //     let run_config = create2crunch::RunConfig::from_config_file(
            //         &config_file,
            //         target,
            //         &computed_addresses,
            //     )?;

            //     // Decide whether to use CPU or GPU
            //     let result_address = if run_config.gpu_device == 255 {
            //         create2crunch::cpu_run(run_config)?
            //     } else {
            //         create2crunch::gpu_run(run_config)?
            //     };

            //     // If this target defines a placeholder, store the computed address
            //     if let Some(placeholder_name) = &target.placeholder_name {
            //         computed_addresses.insert(placeholder_name.clone(), result_address);
            //     }

            //     // Mark this target as processed
            //     processed_indices.push(i);
            // }
        }

        // Remove processed targets from the list
        for &i in processed_indices.iter().rev() {
            remaining_targets.remove(i);
        }

        // If no targets were processed in this iteration, there is a circular dependency or missing placeholder
        if remaining_targets.len() == initial_len {
            return Err("Unable to process all targets".into());
        }
    }

    Ok(())
}
