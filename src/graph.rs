use itertools::Itertools;
use petgraph::{csr::Csr, prelude::*, visit::IntoNeighbors};
use rand::{distributions::uniform::SampleRange, prelude::*};
use statrs::{distribution::Poisson, StatsError};
use std::collections::HashSet;

use crate::ids::{
    self, GraphId, GraphIdKind, ItemId, NodeType, SheepId, TagId,
};

/// A container type holding the graph organizing the simulation data
///
/// Wraps a [`Csr`] with methods for working with the graph in the manner laid
/// out in the tag graph Jupyter notebook, with some extensions to support
/// gradually building it up across many epochs
#[derive(Default)]
pub struct Simulation(pub Csr<NodeType, u32, Undirected, usize>);

impl Simulation {
    /// Adds several nodes to the simulation
    #[inline(always)]
    pub fn create_nodes<K>(
        &mut self,
        n: usize,
    ) -> impl Iterator<Item = GraphId<K>> + use<'_, K>
    where
        K: ids::GraphIdKind,
    {
        (0..n).map(move |_| GraphId::new(self.0.add_node(K::NODE_TYPE)))
    }

    /// Get the associated tags of either a [`SheepId`] or an [`ItemId`]
    ///
    /// Because of how the simulation graph is assembled, this is able to just
    /// retrieve direct neighbors of the input node and that constitutes the
    /// associated tags
    pub fn associated_tags<K>(
        &self,
        GraphId(id, _): GraphId<K>,
    ) -> impl Iterator<Item = TagId> + use<'_, K>
    where
        K: ids::IsItemOrSheep,
    {
        self.0.neighbors(id).map(GraphId::new)
    }

    /// Forms up to `max_groups` tag groups from the provided tags
    ///
    /// This method builds groups of tags (which are all connected to one
    /// another by edges with weights in the range `5..=10`) and forms edges
    /// across groups (with weights in the range `1..=5`)
    pub fn add_new_tag_groups(
        &mut self,
        rng: &mut (impl Rng + ?Sized),
        groups: &mut Vec<HashSet<TagId>>,
        orphans: &mut HashSet<TagId>,
        max_groups: usize,
        tags: impl IntoIterator<Item = TagId>,
    ) -> Result<(), StatsError> {
        groups.reserve(max_groups);
        let mut tags = tags.into_iter().collect::<Vec<TagId>>();
        tags.shuffle(rng);

        let mut n_stored = 0;
        for mut n_tags in
            Poisson::new((tags.len() / (max_groups + 1)) as f64)?
                .sample_iter(&mut *rng)
                .map(|n| n as usize)
                .take(max_groups)
        {
            if n_stored + n_tags >= tags.len() {
                n_tags = tags.len() - n_stored;
                if n_tags == 0 {
                    break;
                }
            }

            groups.push(
                tags[n_stored..n_stored + n_tags].iter().copied().collect(),
            );
            n_stored += n_tags;
        }
        orphans.extend(tags[n_stored..].iter().copied());

        for group in &*groups {
            for (GraphId(a, _), GraphId(b, _)) in
                group.iter().tuple_combinations()
            {
                self.0.add_edge(*a, *b, rng.gen_range(5..=10));
            }
        }

        for (group_a, group_b) in groups.iter().tuple_combinations() {
            for (GraphId(a, _), GraphId(b, _)) in
                group_a.iter().cartesian_product(group_b)
            {
                if rng.gen::<f64>() <= 0.01 {
                    self.0.add_edge(*a, *b, rng.gen_range(1..=5));
                }
            }
        }

        Ok(())
    }

    /// Adds tags to existing tag groups
    ///
    /// This method adds on tags from the provided tags to the provided groups
    /// and adds any orphans to the provided set. Weights of edges follow the
    /// same rules outlined in the description of `add_new_tag_groups`
    pub fn add_to_tag_groups(
        &mut self,
        rng: &mut (impl Rng + ?Sized),
        groups: &mut [HashSet<TagId>],
        orphans: &mut HashSet<TagId>,
        tags: impl IntoIterator<Item = TagId>,
    ) -> Result<(), StatsError> {
        let mut new_members: Vec<HashSet<TagId>> =
            Vec::with_capacity(groups.len());
        let mut tags = tags.into_iter().collect::<Vec<TagId>>();
        tags.shuffle(rng);

        let mut n_stored = 0;
        for mut n_tags in
            Poisson::new((tags.len() / (groups.len() + 1)) as f64)?
                .sample_iter(&mut *rng)
                .map(|n| n as usize)
                .take(groups.len())
        {
            if n_stored + n_tags >= tags.len() {
                n_tags = tags.len() - n_stored;
                if n_tags == 0 {
                    break;
                }
            }

            new_members.push(
                tags[n_stored..n_stored + n_tags].iter().copied().collect(),
            );
            n_stored += n_tags;
        }
        orphans.extend(tags[n_stored..].iter().copied());

        for (i, members) in new_members.iter().enumerate() {
            for (GraphId(a, _), GraphId(b, _)) in
                members.iter().tuple_combinations()
            {
                self.0.add_edge(*a, *b, rng.gen_range(5..=10));
            }

            for (GraphId(a, _), GraphId(b, _)) in
                members.iter().cartesian_product(groups[i].iter())
            {
                self.0.add_edge(*a, *b, rng.gen_range(5..=10));
            }
        }

        for (i, j) in (0..groups.len()).tuple_combinations() {
            for (GraphId(a, _), GraphId(b, _)) in
                new_members[i].iter().cartesian_product(groups[j].iter())
            {
                if rng.gen::<f64>() <= 0.01 {
                    self.0.add_edge(*a, *b, rng.gen_range(1..=5));
                }
            }
        }

        for (group, new_members) in groups.iter_mut().zip(new_members) {
            group.extend(new_members);
        }

        Ok(())
    }

    /// Adds singular edges between nodes specified in the `source_nodes` and
    /// `target_nodes` lists
    ///
    /// A number of edges within the range specified by `edge_bounds` will be
    /// added from a source node to distinct target nodes. A weight in the
    /// range `1..=10` is assigned to the edge, sampled from a discrete
    /// uniform distribution
    pub fn connect_extremities<K>(
        &mut self,
        rng: &mut (impl Rng + ?Sized),
        source_nodes: impl IntoIterator<Item = GraphId<K>>,
        target_nodes: impl IntoIterator<Item = TagId> + Clone,
        edge_bounds: impl SampleRange<usize> + Clone,
    ) where
        K: ids::IsItemOrSheep,
    {
        // TODO: add some behavior here where we "magically" connect new tags
        //       to source nodes. the chance of this happening will be
        //       computed based on the weight associated with the edge between
        //       the source node and target node combined with the weight
        //       associated with the edge between the candidate tag and the
        //       tag already connected to the source node

        for GraphId(source, _) in source_nodes {
            let n_edges = rng.gen_range(edge_bounds.clone());
            for GraphId(tag, _) in target_nodes
                .clone()
                .into_iter()
                .choose_multiple(rng, n_edges)
            {
                self.0.add_edge(source, tag, rng.gen_range(1..=10));
            }
        }
    }
}
