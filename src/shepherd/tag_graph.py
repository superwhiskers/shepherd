import random
from itertools import combinations, product
from ulid import ULID

import matplotlib.pyplot as plt
import networkx as nx
import numpy as np


class SimulationGraph:
    """
    A container type holding the graph organizing the simulation data

    Wraps a networkx.Graph with methods for working with the graph in the
    manner laid out in the tag_graph Jupyter notebook, with some extensions to
    support gradually building it up across many epochs
    """

    graph: nx.Graph

    def __init__(self) -> None:
        self.graph = nx.Graph()

    def add_users(self, n_users: int) -> list[ULID]:
        """
        Adds a specified amount of users to the simulation, returning their
        identifiers
        """
        users = [ULID() for _ in range(n_users)]
        self.graph.add_nodes_from(users, node_type="user", color="#ffb8b8")
        return users

    def add_tags(self, n_tags: int) -> list[ULID]:
        """
        Adds a specified amount of tags to the simulation, returning their
        identifiers
        """
        tags = [ULID() for _ in range(n_tags)]
        self.graph.add_nodes_from(tags, node_type="tag", color="#00eb00")
        return tags

    def add_content(self, n_content: int) -> list[ULID]:
        """
        Adds a specified amount of content to the simulation, returning their
        identifiers
        """
        content = [ULID() for _ in range(n_content)]
        self.graph.add_nodes_from(content, node_type="content", color="#c7c7ff")
        return content

    def add_new_tag_groups(
        self, max_groups: int, tags: list[ULID]
    ) -> tuple[list[list[ULID]], list[ULID]]:
        """
        Forms up to max_groups tag groups from the provided tags

        This method builds "groups" of tags (which are all connected to one
        another by edges with weights in the range [0.5, 1.0]) and forms edges
        across groups (with weights in the range [0.1, 0.5]). Returns a tuple
        containing nested lists of tag groups and a separate list of orphaned
        tags (tags not part of a group)
        """
        rng = np.random.default_rng()
        average_tags = len(tags) // (max_groups + 1)
        random.shuffle(tags)

        t = 0
        tag_groups = []
        for _ in range(max_groups):
            n = rng.poisson(average_tags)
            if t + n >= len(tags):
                n = len(tags) - t
                if n == 0:
                    break

            tag_groups.append(tags[t : t + n])
            t += n

        orphans = tags[t:]

        for group in tag_groups:
            for a, b in combinations(group, 2):
                weight = rng.uniform(0.5, 1.0)
                self.graph.add_edge(a, b, weight=weight)

        for group_a, group_b in combinations(tag_groups, 2):
            for a, b in product(group_a, group_b):
                if rng.random() <= 0.01:
                    weight = rng.uniform(0.1, 0.5)
                    self.graph.add_edge(a, b, weight=weight)

        return (tag_groups, orphans)

    def add_to_tag_groups(
        self, groups: list[list[ULID]], tags: list[ULID]
    ) -> tuple[list[list[ULID]], list[ULID]]:
        """
        Adds tags to existing tag groups

        This method takes a list of tag groups and adds on tags from the
        provided tags to these groups. Weights of edges follow the same rules
        outlined in the description of `add_new_tag_groups`. Returns a tuple
        containing nested lists of tag groups and a separate list of orphaned
        tags
        """
        rng = np.random.default_rng()
        average_tags = len(tags) // (len(groups) + 1)
        random.shuffle(tags)

        t = 0
        new_members = []
        for _ in range(len(groups)):
            n = rng.poisson(average_tags)
            if t + n >= len(tags):
                n = len(tags) - t
                if n == 0:
                    break

            new_members.append(tags[t : t + n])
            t += n

        orphans = tags[t:]

        for i, members in enumerate(new_members):
            for a, b in combinations(members, 2):
                weight = rng.uniform(0.5, 1.0)
                self.graph.add_edge(a, b, weight=weight)

            for a, b in product(members, groups[i]):
                weight = rng.uniform(0.5, 1.0)
                self.graph.add_edge(a, b, weight=weight)

        for i, j in combinations(list(range(len(groups))), 2):
            for a, b in product(new_members[i], groups[j]):
                if rng.random() <= 0.01:
                    weight = rng.uniform(0.1, 0.5)
                    self.graph.add_edge(a, b, weight=weight)

        for i, group in enumerate(groups):
            group.extend(new_members[i])

        return (groups, orphans)

    def connect_extremities(
        self,
        source_nodes: list[ULID],
        target_nodes: list[ULID],
        maximum_edges: int = 10,
    ) -> None:
        """
        Adds singular edges between nodes specified in the source_nodes and
        target_nodes lists

        From one up to maximum_edges edges may be added from a source node to
        distinct target nodes. A weight in the range [0.1, 1.0] is assigned to
        the edge, sampled from a uniform distribution
        """
        for source in source_nodes:
            num_edges = random.randint(1, maximum_edges)
            connected_tags = random.sample(
                target_nodes, min(num_edges, len(target_nodes))
            )
            for tag in connected_tags:
                weight = random.uniform(0.1, 1.0)
                self.graph.add_edge(source, tag, weight=weight)
