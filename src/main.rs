use log::debug;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::{fs, io, process};
use structopt::StructOpt;

#[derive(Debug)]
pub enum MakeProjectError {
    ArgumentError(String),
    Io(io::Error),
    Process(String, i32),
}

impl std::convert::From<io::Error> for MakeProjectError {
    fn from(e: io::Error) -> MakeProjectError {
        MakeProjectError::Io(e)
    }
}

impl std::fmt::Display for MakeProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            MakeProjectError::ArgumentError(msg) => write!(f, "Error: {}", msg),
            MakeProjectError::Io(e) => e.fmt(f),
            MakeProjectError::Process(msg, _) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for MakeProjectError {}

#[derive(Debug, PartialEq, Eq)]
enum Language {
    Python,
    Rust,
}

impl FromStr for Language {
    type Err = MakeProjectError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "python" => Ok(Language::Python),
            "rust" => Ok(Language::Rust),
            o => Err(MakeProjectError::ArgumentError(format!(
                "parsing model from given command: `{}`",
                o
            ))),
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "mkproject", about = "Create projects with templates easily")]
struct Opt {
    #[structopt(short = "l", long = "language")]
    language: Language,

    #[structopt(parse(from_os_str))]
    path: PathBuf,
}

fn run_command(cmd: &mut process::Command) -> Result<(), MakeProjectError> {
    let op = cmd.output()?;
    check_status(op)
}

fn check_status(op: process::Output) -> Result<(), MakeProjectError> {
    let status = op.status;
    if !status.success() {
        let code = status.code().expect("process should have an exit code");

        return Err(MakeProjectError::Process(
            format!("running `cargo new` command, exit code: {}", code),
            code,
        ));
    }
    Ok(())
}

fn create_readme(path: &PathBuf) -> Result<(), MakeProjectError> {
    debug!("Creating initial readme");
    let readme_path = path.join("README.md");
    let project_name = compute_project_name(path);
    let mut file = fs::File::create(readme_path)?;

    let project_name = project_name
        .into_string()
        .expect("path contains invalid UTF-8 data");
    writeln!(file, "# {}", project_name)?;
    Ok(())
}

fn compute_project_name(project_path: &PathBuf) -> std::ffi::OsString {
    let path = project_path.as_path();
    let stub = path.file_name().expect("no final path component given");

    stub.to_os_string()
}

fn create_python_project(path: &PathBuf) -> Result<(), MakeProjectError> {
    debug!("Creating dir: {:?}", path);

    fs::create_dir(&path)?;

    let venv_path = path.join("venv");

    debug!("Creating virtual environment");
    run_command(
        process::Command::new("python3")
            .arg("-m")
            .arg("venv")
            .arg(&venv_path),
    )?;

    debug!("Installing ipython");
    run_command(
        process::Command::new(venv_path.join("bin").join("pip"))
            .arg("install")
            .arg("ipython"),
    )?;

    create_readme(path)?;
    Ok(())
}

// TODO: add optional language-specific arguments
fn create_rust_project(path: &PathBuf) -> Result<(), MakeProjectError> {
    debug!("Running cargo new");
    run_command(
        process::Command::new("cargo")
            .arg("new")
            .arg(path.to_str().unwrap()),
    )?;

    create_readme(path)?;
    Ok(())
}

fn main() -> Result<(), MakeProjectError> {
    env_logger::init();

    let opts = Opt::from_args();

    let result = match opts.language {
        Language::Python => create_python_project(&opts.path),
        Language::Rust => create_rust_project(&opts.path),
    };

    match result {
        Err(MakeProjectError::Process(msg, code)) => {
            eprintln!("Error: {}", msg);
            process::exit(code);
        }
        _ => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn parsing_python() {
        let s = "python";
        assert_eq!(Language::from_str(s).unwrap(), Language::Python);
    }

    #[test]
    fn parsing_rust() {
        let s = "rust";
        assert_eq!(Language::from_str(s).unwrap(), Language::Rust);
    }

    #[test]
    fn parsing_something_else() {
        let s = "other";
        assert!(Language::from_str(s).is_err());
    }

    #[test]
    fn creating_a_rust_project() {
        let temp_dir = TempDir::new("mkproject-rust-project").unwrap();
        let path = temp_dir.path().join("myproject");

        create_rust_project(&path).expect("creating Rust project");

        assert!(path.join("Cargo.toml").is_file());
        assert!(path.join("src").is_dir());
        assert!(path.join("src").join("main.rs").is_file());
        assert!(path.join("README.md").is_file());

        let readme_contents = fs::read_to_string(path.join("README.md")).unwrap();
        assert_eq!(readme_contents, "# myproject\n");
    }

    #[test]
    fn creating_a_python_project() {
        let temp_dir = TempDir::new("mkproject-rust-project").unwrap();
        let path = temp_dir.path().join("myproject");

        create_python_project(&path).expect("creating a Python project");

        assert!(path.join("venv").is_dir());
        assert!(path.join("README.md").is_file());

        let readme_contents = fs::read_to_string(path.join("README.md")).unwrap();
        assert_eq!(readme_contents, "# myproject\n");

        // Check that ipython is installed
        assert!(path.join("venv").join("bin").join("ipython").is_file());
    }
}
