use serde::{Deserialize, Serialize};

use crate::ids::ItemId;

/// An enum indicating a Sheep's response to a [`Feed`] item
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum Response {
    /// A positive response to a feed item
    Positive,

    /// A neutral response to a feed item
    Neutral,

    /// A negative response to a feed item
    Negative,
}

/// The [`Response`]s returned from a Sheep after evaluating a [`Feed`]
///
/// The first two values are self-explanatory, the third is a count of how many
/// hops are required to get from the Sheep to the feed item (if it is
/// reachable)
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Responses(pub Vec<(ItemId, Response, Option<u32>)>);

/// The feed returned from a [`Shepherd`], prepared for a specific Sheep
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Feed(pub Vec<ItemId>);
