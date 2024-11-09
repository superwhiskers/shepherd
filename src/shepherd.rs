use petgraph::visit::IntoNeighbors;
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

use crate::{
    graph::Simulation,
    ids::{GraphId, SheepId, TagId},
    simulation::Epoch,
};

/// A wrapper around a child process which implements a feed algorithm
pub struct Shepherd {
    process: Child,
    stdin: ChildStdin,
    stdout: ChildStdout,
}

impl Shepherd {
    /// Create a new [`Shepherd`] from a command name or path
    pub fn new(program: impl AsRef<OsStr>) -> Self {
        let mut process = Command::new(program)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Unable to spawn a shepherd process");

        let stdin = process.stdin.take().expect(
            "Unable to extract the stdin handle from the shepherd process",
        );
        let stdout = process.stdout.take().expect(
            "Unable to extract the stdout handle from the shepherd process",
        );

        Self {
            process,
            stdin,
            stdout,
        }
    }

    /// Write an arbitrary [`SimulationEvent`] to this [`Shepherd`]'s standard
    /// input
    pub fn write_event(&mut self, event: &SimulationEvent) {
        serde_json::to_writer(&self.stdin, event)
            .expect("Unable to pass an event to the shepherd process")
    }

    /// Notify this [`Shepherd`] of the start of a new epoch
    pub fn begin(&mut self, epoch: Epoch) {
        self.write_event(&SimulationEvent::BeginEpoch(epoch))
    }

    /// Introduce this [`Shepherd`] to a sheep
    pub fn introduce_to(&mut self, graph: &Simulation, sheep: SheepId) {
        self.write_event(&SimulationEvent::SheepIntroduction {
            sheep,
            associated_tags: graph.associated_tags(sheep).collect(),
        })
    }
}

#[derive(Serialize)]
#[serde(tag = "kind")]
pub enum SimulationEvent {
    BeginEpoch(Epoch),
    SheepIntroduction {
        sheep: SheepId,
        associated_tags: Vec<TagId>,
    },
}
