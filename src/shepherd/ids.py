from typing import NewType
from ulid import ULID


"""A tag within the simulation"""
TagId = NewType("TagId", ULID)


"""A Sheep's ID"""
SheepId = NewType("SheepId", ULID)


"""A Shepherd's ID"""
ShepherdId = NewType("ShepherdId", ULID)


"""An Epoch's ID"""
EpochId = NewType("EpochId", ULID)


"""An Item's ID"""
ItemId = NewType("ItemId", ULID)
