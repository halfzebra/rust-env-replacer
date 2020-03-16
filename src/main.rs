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
    let token_regex = Regex::new(r"\{\{([a-zA-Z0-9_]+)\}\}").unwrap();
    let glob_pattern = &format!(
        "{}{}",
        env::current_dir().unwrap().to_str().unwrap(),
        &opts.glob
    );
    let env_vars_to_use: HashMap<String, String> = env::vars()
        .filter(|(name, _)| name.starts_with(&opts.prefix))
        .collect();
    let env_var_names = env_vars_to_use.keys().cloned().collect::<HashSet<String>>();

    if opts.debug {
        println!("Glob Pattern: {:?}\n", glob_pattern);
        println!("env_var_names: {:?}\n", env_var_names);
        println!("{:?}\n", opts);
    }

    for entry in glob(glob_pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                if opts.debug {
                    println!("File: {:#?}", path);
                }
                let file_content = fs::read_to_string(path.clone().into_os_string()).unwrap();
                let token_match_iter = token_regex.captures_iter(&file_content);
                let unique_tokens_from_file: HashSet<String> = token_match_iter
                    .map(|capture| String::from(capture.get(1).unwrap().as_str()))
                    .collect();

                let tokens_from_file_without_env_var: HashSet<&String> =
                    unique_tokens_from_file.difference(&env_var_names).collect();

                println!("tokens_from_file_without_env_var: {:?}", tokens_from_file_without_env_var);
                println!("unique_tokens_from_file {:?}", unique_tokens_from_file);

                // for (name, value) in env::vars() {
                //     let replace_name_pattern = format!("{{{{{}}}}}", name);
                //     file = file.replace(&replace_name_pattern, &value);
                // }

                if opts.debug {
                    println!("File content: {:#?}", file_content);
                }

                // let mut f = File::create(path).unwrap();
                // f.write_all(&file.into_bytes()).unwrap();
                // f.sync_data().unwrap();
            }
            Err(e) => {
                println!("{:?}", e);
                process::exit(1);
            }
        }
    }
}
