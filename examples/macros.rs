use sarge::prelude::*;

sarge! {
    /// This is a basic macros example.
    Args,

    /// Show this help message.
    'h' help: bool,

    /// The name to greet.
    'n' @NAME name: String,

    /// The number of times to greet.
    /// Defaults to 1.
    #ok times: u32,
}

fn main() {
    let args = match Args::parse() {
        Ok((a, _)) => a,
        Err(e) => {
            eprintln!("failed to parse arguments: {e}");
            Args::print_help();
            std::process::exit(1);
        }
    };

    if args.help {
        Args::print_help();
        return;
    }

    for _ in 0..args.times.unwrap_or(1) {
        println!("Hello, {}!", args.name);
    }
}
