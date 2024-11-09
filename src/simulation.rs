use rand::prelude::*;
use serde::{Deserialize, Serialize};
use statrs::StatsError;
use std::collections::{HashMap, HashSet};

use crate::{
    feed::{Feed, Responses},
    graph::Simulation as SimulationGraph,
    ids::{EpochId, ItemId, SheepId, ShepherdId, TagId},
    shepherd::{Shepherd, SimulationEvent},
};

/// Settings for the simulation
pub struct Settings {
    /// Bounds on the number of tags added at the start of each epoch
    pub n_tags_bounds: (usize, usize),

    /// Bounds on the number of items added at the start of each epoch
    pub n_items_bounds: (usize, usize),

    /// Bounds on the number of tags assigned to a new Item
    pub n_item_tags_bounds: (usize, usize),

    /// Bounds on the number of tags a sheep has
    pub n_sheep_tags_bounds: (usize, usize),

    /// Bounds on the initial number of tags used to seed the simulation
    pub initial_n_tags_bounds: (usize, usize),

    /// Bounds on the initial number of items used to seed the simulation
    pub initial_n_items_bounds: (usize, usize),

    /// Bounds on the initial number of sheep added to the simulation
    pub initial_n_sheep_bounds: (usize, usize),

    /// An approximate measure of how many tags belong in a group
    ///
    /// This is used to determine the upper limit on how many groups should be
    /// added when there is a sufficient amount of tags orphaned from a group
    ///
    /// Additionally, this is used when adding the first tags. This should be
    /// at most the lower bound of `initial_n_tags_bounds`, but ideally much
    /// lower than that
    pub average_tags_per_group: usize,

    /// The threshold of orphaned tags at which new groups will be formed
    ///
    /// This should be at most the lower bound of `initial_n_tags_bounds`
    pub orphaned_tag_threshold: usize,

    /// Hook that is called when a new epoch is started
    pub new_epoch_hook: Option<Box<dyn FnMut(EpochId, &Epoch)>>,

    /// Hook that is called when a [`Shepherd`] has generated a [`Feed`] for
    /// a sheep
    pub feed_generation_hook:
        Option<Box<dyn FnMut(ShepherdId, SheepId, &Feed)>>,

    /// Hook that is called when a sheep has finished rating a [`Feed`] given
    /// by a [`Shepherd`]
    pub feed_rated_hook:
        Option<Box<dyn FnMut(ShepherdId, SheepId, &Responses)>>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            n_tags_bounds: (0, 2),
            n_items_bounds: (0, 50),
            n_item_tags_bounds: (5, 10),
            n_sheep_tags_bounds: (5, 25),
            initial_n_tags_bounds: (20, 40),
            initial_n_items_bounds: (40, 60),
            initial_n_sheep_bounds: (20, 40),
            average_tags_per_group: 7,
            orphaned_tag_threshold: 20,
            new_epoch_hook: None,
            feed_generation_hook: None,
            feed_rated_hook: None,
        }
    }
}

/// A representation of the tags and content introduced at the beginning of a
/// new epoch within the simulation
#[derive(Serialize, Deserialize)]
pub struct Epoch {
    /// Tags introduced at the beginning of this epoch
    pub tags: Vec<TagId>,

    /// Items introduced at the beginning of this epoch
    pub items: Vec<ItemId>,
}

/// A container for the state associated with a simulation
#[derive(Default)]
pub struct Simulation {
    /// The epoch counter
    current_epoch: EpochId,

    /// The simulation graph
    graph: SimulationGraph,

    /// The settings of the simulation
    settings: Settings,

    /// Tags present in the simulation
    tags: Vec<TagId>,

    /// Sheep present in the simulation
    sheep: Vec<SheepId>,

    /// Items present in the simulation
    items: Vec<ItemId>,

    /// Tag groups present in the simulation
    tag_groups: Vec<HashSet<TagId>>,

    /// Orphaned tags present in the simulation
    tag_orphans: HashSet<TagId>,

    /// [`Shepherd`]s present within the simulation and a map keeping track of
    /// the items each one has shown each sheep
    shepherds: Vec<(Shepherd, HashMap<SheepId, HashSet<ItemId>>)>,
}

impl Simulation {
    pub fn new(
        rng: &mut (impl Rng + ?Sized),
        shepherds: impl IntoIterator<Item = Shepherd>,
        settings: Settings,
    ) -> Result<Self, StatsError> {
        let mut simulation = Self {
            settings,
            shepherds: shepherds
                .into_iter()
                .map(|s| (s, Default::default()))
                .collect(),
            ..Default::default()
        };

        simulation
            .tags
            .extend(simulation.graph.create_nodes(rng.gen_range(
                simulation.settings.initial_n_tags_bounds.0
                    ..=simulation.settings.initial_n_tags_bounds.1,
            )));

        simulation.graph.add_new_tag_groups(
            &mut *rng,
            &mut simulation.tag_groups,
            &mut simulation.tag_orphans,
            simulation.tags.len()
                / simulation.settings.average_tags_per_group,
            simulation.tags.iter().copied(),
        )?;

        simulation.sheep.extend(simulation.graph.create_nodes(
            rng.gen_range(
                simulation.settings.initial_n_sheep_bounds.0
                    ..=simulation.settings.initial_n_sheep_bounds.1,
            ),
        ));
        simulation.graph.connect_extremities(
            &mut *rng,
            simulation.sheep.iter().copied(),
            simulation.tags.iter().copied(),
            simulation.settings.n_sheep_tags_bounds.0
                ..=simulation.settings.n_sheep_tags_bounds.1,
        );

        simulation.items.extend(simulation.graph.create_nodes(
            rng.gen_range(
                simulation.settings.initial_n_items_bounds.0
                    ..=simulation.settings.initial_n_items_bounds.1,
            ),
        ));
        simulation.graph.connect_extremities(
            &mut *rng,
            simulation.items.iter().copied(),
            simulation.tags.iter().copied(),
            simulation.settings.n_item_tags_bounds.0
                ..=simulation.settings.n_item_tags_bounds.1,
        );

        let introduction_epoch = Epoch {
            tags: simulation.tags.clone(),
            items: simulation.items.clone(),
        };

        if let Some(hook) = &mut simulation.settings.new_epoch_hook {
            hook(simulation.current_epoch, &introduction_epoch);
        }

        let introduction_epoch =
            SimulationEvent::BeginEpoch(introduction_epoch);
        for (shepherd, _) in &mut simulation.shepherds {
            shepherd.write_event(&introduction_epoch);
            for sheep in simulation.sheep.iter().copied() {
                shepherd.introduce_to(&simulation.graph, sheep);
            }
        }

        Ok(simulation)
    }
}
