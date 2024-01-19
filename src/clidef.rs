use clap::builder::styling;
use clap::{Arg, ArgAction, Command};
use colored::Colorize;

/// Define CLI arguments and styling
pub fn cli(version: &'static str) -> Command {
    let styles = styling::Styles::styled()
        .header(styling::AnsiColor::White.on_default() | styling::Effects::BOLD)
        .usage(styling::AnsiColor::White.on_default() | styling::Effects::BOLD)
        .literal(styling::AnsiColor::BrightCyan.on_default())
        .placeholder(styling::AnsiColor::Cyan.on_default());

    Command::new("mezzotint")
        .version(version)
        .about(format!("{}{} - {}", "mezzo".bold().underline(), "tint", "is a tool to turn your container into an App Bundle"))
        .override_usage(format!("{} {} {}", "mezzotint".bright_cyan(), "[OPTIONS]".cyan(), "[FILTERS]".cyan()))
        // Config
        .arg(
            Arg::new("exe")
                .short('x')
                .long("exe")
                .conflicts_with("profile")
                .help("Specify path to an executable which needs to be preserved")
        )
        .arg(
            Arg::new("profile")
                .short('p')
                .long("profile")
                .conflicts_with("exe")
                .help("Profile, describing whole setup")
        )
        .arg(
            Arg::new("packages")
                .short('k')
                .long("pkgs")
                .aliases(["packages", "packags", "packs"])
                .help("Comma-separated list of packages to account")
        )
        .arg(
            Arg::new("invert-filters")
                .short('i')
                .long("invert")
                .action(clap::ArgAction::SetTrue)
                .help("Invert filters behaviour")
        )
        .arg(
            Arg::new("dry-run")
                .short('t')
                .long("dry-run")
                .action(clap::ArgAction::SetTrue)
                .help("Do not remove anything, only display what will be removed")
        )
        .arg(
            Arg::new("autodeps")
                .short('a')
                .long("autodeps")
                .default_value("none")
                .value_name("mode")
                .value_parser(["free", "clean", "tight", "none"])
                .help(format!("Auto-add package dependencies.\n{}", " NOTE: This can increase the size, but might not always be useful\n".yellow()))
        )
        .arg(
            Arg::new("delta-only")
                .short('l')
                .long("delta-only")
                .action(clap::ArgAction::SetTrue)
                .help(format!("Remove all identical files, those are found on the current host system.\n{}",
                " NOTE: Installation of such app bundle container might be erroneous\n on other systems! Make sure this container is installed on \n the same distribution and its version!".yellow()))
        )
        .arg(
            Arg::new("root")
                .short('r')
                .long("root")
                .required_unless_present_any(["help", "version"])
                .help("Root filesystem, e.g. mountpoint of an image")
        )

        // Filters
        .next_help_heading("Filters")
        .arg(
            Arg::new("f_l10n").long("l10n").action(clap::ArgAction::SetTrue).help("Leave localisation data")
        )
        .arg(
            Arg::new("f_i18n").long("i18n").action(clap::ArgAction::SetTrue).help("Leave internationalisation data")
        )
        .arg(
            Arg::new("f_doc").long("doc").action(clap::ArgAction::SetTrue).help("Leave documents, texts, licences etc")
        )
        .arg(
            Arg::new("f_man").long("man").action(clap::ArgAction::SetTrue).help("Leave manpages")
        )
        .arg(
            Arg::new("f_dir").long("dirs").action(clap::ArgAction::SetTrue).help("Leave empty directories (except required)")
        )
        .arg(
            Arg::new("f_log").long("logs").action(clap::ArgAction::SetTrue).help("Leave any kind of logs")
        )
        .arg(
            Arg::new("f_pic").long("pic").action(clap::ArgAction::SetTrue).help("Leave any graphics (pictures)")
        )
        .arg(
            Arg::new("f_arc").long("arc").action(clap::ArgAction::SetTrue).help("Leave any kind of archives/tarballs")
        )

        // Other
        .next_help_heading("Other")
        .arg(
            Arg::new("debug")
                .short('d')
                .long("debug")
                .action(ArgAction::SetTrue)
                .help("Set debug mode for more verbose output."),
        )
        .arg(
            Arg::new("help")
                .short('h')
                .long("help")
                .action(ArgAction::SetTrue)
                .help("Display help"),
        )
        .arg(
            Arg::new("version")
                .short('v')
                .long("version")
                .action(ArgAction::SetTrue)
                .help("Get current version."),
        )
        .disable_help_flag(true) // Otherwise it is displayed in a wrong position
        .disable_version_flag(true)
        .disable_colored_help(false)
        .styles(styles)
        .after_help("NOTE: This tool is in very early development.
      If it doesn't work for you, please fill a bug report here:
      https://github.com/isbm/mezzotint/issues\n".bright_yellow().to_string())
}
