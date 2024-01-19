mod clidef;
mod filters;
mod logger;
mod procdata;
mod profile;
mod rootfs;
mod scanner;
mod shcall;
use crate::profile::Profile;
use clap::{error::ErrorKind, ArgMatches, Command};
use colored::Colorize;
use std::{
    env,
    path::{Path, PathBuf},
    process,
};

static VERSION: &str = "0.1";
static LOGGER: logger::STDOUTLogger = logger::STDOUTLogger;

/// Get flag from the params and return inverted if requested
fn f(params: &ArgMatches, name: &str) -> bool {
    let p = *params.get_one::<bool>(name).unwrap();
    if *params.get_one::<bool>("invert-filters").unwrap() {
        return !p;
    }
    p
}

/// Checks if flag is set at all
fn is_f(params: &ArgMatches, name: &str) -> bool {
    *params.get_one::<bool>(name).unwrap()
}

/// Construct profile either from the CLI
/// or get it from the YAML and configure it.
fn get_profile(mut cli: Command, params: &ArgMatches) -> Profile {
    let exe = params.get_one::<String>("exe");
    let profile_path = params.get_one::<String>("profile");

    if exe.is_none() && profile_path.is_none() {
        cli.print_help().unwrap();
        process::exit(exitcode::OK);
    }

    let mut profile: Profile = Profile::default();
    if let Some(exe) = exe {
        log::info!("Getting data for the target {exe}");
        profile
            .add_target(exe.to_string())
            .set_manpages(f(params, "f_man"))
            .set_dir(f(params, "f_dir"))
            .set_doc(f(params, "f_doc"))
            .set_i18n(f(params, "f_i18n"))
            .set_l10n(f(params, "f_l10n"))
            .set_log(f(params, "f_log"))
            .set_arch(f(params, "f_arc"))
            .set_img(f(params, "f_pic"));
    } else if let Some(profile_path) = profile_path {
        log::info!("Getting profile at {profile_path}");
        match Profile::new(Path::new(profile_path)) {
            Ok(p) => {
                profile = p;

                // Override profile
                if is_f(params, "f_man") {
                    profile.set_manpages(f(params, "f_man"));
                }
                if is_f(params, "f_dir") {
                    profile.set_manpages(f(params, "f_dir"));
                }
                if is_f(params, "f_doc") {
                    profile.set_manpages(f(params, "f_doc"));
                }
                if is_f(params, "f_i18n") {
                    profile.set_manpages(f(params, "f_i18n"));
                }
                if is_f(params, "f_l10n") {
                    profile.set_manpages(f(params, "f_l10n"));
                }
                if is_f(params, "f_log") {
                    profile.set_manpages(f(params, "f_log"));
                }
                if is_f(params, "f_pic") {
                    profile.set_img(f(params, "f_pic"));
                }
                if is_f(params, "f_arc") {
                    profile.set_arch(f(params, "f_arc"));
                }
            }
            Err(err) => {
                log::error!("{}", err);
                process::exit(exitcode::OSERR);
            }
        }
    }

    profile
}

/// Main
fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();
    let mut cli = clidef::cli(VERSION);

    if args.len() == 1 {
        return {
            cli.print_help().unwrap();
            Ok(())
        };
    }

    let params = cli.to_owned().get_matches();

    // Since --help is disabled on purpose in CLI definition, it is checked manually.
    if *params.get_one::<bool>("help").unwrap() {
        cli.print_help().unwrap();
        return Ok(());
    }

    if *params.get_one::<bool>("delta-only").unwrap() {
        if sys_info::os_type().unwrap_or_default().to_lowercase() != "linux" {
            return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Unsupported OS. Only Linux for now..."));
        }

        return Ok(());
    }

    if *params.get_one::<bool>("version").unwrap() {
        println!("Version {}", VERSION);
        return Ok(());
    }

    // Setup logger
    if let Err(err) = log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(if params.get_flag("debug") { log::LevelFilter::Trace } else { log::LevelFilter::Info }))
    {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()));
    }

    // Start data processor
    let rpth = PathBuf::from(params.get_one::<String>("root").unwrap());
    if !rpth.exists() {
        log::error!("Mountpoint \"{}\" does not exist or is not accessible", rpth.to_str().unwrap().bright_yellow());
        process::exit(exitcode::IOERR);
    }

    log::info!("Launching scanner and data processor");

    if let Err(err) = procdata::TintProcessor::new(rpth)
        .set_profile(get_profile(cli, &params))
        .set_dry_run(params.get_flag("dry-run"))
        .set_autodeps(params.get_one::<String>("autodeps").unwrap().to_string())
        .start()
    {
        log::error!("{}", err);
        process::exit(exitcode::IOERR);
    }

    if params.get_flag("dry-run") {
        log::warn!("This was a dry-run. Changes were not applied.");
    } else {
        log::info!("Finished. Hopefully it even works :-)");
    }

    Ok(())
}
