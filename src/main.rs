extern crate clap;

use clap::App;
use indicatif::{ProgressBar, ProgressStyle};

mod configuration;
mod doctor;
mod repository;
mod subcommand;

fn main() {
    let init = App::new("init").about("Initialize a new configuration file");
    let clone = App::new("clone").about("Clone repositories to disk");
    let doctor = App::new("doctor").about("Checks that the required tooling is availabile");

    let matches = App::new("teamtailor-cli")
        .version("v0.1-beta")
        .about("Helps out with your development environment")
        .subcommand(init)
        .subcommand(clone)
        .subcommand(doctor)
        .get_matches();

    match matches.subcommand() {
        ("init", _) => run_init_command(),
        ("clone", _) => run_clone_command(),
        ("doctor", _) => run_doctor_command(),
        _ => std::process::exit(1),
    }
}

fn run_init_command() -> () {
    match subcommand::init::call() {
        Ok(configuration) => {
            println!(
                "success: created configuration file ({})",
                configuration.filepath()
            );
        }
        Err(subcommand::init::Error::CreateConfigurationError(error)) => match error {
            configuration::CreateError::ConfigurationAlreadyExists => {
                eprintln!(
                    "fatal: configuration file already exists: {}",
                    configuration::path().to_str().unwrap()
                );
                std::process::exit(1);
            }
            configuration::CreateError::CouldNotSerializeConfiguration(serde_error) => {
                eprintln!(
                    "fatal: could not create the configuration file ({})",
                    serde_error
                );
                std::process::exit(1);
            }
            configuration::CreateError::CouldNotCreateFile(io_error) => {
                eprintln!(
                    "fatal: could not create the configuration file ({})",
                    io_error
                );
                std::process::exit(1);
            }
            configuration::CreateError::CouldNotCreateConfigurationDirectory(io_error) => {
                eprintln!(
                    "fatal: could not create the configuration directory ({})",
                    io_error
                );
                std::process::exit(1);
            }
        },
    }
}

fn run_clone_command() -> () {
    let configuration = configuration::Configuration::load_configuration();

    match configuration {
        Ok(configuration) => {
            let remote_repositories = repository::RemoteRepository::all();

            for repo in remote_repositories.iter() {
                let pb = ProgressBar::new_spinner();
                pb.enable_steady_tick(120);
                pb.set_style(
                    ProgressStyle::default_spinner()
                        .tick_strings(&[
                            "▹▹▹▹▹",
                            "▸▹▹▹▹",
                            "▹▸▹▹▹",
                            "▹▹▸▹▹",
                            "▹▹▹▸▹",
                            "▹▹▹▹▸",
                            "▪▪▪▪▪",
                        ])
                        .template("{spinner:.blue} {msg}"),
                );
                let message = format!("Cloning repository '{}'", repo.name());
                pb.set_message(&message);

                let result = repo.clone_repostiory(&configuration);

                match result {
                    Ok(_local_repository) => {
                        let finish_message = format!("[{}] done", repo.name());
                        pb.finish_with_message(&finish_message);
                    }
                    Err(repository::CloneError::FailedToClone(_repo, git_error)) => {
                        let finish_message = format!(
                            "[{}] failed to clone ({})",
                            repo.name(),
                            git_error.message()
                        );
                        pb.finish_with_message(&finish_message);
                    }
                    Err(repository::CloneError::AlreadyCloned(_)) => {
                        let finish_message = format!("[{}] already cloned", repo.name());
                        pb.finish_with_message(&finish_message);
                    }
                }
            }

            std::process::exit(0);
        }
        Err(_) => {
            eprintln!("fatal: failed to load the configuration file");
            std::process::exit(1);
        }
    }
}

fn run_doctor_command() {
    for executable in doctor::check_executables() {
        print!("checking for {}... ", executable.name());

        match executable.path() {
            Some(path) => {
                println!("found! {}", path);
            }
            None => {
                println!("missing");
                std::process::exit(1);
            }
        }
    }

    std::process::exit(0);
}
