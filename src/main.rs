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

fn tokens_from_file(file_content: &String) -> HashSet<String> {
    let token_regex = Regex::new(r"\{\{([a-zA-Z0-9_]+)\}\}").unwrap();
    let token_match_iter = token_regex.captures_iter(&file_content);
    token_match_iter
        .map(|capture| String::from(capture.get(1).unwrap().as_str()))
        .collect()
}

fn replace_tokens(token_map: &HashMap<String, String>, file_content: &mut String) {
    let unique_tokens_from_file = tokens_from_file(file_content);
    let env_var_names: HashSet<String> = token_map.keys().cloned().collect();
    let tokens_present_in_env_and_file = unique_tokens_from_file.intersection(&env_var_names);
    for name in tokens_present_in_env_and_file {
        let replace_name_pattern = format!("{{{{{}}}}}", name);
        *file_content = file_content.replace(&replace_name_pattern, token_map.get(name).unwrap());
    }
}

#[derive(Clap, Debug)]
#[clap(version = "1.0")]
struct Opts {
    #[clap(short, long)]
    debug: bool,
    /// File glob to process
    #[clap(name = "GLOB", parse(try_from_str = parse_glob))]
    glob: String,

    #[clap(short, long, default_value("APP_"))]
    prefix: String,
}

fn main() {
    let opts: Opts = Opts::parse();
    let glob_pattern = &format!(
        "{}{}",
        env::current_dir().unwrap().to_str().unwrap(),
        &opts.glob
    );
    let env_vars_to_use: HashMap<String, String> = env::vars()
        .filter(|(name, _)| name.starts_with(&opts.prefix))
        .collect();

    if opts.debug {
        println!("Glob Pattern: {:?}\n", glob_pattern);
        println!("{:?}\n", opts);
    }

    for entry in glob(glob_pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                if opts.debug {
                    println!("File: {:#?}", path);
                }
                let mut file_content = fs::read_to_string(path.clone().into_os_string()).unwrap();
                let unique_tokens_from_file = tokens_from_file(&file_content);
                let env_var_names: HashSet<String> = env_vars_to_use.keys().cloned().collect();
                let tokens_from_file_without_env_var: HashSet<&String> =
                    unique_tokens_from_file.difference(&env_var_names).collect();

                println!(
                    "tokens_from_file_without_env_var: {:?}",
                    tokens_from_file_without_env_var
                );

                if opts.debug {
                    println!("File content: {:#?}", file_content);
                }

                replace_tokens(&env_vars_to_use, &mut file_content);

                write_file(path, file_content).map_err(|_| "Error writing to file \n: {}");
            }
            Err(e) => {
                println!("{:?}", e);
                process::exit(1);
            }
        }
    }
}
