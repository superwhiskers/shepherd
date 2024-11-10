use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// An identifier for an epoch within the simulation
#[repr(transparent)]
#[derive(
    Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct EpochId(pub usize);

/// An identifier for a shepherd within the simulation
#[repr(transparent)]
#[derive(
    Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct ShepherdId(pub usize);

/// An identifier that represents a tag within the simulation
pub type TagId = GraphId<Tag>;

/// An identifier that represents a sheep within the simulation
pub type SheepId = GraphId<Sheep>;

/// An identifier that represents an item within the simulation
pub type ItemId = GraphId<Item>;

/// An identifier that relates to an item on the simulation graph
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GraphId<K: GraphIdKind>(pub usize, pub PhantomData<K>);

impl<K> GraphId<K>
where
    K: GraphIdKind,
{
    /// Makes a new [`GraphId`] given the identifier
    pub fn new(id: usize) -> Self {
        Self(id, PhantomData)
    }
}

/// A trait represeting the kinds of identifiers that relate to the
/// simulation graph
#[allow(private_bounds)]
pub trait GraphIdKind: SealedGraphIdKind {
    const NODE_TYPE: NodeType;
}

/// Sealed trait preventing external implementations of [`GraphIdKind`]
trait SealedGraphIdKind {
    const NODE_TYPE: NodeType;
}

impl<T> GraphIdKind for T
where
    T: SealedGraphIdKind,
{
    const NODE_TYPE: NodeType = <T as SealedGraphIdKind>::NODE_TYPE;
}

/// The identifier represents a tag within the simulation
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Tag;

impl SealedGraphIdKind for Tag {
    const NODE_TYPE: NodeType = NodeType::Tag;
}

/// The identifier represents a sheep within the simulation
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Sheep;

impl SealedGraphIdKind for Sheep {
    const NODE_TYPE: NodeType = NodeType::Sheep;
}

/// The identifier represents an item within the simulation
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Item;

impl SealedGraphIdKind for Item {
    const NODE_TYPE: NodeType = NodeType::Item;
}

/// A trait indicating if the id is either an [`Item`] or a [`Sheep`]
pub trait IsItemOrSheep: GraphIdKind {}

impl IsItemOrSheep for Item {}

impl IsItemOrSheep for Sheep {}

/// An enumeration over the kinds of nodes in the tag graph
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum NodeType {
    /// A sheep (user) in the simulation
    Sheep,

    /// A tag in the simulation
    Tag,

    /// An item (content) in the simulation
    Item,
}
