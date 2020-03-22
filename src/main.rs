use clap::Clap;
use glob::glob;
use regex::Regex;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::process;

#[derive(Clap, Debug)]
#[clap(version = "1.0")]
struct Opts {
    #[clap(short, long)]
    debug: bool,
    /// File glob to process
    #[clap(name = "GLOB", parse(try_from_str = parse_glob))]
    glob: String,

    /// Env var name prefix to use
    #[clap(short, long, default_value("APP_"))]
    prefix: String,

    /// Continue substitution if some of the tokens in files don't have a corresponding env var
    #[clap(long)]
    ignore_missing: bool,
}

#[derive(Debug)]
struct EnvReplacerError(String);

fn parse_glob(glob_str: &str) -> Result<String, &'static str> {
    match glob::Pattern::new(&glob_str) {
        Ok(_) => Ok(String::from(glob_str)),
        Err(_) => Err("Failed to parse glob"),
    }
}

fn write_file(
    path: std::path::PathBuf,
    content: String,
) -> std::result::Result<(), std::io::Error> {
    let mut f = File::create(path)?;
    f.write_all(&content.into_bytes())?;
    f.sync_data()?;
    Ok(())
}

fn tokens_from_string(file_content: &String) -> HashSet<String> {
    let token_regex = Regex::new(r"\{\{([a-zA-Z0-9_]+)\}\}").unwrap();
    let token_match_iter = token_regex.captures_iter(&file_content);
    token_match_iter
        .map(|capture| String::from(capture.get(1).unwrap().as_str()))
        .collect()
}

fn replace_tokens(token_map: &HashMap<String, String>, file_content: &mut String) {
    let unique_tokens_from_file = tokens_from_string(file_content);
    let env_var_names: HashSet<String> = token_map.keys().cloned().collect();
    let tokens_present_in_env_and_file = unique_tokens_from_file.intersection(&env_var_names);
    for name in tokens_present_in_env_and_file {
        let replace_name_pattern = format!("{{{{{}}}}}", name);
        *file_content = file_content.replace(&replace_name_pattern, token_map.get(name).unwrap());
    }
}

fn unknown_tokens<'a>(
    tokens_from_file: &'a HashSet<String>,
    env_var_names: &'a HashSet<String>,
) -> HashSet<&'a String> {
    tokens_from_file.difference(&env_var_names).collect()
}

fn process_files(opts: &Opts) -> Result<(), EnvReplacerError> {
    let glob_pattern: String = format!(
        "{}{}",
        env::current_dir()
            .map_err(|_| EnvReplacerError(
                "Failed to retrieve the current working directory".into()
            ))?
            .to_str()
            .ok_or(EnvReplacerError(
                "Failed to convert the current working directory path to a string".into()
            ))?,
        &opts.glob
    );

    let env_vars_to_use: HashMap<String, String> = env::vars()
        .filter(|(name, _)| name.starts_with(&opts.prefix))
        .collect();
    let env_var_token_names: HashSet<String> = env_vars_to_use.keys().cloned().collect();
    let paths_iter =
        glob(&glob_pattern).map_err(|_| EnvReplacerError("Failed to read glob pattern".into()))?;

    if env_vars_to_use.len() == 0 {
        return Err(EnvReplacerError(format!(
            "Could not find any env vars starting with \"{}\"",
            &opts.prefix
        )));
    }

    for entry in paths_iter {
        println!("{:?}", entry);
        match entry {
            Ok(path) => {
                let file_path_str = path.clone().into_os_string().into_string();
                let mut file_content = fs::read_to_string(&path).unwrap();
                let tokens_from_file = tokens_from_string(&file_content);
                let tokens_from_file_without_env_var =
                    unknown_tokens(&tokens_from_file, &env_var_token_names);

                if tokens_from_file_without_env_var.len() > 0 {
                    return Err(EnvReplacerError(format!("{}", &file_path_str.unwrap())));
                }

                replace_tokens(&env_vars_to_use, &mut file_content);
                write_file(path, file_content).map_err(|_| {
                    EnvReplacerError(format!(
                        "Failed to update file: {}",
                        &file_path_str.unwrap()
                    ))
                })?;
            }
            Err(_e) => {
                if opts.ignore_missing == false {
                    return Err(EnvReplacerError(
                        "Failed to unwrap one of the glob entries".into(),
                    ));
                }
            }
        };
    }

    Ok(())
}

fn handle_processing_result(res: Result<(), EnvReplacerError>) {
    match res {
        Ok(_) => {
            println!("Substitution successful");
            process::exit(0);
        }
        Err(e) => {
            eprintln!("Substitution failed with error:\n  {}", e.0);
            process::exit(65);
        }
    }
}

fn main() {
    handle_processing_result(process_files(&Opts::parse()));
}
