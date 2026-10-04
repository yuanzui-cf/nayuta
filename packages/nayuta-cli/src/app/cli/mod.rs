use crate::{
    core::{
        context::{self, Project},
        create,
        post::{self, NewPost},
        schema::apply_sets,
    },
    i18n::I18n,
    infra::{
        error::{Result, user},
        git::{github, template},
        logging,
    },
};
use clap::{Arg, ArgAction, ArgMatches, ColorChoice, Command};
use serde_json::Value;
use std::{
    ffi::OsString,
    io::IsTerminal,
    path::{Path, PathBuf},
};

pub struct Ui {
    pub cwd: PathBuf,
    pub project: Option<Project>,
    pub i18n: I18n,
    pub color: bool,
    pub mouse: bool,
}

pub enum Launch {
    Exit(u8),
    Tui(Box<Ui>),
}

fn value(name: &'static str, help: String) -> Arg {
    Arg::new(name).long(name).help(help).num_args(1)
}
fn flag(name: &'static str, help: String) -> Arg {
    Arg::new(name)
        .long(name)
        .help(help)
        .action(ArgAction::SetTrue)
}

pub fn command(i18n: &I18n, ready: bool) -> Command {
    let sets = || value("set", i18n.t("arg_set")).action(ArgAction::Append);
    let create = Command::new("create")
        .about(i18n.t("create"))
        .arg(
            Arg::new("directory")
                .required(true)
                .help(i18n.t("directory")),
        )
        .arg(value("ref", i18n.t("ref")))
        .arg(value("config", i18n.t("arg_config")))
        .arg(sets())
        .arg(flag("prerelease", i18n.t("arg_prerelease")))
        .arg(flag("offline", i18n.t("arg_offline")));
    let mut new = Command::new("new")
        .about(i18n.t("new_post"))
        .arg(Arg::new("slug").required(true).help(i18n.t("slug")))
        .arg(
            value("format", i18n.t("format"))
                .value_parser(["md", "mdx"])
                .default_value("md"),
        )
        .arg(value("frontmatter", i18n.t("arg_config")))
        .arg(sets())
        .arg(
            value("date-prefix", i18n.t("date_prefix"))
                .value_parser(["true", "false"])
                .default_value("true"),
        )
        .arg(value("tag", i18n.t("tags")).action(ArgAction::Append));
    for (arg, key) in [
        ("title", "title"),
        ("publish-date", "publishDate"),
        ("description", "description"),
        ("cover", "cover"),
    ] {
        new = new.arg(value(arg, i18n.t(key)));
    }
    new = new.arg(value("draft", i18n.t("draft")).value_parser(["true", "false"]));
    let mut cmd = Command::new("nayuta")
        .version(env!("CARGO_PKG_VERSION"))
        .about(i18n.t("welcome"))
        .subcommand_help_heading(i18n.t("commands"))
        .next_help_heading(i18n.t("options"))
        .arg(value("cwd", i18n.t("arg_cwd")).global(true))
        .arg(value("locale", i18n.t("arg_locale")).global(true))
        .arg(
            value("color", i18n.t("arg_color"))
                .value_parser(["auto", "always", "never"])
                .global(true),
        )
        .arg(flag("no-mouse", i18n.t("arg_mouse")).global(true))
        .arg(flag("tui", i18n.t("arg_tui")).global(true))
        .arg(flag("verbose", i18n.t("arg_verbose")).global(true))
        .arg(value("log-file", i18n.t("arg_log")).global(true))
        .after_help(i18n.t(if ready { "help_ready" } else { "help_normal" }));
    cmd = if ready {
        cmd.subcommand(
            Command::new("post")
                .about(i18n.t("new_post"))
                .subcommand_required(true)
                .subcommand(new),
        )
    } else {
        cmd.subcommand(create)
    };
    cmd.disable_help_flag(true)
        .disable_version_flag(true)
        .propagate_version(true)
        .arg(
            Arg::new("help")
                .long("help")
                .short('h')
                .help(i18n.t("help"))
                .action(ArgAction::Help)
                .global(true),
        )
        .arg(
            Arg::new("version")
                .long("version")
                .short('V')
                .help(i18n.t("arg_version"))
                .action(ArgAction::Version)
                .global(true),
        )
}

// Bootstrap reads only globals needed to construct the strict, mode-aware parser.
pub fn bootstrap(args: &[OsString], key: &str) -> Option<OsString> {
    let mut result = None;
    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        if arg == "--" {
            break;
        }
        if arg == std::ffi::OsStr::new(&format!("--{key}")) {
            result = iter.next().cloned();
        } else if let Some(value) = arg
            .to_str()
            .and_then(|s| s.strip_prefix(&format!("--{key}=")))
        {
            result = Some(value.into());
        }
    }
    result
}

pub fn launch(args: Vec<OsString>) -> Launch {
    let detected = bootstrap(&args, "locale");
    let i18n = match I18n::detect(detected.as_deref().and_then(|s| s.to_str())) {
        Ok(i18n) => i18n,
        Err(error) => {
            eprintln!("{}", I18n::default().error(&error));
            return Launch::Exit(2);
        }
    };
    // Version is independent of the current project, locale, network and schema health.
    if args.len() == 2 && matches!(args[1].to_str(), Some("--version" | "-V")) {
        println!("nayuta {}", env!("CARGO_PKG_VERSION"));
        return Launch::Exit(0);
    }
    let cwd = bootstrap(&args, "cwd")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let project = match context::discover(&cwd) {
        Ok(project) => project,
        Err(error) => {
            eprintln!("{}", i18n.error(&error));
            return Launch::Exit(1);
        }
    };
    let mut cmd = command(&i18n, project.is_some());
    let color_arg = bootstrap(&args, "color");
    let color = match color_arg.as_deref().and_then(|v| v.to_str()) {
        Some("always") => true,
        Some("never") => false,
        _ => std::env::var_os("NO_COLOR").is_none() && std::io::stdout().is_terminal(),
    };
    cmd = cmd.color(if color {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    });
    let matches = match cmd.clone().try_get_matches_from(args) {
        Ok(matches) => matches,
        Err(error) => {
            let code = if matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) {
                0
            } else {
                2
            };
            if code == 0 {
                print!("{error}");
            } else {
                eprintln!("{}: {error}\n{}", i18n.t("error"), cmd.render_help());
            }
            return Launch::Exit(code);
        }
    };
    let tui = matches.subcommand().is_none();
    if matches.get_flag("tui") && !tui {
        eprintln!("{}: --tui\n{}", i18n.t("mode"), cmd.render_help());
        return Launch::Exit(2);
    }
    if tui && (!std::io::stdin().is_terminal() || !std::io::stdout().is_terminal()) {
        eprintln!("{}\n{}", i18n.t("tty"), cmd.render_help());
        return Launch::Exit(2);
    }
    let log = matches.get_one::<String>("log-file").map(|s| cwd.join(s));
    if let Err(error) = logging::init(matches.get_flag("verbose"), tui, log.as_deref()) {
        eprintln!("{}", i18n.error(&error));
        return Launch::Exit(1);
    }
    if tui {
        return Launch::Tui(Box::new(Ui {
            cwd,
            project,
            i18n,
            color,
            mouse: !matches.get_flag("no-mouse"),
        }));
    }
    let result = execute(&cwd, project.as_ref(), &matches, &i18n);
    match result {
        Ok(()) => Launch::Exit(0),
        Err(error) => {
            tracing::debug!(error=%error, "command failed");
            eprintln!("{}", i18n.error(&error));
            Launch::Exit(1)
        }
    }
}

pub fn merge(target: &mut Value, source: Value) {
    if let (Some(target), Value::Object(source)) = (target.as_object_mut(), &source) {
        for (key, value) in source {
            merge(
                target.entry(key.clone()).or_insert(Value::Null),
                value.clone(),
            );
        }
    } else {
        *target = source;
    }
}

fn json_file(cwd: &Path, matches: &ArgMatches, arg: &str, values: &mut Value) -> Result<()> {
    if let Some(file) = matches.get_one::<String>(arg) {
        let value: Value = serde_json::from_slice(&std::fs::read(cwd.join(file))?)?;
        if !value.is_object() {
            return Err(user("invalid_json", file));
        }
        merge(values, value);
    }
    Ok(())
}
fn sets(matches: &ArgMatches) -> Vec<String> {
    matches
        .get_many::<String>("set")
        .map(|v| v.cloned().collect())
        .unwrap_or_default()
}

fn execute(cwd: &Path, project: Option<&Project>, matches: &ArgMatches, i18n: &I18n) -> Result<()> {
    match matches.subcommand() {
        Some(("create", args)) => {
            let offline = args.get_flag("offline");
            let reference = if let Some(reference) = args.get_one::<String>("ref") {
                reference.clone()
            } else {
                github::releases(offline)?
                    .into_iter()
                    .find(|r| args.get_flag("prerelease") || !r.prerelease)
                    .ok_or_else(|| user("not_found", "Release"))?
                    .tag_name
            };
            let template = template::fetch(&reference, offline)?;
            let mut values = create::defaults(&template, cwd)?;
            json_file(cwd, args, "config", &mut values)?;
            let schema = crate::infra::config::schema::load(&template.root)?;
            apply_sets(&schema.config, &mut values, &sets(args))?;
            let project = create::create(
                &template,
                Path::new(args.get_one::<String>("directory").unwrap()),
                &mut values,
                cwd,
            )?;
            println!(
                "{}: {}\n{}",
                i18n.t("created"),
                project.root.display(),
                i18n.t("next_steps")
            );
        }
        Some(("post", args)) => {
            let project = project.ok_or_else(|| user("mode", "post"))?;
            let args = args.subcommand_matches("new").unwrap();
            let mut values = post::defaults(project);
            json_file(cwd, args, "frontmatter", &mut values)?;
            for (arg, key) in [
                ("title", "title"),
                ("publish-date", "publishDate"),
                ("description", "description"),
                ("cover", "cover"),
                ("draft", "draft"),
            ] {
                if let Some(value) = args.get_one::<String>(arg) {
                    if project.schema.post.field(key).is_none() {
                        return Err(user("unsupported", key));
                    }
                    values[key] = if key == "draft" {
                        Value::Bool(value == "true")
                    } else {
                        Value::String(value.clone())
                    };
                }
            }
            if let Some(tags) = args.get_many::<String>("tag") {
                if project.schema.post.field("tags").is_none() {
                    return Err(user("unsupported", "tags"));
                }
                values["tags"] = Value::Array(tags.map(|s| Value::String(s.clone())).collect());
            }
            apply_sets(&project.schema.post, &mut values, &sets(args))?;
            let mut options = NewPost {
                slug: args.get_one::<String>("slug").unwrap().clone(),
                format: args.get_one::<String>("format").unwrap().clone(),
                date_prefix: args.get_one::<String>("date-prefix").unwrap() == "true",
                values,
            };
            let path = post::create(project, &mut options)?;
            println!("{}: {}", i18n.t("post_created"), path.display());
        }
        _ => return Err(user("mode", "command")),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mode_help_and_invalid_arguments_are_strict() {
        let i18n = I18n::default();
        assert!(
            command(&i18n, false)
                .try_get_matches_from(["nayuta", "post", "new", "x"])
                .is_err()
        );
        assert!(
            command(&i18n, true)
                .try_get_matches_from(["nayuta", "create", "x"])
                .is_err()
        );
        assert!(
            command(&i18n, true)
                .try_get_matches_from(["nayuta", "--bogus"])
                .is_err()
        );
        assert!(
            command(&i18n, true)
                .try_get_matches_from(["nayuta", "post", "new", "hello", "--draft", "maybe"])
                .is_err()
        );
        assert!(
            command(&i18n, false)
                .try_get_matches_from(["nayuta", "--locale", "zh-HK", "create", "blog"])
                .is_ok()
        );
        assert!(
            bootstrap(
                &["nayuta".into(), "--".into(), "--cwd".into(), "x".into()],
                "cwd"
            )
            .is_none()
        );
    }
}
