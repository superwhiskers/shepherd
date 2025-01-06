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
use duckdb::{params, Connection};
use rand::prelude::*;
use std::{
    collections::HashSet,
    io::{self, prelude::*},
};

use shepherd_lib::{
    feed::Feed,
    ids::GraphId,
    shepherd::{ShepherdEvent, SimulationEvent},
    simulation::Epoch,
};

fn main() -> anyhow::Result<()> {
    let mut stdout = io::stdout();

    let duckdb = Connection::open_in_memory()
        .context("Unable to open a duckdb database")?;
    duckdb
        .execute_batch(
            "
            CREATE TYPE kind AS ENUM ('sheep', 'item');
            CREATE TABLE associations (
                id UINTEGER NOT NULL,
                kind kind NOT NULL,
                tag UINTEGER NOT NULL,
                PRIMARY KEY (id, kind, tag)
            );
            CREATE TABLE seen (
                sheep_id UINTEGER NOT NULL,
                item_id UINTEGER NOT NULL,
                PRIMARY KEY (sheep_id, item_id)
            );
            ",
        )
        .context("Unable to initialize duckdb")?;

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
            } => {
                for (GraphId(id, _), tags) in new_items {
                    for GraphId(tag, _) in tags {
                        duckdb.execute(
                            "INSERT INTO associations (id, kind, tag) VALUES (?, 'item', ?)",
                            params![id, tag]
                        )
                        .context("Unable to insert an item association into the database")?;
                    }
                }
            }
            SimulationEvent::SheepIntroduction {
                sheep: GraphId(id, _),
                associated_tags,
            } => {
                // TODO: there was an error with conflicts here ? what was that
                for GraphId(tag, _) in associated_tags {
                    duckdb.execute(
                        "INSERT INTO associations (id, kind, tag) VALUES (?, 'sheep', ?) ON CONFLICT DO NOTHING",
                        params![id, tag]
                    )
                    .context("Unable to insert a sheep association into the database")?;
                }
            }
            SimulationEvent::FeedRequest {
                sheep: GraphId(id, _),
            } => {
                let mut candidates = HashSet::new();
                let mut tag_query = duckdb
                    .prepare("SELECT tag FROM associations WHERE id = ? AND kind = 'sheep'")
                    .context("Unable to prepare a statement")?;
                let tags = tag_query
                    .query_map([id], |row| row.get("tag"))
                    .context(
                        "Unable to retrieve tags associated with a sheep",
                    )?
                    .collect::<Result<Vec<usize>, _>>()?;

                let mut item_query = duckdb
                    .prepare(
                        "
                        SELECT id FROM associations
                        WHERE NOT EXISTS (
                            SELECT 1
                              FROM seen
                             WHERE seen.sheep_id = ?
                               AND seen.item_id = associations.id
                        )
                        AND tag = ? AND kind = 'item'
                        ",
                    )
                    .context("Unable to prepare a statement")?;

                for tag in tags {
                    candidates.extend(
                        item_query
                            .query_map([id, tag], |row| row.get("id"))
                            .context("Unable to retrieve unseen items")?
                            .collect::<Result<Vec<usize>, _>>()?,
                    );
                }

                let chosen = candidates
                    .into_iter()
                    .choose_multiple(&mut rand::thread_rng(), 10);

                for item in &chosen {
                    duckdb
                        .execute(
                            "INSERT INTO seen (sheep_id, item_id) VALUES (?, ?)",
                            [id, *item]
                        )
                        .context("Unable to mark items as seen")?;
                }

                serde_json::to_writer(
                    &mut stdout,
                    &ShepherdEvent::Feed(Feed(
                        chosen.into_iter().map(GraphId::new).collect(),
                    )),
                )
                .context("Unable to write an event to stdout")?;
                stdout.flush().context("Unable to flush stdout")?;
            }
            _ => (),
        }
    }

    Ok(())
}
