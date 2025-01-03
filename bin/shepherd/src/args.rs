use anyhow::Context;
use lexopt::prelude::*;
use std::{env, path::PathBuf, process};

use shepherd_lib::shepherd::Shepherd;

#[derive(Default)]
pub struct Args<'de> {
    pub n_epochs: usize,
    pub shepherds: Vec<Shepherd<'de>>,
    pub database_file: Option<PathBuf>,
}

fn usage() {
    println!(
        "usage: {} [-h|--help] [-n|--n-epochs=EPOCHS] [-d|--database=DATABASE_FILE] [shepherds...]",
        env::args().next().as_deref().unwrap_or("shepherd")
    );
}

pub fn parse_args<'de>() -> anyhow::Result<Args<'de>> {
    let mut args = Args::default();
    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Short('h') | Long("help") => {
                usage();
                #[allow(clippy::exit)]
                process::exit(0);
            }
            Short('n') | Long("n-epochs") => {
                args.n_epochs = parser
                    .value()
                    .context("No argument given to -n or --n-epochs")?
                    .parse()
                    .context("Invalid argument to -n or --n-epochs")?;
            }
            Short('d') | Long("database") => {
                args.database_file = Some(
                    parser
                        .value()
                        .context("No argument given to -d or --database")?
                        .into(),
                );
            }
            Value(shepherd) => {
                args.shepherds.push(Shepherd::new(shepherd).context(
                    "Unable to build a shepherd from a given path",
                )?);
            }
            a => {
                println!("unknown argument: {:?}", a);
                usage();
                #[allow(clippy::exit)]
                process::exit(1);
            }
        }
    }

    Ok(args)
}
