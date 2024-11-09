#![allow(clippy::cognitive_complexity)]
#![warn(clippy::cargo_common_metadata)]
#![warn(clippy::dbg_macro)]
#![warn(clippy::explicit_deref_methods)]
#![warn(clippy::filetype_is_file)]
#![warn(clippy::imprecise_flops)]
#![warn(clippy::large_stack_arrays)]
#![warn(clippy::todo)]
#![warn(clippy::unimplemented)]
#![deny(clippy::await_holding_lock)]
#![deny(clippy::cast_lossless)]
#![deny(clippy::clone_on_ref_ptr)]
#![deny(clippy::doc_markdown)]
#![deny(clippy::empty_enum)]
#![deny(clippy::enum_glob_use)]
#![deny(clippy::exit)]
#![deny(clippy::explicit_into_iter_loop)]
#![deny(clippy::explicit_iter_loop)]
#![deny(clippy::fallible_impl_from)]
#![deny(clippy::inefficient_to_string)]
#![deny(clippy::large_digit_groups)]
#![deny(clippy::wildcard_dependencies)]
#![deny(clippy::wildcard_imports)]
#![deny(clippy::unused_self)]
#![deny(clippy::single_match_else)]
#![deny(clippy::option_option)]
#![deny(clippy::mut_mut)]

use anyhow::Context;
use std::io;
use tracing::info;

use crate::{
    args::Args,
    simulation::{Settings, Simulation},
};

mod args;
mod feed;
mod graph;
mod ids;
mod sheep;
mod shepherd;
mod simulation;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_writer(io::stderr).init();

    let Args {
        n_epochs,
        shepherds,
    } = args::parse_args().context("Unable to parse arguments")?;
    let mut simulation = Simulation::new(
        &mut rand::thread_rng(),
        shepherds,
        Settings {
            new_epoch_hook: Some(Box::new(|i, _| {
                info!("starting epoch {:?}", i)
            })),
            ..Default::default()
        },
    )
    .context("Unable to initialize the simulation")?;

    for i in 0..n_epochs {
        simulation
            .simulate_epoch(&mut rand::thread_rng())
            .context("Unable to simulate an epoch")?;
    }

    Ok(())
}
