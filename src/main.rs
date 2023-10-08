use log::{debug, info, error, log_enabled, Level};
use std::{env, process, fs, io};
use kfg::{lexer, parser};

fn main() -> io::Result<()> {
    env_logger::init();

    let args = env::args().collect::<Vec<_>>();

    if args.len() < 2 {
        error!("Syntax: <file>");
        process::exit(1);
    }

    let file = &args[1];

    info!("Reading '{file}'");
    let content = fs::read(file)?;

    info!("Tokenizing '{file}'");
    let tokens = lexer::tokenize(&content);

    if log_enabled!(Level::Debug) {
        for token in &tokens {
            debug!("{:?}", token.kind);
        }
    }

    info!("Lexing '{file}'");
    let tokens = lexer::filter(&tokens);

    if log_enabled!(Level::Debug) {
        for token in &tokens {
            debug!("{:?}", token.kind);
        }
    }

    info!("Parsing '{file}'");
    let ast = parser::parse(&tokens).unwrap();

    info!("\n{ast}");

    Ok(())
}
