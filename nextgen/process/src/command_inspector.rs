use crate::command::Command;
use moon_common::color;
use once_cell::sync::OnceCell;
use rustc_hash::FxHashMap;
use shell_words::join;
use std::env;
use std::fmt::{self, Display};
use std::path::{PathBuf, MAIN_SEPARATOR};
use tracing::{debug, enabled};

pub struct CommandLine {
    pub command: Vec<String>,
    pub input: Vec<String>,
    pub main_command: String,
}

impl Display for CommandLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let command = join(&self.command);

        write!(f, "{}", &command)?;

        if !self.input.is_empty() {
            let debug_input = env::var("MOON_DEBUG_PROCESS_INPUT").is_ok();
            let input = join(&self.input);

            if !command.ends_with('-') {
                write!(f, " -")?;
            }

            write!(
                f,
                " {}",
                if input.len() > 200 && !debug_input {
                    "(truncated)".into()
                } else {
                    input.replace('\n', " ")
                }
            )?;
        }

        Ok(())
    }
}

pub struct CommandInspector<'cmd> {
    command: &'cmd Command,
    line_cache: OnceCell<CommandLine>,
}

impl<'cmd> CommandInspector<'cmd> {
    pub fn new(command: &'cmd Command) -> Self {
        Self {
            command,
            line_cache: OnceCell::new(),
        }
    }

    pub fn get_command_line(&self) -> &CommandLine {
        self.line_cache.get_or_init(|| self.create_command_line())
    }

    pub fn get_prefix(&self) -> String {
        self.command.prefix.clone().unwrap_or_default()
    }

    pub fn should_error_nonzero(&self) -> bool {
        self.command.error_on_nonzero
    }

    pub fn should_pass_stdin(&self) -> bool {
        !self.command.input.is_empty() || self.should_pass_args_stdin()
    }

    pub fn should_pass_args_stdin(&self) -> bool {
        self.command
            .shell
            .as_ref()
            .map(|s| s.pass_args_stdin)
            .unwrap_or(false)
    }

    pub fn format_command(&self, line: &str) -> String {
        let workspace_root = env::var("MOON_WORKSPACE_ROOT")
            .map(|root| PathBuf::from(root))
            .unwrap_or_else(|_| env::current_dir().unwrap());

        let working_dir = self.command.cwd.as_ref().unwrap_or(&workspace_root);

        let target_dir = if working_dir == &workspace_root {
            "workspace".into()
        } else {
            format!(
                ".{}{}",
                MAIN_SEPARATOR,
                working_dir
                    .strip_prefix(&workspace_root)
                    .unwrap()
                    .to_string_lossy(),
            )
        };

        format!(
            "{} {}",
            color::muted_light(line),
            color::muted(format!("(in {target_dir})"))
        )
    }

    pub fn log_command(&self) {
        let command_line = self.get_command_line();

        if self.command.print_command {
            println!("{}", self.format_command(&command_line.main_command));
        }

        // Avoid all this overhead if we're not logging
        if !enabled!(tracing::Level::DEBUG) {
            return;
        }

        let debug_env = env::var("MOON_DEBUG_PROCESS_ENV").is_ok();

        let env_vars_field = self
            .command
            .env
            .iter()
            .filter(|(key, _)| {
                if debug_env {
                    true
                } else {
                    let key = key.to_str().unwrap_or_default();
                    key.starts_with("MOON_") || key.starts_with("PROTO_")
                }
            })
            .collect::<FxHashMap<_, _>>();

        let working_dir_field = self
            .command
            .cwd
            .as_ref()
            .map(|cwd| cwd.display().to_string());

        debug!(
            env_vars = ?env_vars_field,
            working_dir = working_dir_field,
            "Running command {}",
            color::shell(command_line.to_string())
        );
    }

    fn create_command_line(&self) -> CommandLine {
        let mut command_line: Vec<String> = vec![];
        let mut input_line: Vec<String> = vec![];
        let mut main_line: Vec<String> = vec![];

        let push_to_line = |line: &mut Vec<String>| {
            line.push(self.command.bin.to_string_lossy().to_string());

            for arg in &self.command.args {
                line.push(arg.to_string_lossy().to_string());
            }
        };

        // Extract the main command, without shell, for other purposes!
        push_to_line(&mut main_line);

        // If wrapped in a shell, the shell binary and arguments
        // must be placed at the start of the line.
        if let Some(shell) = &self.command.shell {
            command_line.push(shell.bin.clone());
            command_line.extend(shell.args.clone());

            // If the main command should be passed via stdin,
            // then append the input line instead of the command line.
            if shell.pass_args_stdin {
                push_to_line(&mut input_line);

                // Otherwise append as regular arguments. They typically
                // appear after a "-" argument (should come from shell).
            } else {
                push_to_line(&mut command_line);
            }

            // Otherwise we have a normal command and arguments.
        } else {
            push_to_line(&mut command_line);

            // That also may have input.
            if !self.command.input.is_empty() {
                for input in &self.command.input {
                    input_line.push(input.to_string_lossy().to_string());
                }
            }
        }

        CommandLine {
            command: command_line,
            input: input_line,
            main_command: join(main_line),
        }
    }
}
