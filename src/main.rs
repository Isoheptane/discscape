use clap::Parser;

mod cli;

fn main() {

    if let Err(err) = cli::run(cli::Cli::parse()) {
        eprintln!("Error: {err:#}");
        std::process::exit(1);
    }

}