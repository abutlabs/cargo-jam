use crate::cli::args::NewArgs;
use crate::error::{CargoJamError, Result};
use crate::project::generator::ProjectGenerator;
use crate::prompt::interactive::PromptRunner;
use crate::template::bundled::BundledTemplates;
use crate::template::config::TemplateConfig;
use crate::template::git::GitTemplateSource;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::path::PathBuf;

// Enum to hold template source so it stays alive (the temp directory)
#[allow(dead_code)]
enum TemplateSource {
    Bundled(BundledTemplates),
    Git(GitTemplateSource),
}

pub fn execute(args: NewArgs) -> Result<()> {
    let spinner = create_spinner("Preparing template...");

    // Determine template source and keep it alive
    let (_template_source, template_dir) = if let Some(git_url) = &args.git {
        spinner.set_message("Cloning template repository...");
        let mut source = GitTemplateSource::new(git_url.clone())
            .branch(args.branch.clone())
            .subpath(args.path.clone());
        let dir = source.fetch()?;
        (TemplateSource::Git(source), dir)
    } else {
        spinner.set_message("Loading bundled template...");
        let mut templates = BundledTemplates::new();
        let dir = templates.extract(&args.template)?;
        (TemplateSource::Bundled(templates), dir)
    };

    let config = TemplateConfig::load_from_dir(&template_dir)?;

    spinner.finish_and_clear();

    // Collect template variables
    let mut variables = collect_predefined_variables(&args)?;

    // Get project name
    let project_name = if let Some(name) = args.name.clone() {
        validate_project_name(&name)?;
        name
    } else if args.defaults {
        return Err(CargoJamError::InvalidProjectName {
            name: String::new(),
            reason: "Project name is required when using --defaults".to_string(),
        });
    } else {
        let runner = PromptRunner::new();
        runner.prompt_string("Project name", None, Some(r"^[a-z][a-z0-9_-]*$"))?
    };

    variables.insert("project_name".to_string(), project_name.clone());
    variables.insert("crate_name".to_string(), project_name.replace('-', "_"));

    // Run interactive prompts for remaining variables
    if !args.defaults {
        let runner = PromptRunner::new();
        let prompted_vars = runner.collect_variables(&config, &variables)?;
        variables.extend(prompted_vars);
    } else {
        // Apply defaults from config
        for (key, placeholder) in &config.placeholders {
            if !variables.contains_key(key) {
                if let Some(default) = placeholder.default_value() {
                    variables.insert(key.clone(), default);
                }
            }
        }
    }

    // Determine output directory
    let output_dir = args.output.unwrap_or_else(|| PathBuf::from(&project_name));

    // Check if output directory exists
    if output_dir.exists() {
        return Err(CargoJamError::ProjectExists(
            output_dir.display().to_string(),
        ));
    }

    // Generate project
    let spinner = create_spinner("Generating project...");
    let generator = ProjectGenerator::new(template_dir, output_dir.clone(), config);
    generator.generate(&variables)?;
    spinner.finish_and_clear();

    // Initialize git repository
    if !args.no_git {
        let spinner = create_spinner("Initializing git repository...");
        crate::project::git_init::init_git_repo(&output_dir)?;
        spinner.finish_and_clear();
    }

    // Print success message
    println!(
        "\n{} Created JAM service '{}' at {}",
        style("✓").green().bold(),
        style(&project_name).cyan(),
        style(output_dir.display()).yellow()
    );
    println!("\nNext steps:");
    println!("  {} {}", style("cd").cyan(), project_name);
    println!("  {} jam build", style("cargo").cyan());

    Ok(())
}

fn create_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));
    spinner
}

fn collect_predefined_variables(args: &NewArgs) -> Result<HashMap<String, String>> {
    let mut variables = HashMap::new();

    // Parse --define flags
    for define in &args.define {
        if let Some((key, value)) = define.split_once('=') {
            variables.insert(key.to_string(), value.to_string());
        }
    }

    // Load from values file if provided
    if let Some(values_file) = &args.values_file {
        let content = std::fs::read_to_string(values_file)?;
        let values: HashMap<String, String> = toml::from_str(&content)?;
        variables.extend(values);
    }

    Ok(variables)
}

fn validate_project_name(name: &str) -> Result<()> {
    let re = regex::Regex::new(r"^[a-z][a-z0-9_-]*$").unwrap();
    if !re.is_match(name) {
        return Err(CargoJamError::InvalidProjectName {
            name: name.to_string(),
            reason: "Must start with lowercase letter, contain only lowercase letters, numbers, underscores, and hyphens".to_string(),
        });
    }
    Ok(())
}
