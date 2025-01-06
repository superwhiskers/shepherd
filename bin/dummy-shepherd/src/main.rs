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
use rand::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    io::{self, prelude::*},
};

use shepherd_lib::{
    feed::Feed,
    shepherd::{ShepherdEvent, SimulationEvent},
    simulation::Epoch,
};

fn main() -> anyhow::Result<()> {
    let mut items = HashSet::new();
    let mut sheep_seen = HashMap::new();
    let mut stdout = io::stdout();

    for event in serde_json::Deserializer::from_reader(io::stdin())
        .into_iter::<SimulationEvent>()
    {
        let event = event
            .context("Unable to retrieve an event from standard input")?;
        match event {
            SimulationEvent::BeginEpoch {
                data:
                    Epoch {
                        items: new_items, ..
                    },
                ..
            } => items.extend(new_items.into_iter().map(|(id, _)| id)),
            SimulationEvent::FeedRequest { sheep } => {
                let seen =
                    sheep_seen.entry(sheep).or_insert_with(HashSet::new);
                let chosen = items
                    .difference(seen)
                    .copied()
                    .choose_multiple(&mut rand::thread_rng(), 10);
                seen.extend(chosen.iter().copied());
                serde_json::to_writer(
                    &mut stdout,
                    &ShepherdEvent::Feed(Feed(chosen)),
                )
                .context("Unable to write an event to stdout")?;
                stdout.flush().context("Unable to flush stdout")?;
            }
            _ => (),
        }
    }

    Ok(())
}
