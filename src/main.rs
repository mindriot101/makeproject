use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug)]
pub struct MakeProjectError(String);

impl std::fmt::Display for MakeProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "Error: {}", self.0)
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
            o => Err(MakeProjectError(format!(
                "Error parsing model from given command: {}",
                o
            ))),
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "makeproject", about = "Create projects with templates easily")]
struct Opt {
    #[structopt(short = "l", long = "language")]
    language: Language,
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
