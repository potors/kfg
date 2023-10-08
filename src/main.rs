use kfg::{lexer, parser};
use std::{env, fs, io, process};

fn main() -> io::Result<()> {
    env_logger::init();

    let args = env::args().collect::<Vec<_>>();

    if args.len() < 2 {
        log::error!("Syntax: <file>");
        process::exit(1);
    }

    let file = &args[1];

    log::info!("Reading '{file}'");
    let content = fs::read(file)?;

    log::info!("Tokenizing '{file}'");
    let tokens = lexer::tokenize(&content);

    if log::log_enabled!(log::Level::Debug) {
        for token in &tokens {
            log::debug!("{:?}", token.kind);
        }
    }

    log::info!("Lexing '{file}'");
    let tokens = lexer::filter(&tokens);

    if log::log_enabled!(log::Level::Debug) {
        for token in &tokens {
            log::debug!("{:?}", token.kind);
        }
    }

    log::info!("Parsing '{file}'");
    let ast = parser::parse(&tokens);

    match ast {
        Ok(ast) => log::info!("\n{ast}"),
        Err(err) => log::debug!("{err:?}"),
    }

    Ok(())
}
