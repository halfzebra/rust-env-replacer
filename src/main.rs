use std::env;
use std::fs;
use glob::glob;
use std::fs::File;
use std::io::Write;
use clap::Clap;

#[derive(Clap, Debug)]
#[clap(version = "1.0")]
struct Opts {
    #[clap(short, long)]
    debug: bool,
    /// Files to process
    #[clap(name = "FILE")]
    glob: String,
}

fn main() {
    let opts: Opts = Opts::parse();

    let glob_pattern = &format!("{}{}", env::current_dir().unwrap().to_str().unwrap(), "/**/*.js");

    println!("{:?}", glob_pattern);
    println!("{:?}", opts);

    for entry in glob(glob_pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let mut file = fs::read_to_string(path.clone().into_os_string()).unwrap();

                for (name, value) in env::vars() {
                    let replace_name_pattern = format!("{{{{{}}}}}", name);
                    
                    if file.contains(&replace_name_pattern) {
                        println!("{}", replace_name_pattern);
                        file = file.replace(&replace_name_pattern, &value);
                    }
                }

                println!("{:?}", file);

                let mut f = File::create(path).unwrap();
                f.write_all(&file.into_bytes()).unwrap();
                f.sync_data().unwrap();
            },
            Err(e) => println!("{:?}", e),
        }
    }
}
