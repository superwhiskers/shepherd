use petgraph::algo;
use rand::prelude::*;
use std::ops::Neg;
use tracing::info;

use crate::{
    feed::{Feed, Response, Responses},
    graph::Simulation,
    ids::SheepId,
};

/// Calculate the probability of a positive rating given the input sum of
/// weights along the shortest path
pub fn p_positive(distance: f64) -> f64 {
    2f64.powf(distance.neg())
}

/// Calculate the probability of a neutral rating given the input sum of
/// weights along the shortest path
pub fn p_neutral(distance: f64) -> f64 {
    9f64.powf(distance) / 10f64.powf(distance)
}

/// Process a feed given the tag graph, sheep id, and feed
pub fn process_feed(
    rng: &mut (impl Rng + ?Sized),
    graph: &Simulation,
    sheep: SheepId,
    feed: Feed,
) -> Responses {
    let mut responses = Vec::with_capacity(feed.0.len());

    for item in feed.0 {
        responses.push((
            item,
            if let Some(distance) =
                algo::dijkstra(&graph.0, sheep.0, Some(item.0), |e| {
                    *e.weight()
                })
                .get(&item.0)
            {
                match rng.gen::<f64>() {
                    c if c <= p_positive(f64::from(*distance)) => {
                        info!(
                            sheep = sheep.0,
                            item = item.0,
                            distance = distance,
                            probability = c,
                            threshold = p_positive(f64::from(*distance)),
                            rating = "positive"
                        );
                        Response::Positive
                    }
                    c if c <= p_neutral(f64::from(*distance)) => {
                        info!(
                            sheep = sheep.0,
                            item = item.0,
                            distance = distance,
                            probability = c,
                            threshold = p_neutral(f64::from(*distance)),
                            rating = "neutral"
                        );
                        Response::Neutral
                    }
                    _ => {
                        info!(
                            sheep = sheep.0,
                            item = item.0,
                            distance = distance,
                            rating = "negative"
                        );

                        Response::Negative
                    }
                }
            } else {
                // to keep the model simple, we always respond negatively to
                // content for which no path exists
                //
                // the assumptions being made here for this to work are:
                // - the tag graph is taken to be axiomatic
                // - everything is comprehensively tagged and no more existing
                //   tags fit
                info!(
                    sheep = sheep.0,
                    item = item.0,
                    distance = "unconnected",
                    rating = "negative"
                );
                Response::Negative
            },
        ));
    }

    Responses(responses)
}
