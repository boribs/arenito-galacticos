from autils import *
from math import floor
import unittest

# Terrain polygons' indexing by coordinate: Finding out what chunk some triangle
# belongs to, given that each chunk is `k` units wide and there isn't space
# between chunks.
#
# A polygon belongs to a chunk when it's center of mass `cm` (cm_x, cm_z) is inside
# the limit [xk, (x + 1)k) for its x and [zk, (z + 1)k) for its z.
#
# Chunks are a collection of polygons. They're referenced by the coordinate
# that describes their position on a grid where one unit is equal to `k`.
#
# The terrain chunk collection is just a dictionary where each chunk is referenced by
# key. This is to limit the ammount of polygons to search from when arenito moves on
# irregular terrain.
#
# The next step is to find the triangle the wheel is on!

def get_chunk_key(mc: tuple[float], k: int) -> str:
    """
    Given some center of mass `mc` and some chunk size `k`,
    determine which chunk `mc` belongs to.

    Returns the key.
    """

    x, z = mc

    # get previous and next closest integer, multiple of k
    x = pcimk(floor(x), k)
    # round because if ceil, numbers like -0.9 become 0 and changes
    # chabges behaviour drastically.
    # consider test `test_chunk_key_with_float_2`.
    z = ncimk(round(z), k)

    return f'{x},{z}'

# To find the triangle some point (x, z) is on, we need to have the terrain stored somehow.
# Since this is eventually going to be ported to rust (bevy) I'll skip a few things.
# The terrain is divided in chunks. Each chunk has its list of triangles, but there can still
# be a lot of them, and finding it fast is important for this algorithm's efficiency.
#
# The next thing we'll do is finding (quickly) the nearest neighbour (Nearest Neighbour Search),
# proposed on Szymon Tengler and Kornel Warwas' paper "An Effective Algorithm of Uneven Road
# Surface Modeling and Calculating Reaction Forces vor a Vehicle Dynamics Simulation" simplified.
#
# The original algorithm (pages 6 - 9) requires calculating "cutting planes" to limit the
# ammount of searched nodes. I'll be skipping that part, instead selecting arbitrarily any node
# and cut the rest accordingly.

class Node:
    """
    Node for tree-like nearest neighbour search.
    """

    def __init__(self, index: int):
        self.index = index
        self.child = None

def _cut_gt(lim: float, i: int, valid_nodes: list[tuple]):
    """
    Removes vertices with `i` positions greater than the limit.
    """

    # print(f'gt: {i}')

    for v in valid_nodes.copy():
        if v[i] > lim:
            valid_nodes.remove(v)
            # print(f'removing: {v}')

def _cut_lt(lim: float, i: int, valid_nodes: list[tuple]):
    """
    Removes vertices with `i` positions lower than the limit.
    """

    # print(f'lt: {i}')

    for v in valid_nodes.copy():
        if v[i] < lim:
            valid_nodes.remove(v)
            # print(f'removing: {v}')

def cut_x(
        point: tuple[float],
        node: Node,
        vertices: list[tuple],
        valid_nodes: list[tuple]
    ):

    """
    Adds nodes to `node`, expanding the tree. Deletes `valid_nodes` values in the process.

    This function is used specifically for cutting nodes considering some
    `x` component.

    It also calls `cut_z`, creating a recursive chain. The results are a tree with the closest
    nodes to `point` and (maybe) another vertex on `valid_nodes`.
    """

    # limit area considering node.index
    lim = vertices[node.index][0]

    # remove this (pivot) point from avaliable nodes
    valid_nodes.remove(vertices[node.index])

    # print('x lim:', lim)

    if len(valid_nodes) <= 1:
        return

    # if the point is to the right of the limit
    if point[0] > lim:
        # remove every point to the left of the limit
        _cut_lt(lim, 0, valid_nodes)
    else:
        # remove every point to the right (or on the same
        # line) as the limit
        _cut_gt(lim, 0, valid_nodes)

    # select next node
    if len(valid_nodes) <= 2:
        return

    # extend tree, get next pivot
    node.child = Node(vertices.index(valid_nodes[0]))

    # now cut in the other axis
    # print('x:', valid_nodes)
    cut_z(point, node.child, vertices, valid_nodes)

def cut_z(
        point: tuple[float],
        node: Node,
        vertices: list[tuple],
        valid_nodes: list[tuple]
    ):

    """
    Adds nodes to `node`, expanding the tree. Deletes `valid_nodes` values in the process.

    This function is used specifically for cutting nodes considering some
    `z` component.

    It also calls `cut_x`, creating a recursive chain. The results are a tree with the closest
    nodes to `point` and (maybe) another vertex on `valid_nodes`.
    """

    # This ons is exactly the same as cut_x, but with other index.
    # I should use just one function...

    lim = vertices[node.index][2]
    valid_nodes.remove(vertices[node.index])

    # print('z lim:', lim)

    if len(valid_nodes) <= 1:
        return

    if point[2] > lim:
        _cut_lt(lim, 2, valid_nodes)
    else:
        _cut_gt(lim, 2, valid_nodes)

    if len(valid_nodes) <= 2:
        return

    node.child = Node(vertices.index(valid_nodes[0]))
    # print('z:', valid_nodes)
    cut_x(point, node.child, vertices, valid_nodes)

def dist(a: tuple[float], b: tuple[float]) -> float:
    """
    Returns the distance between to points `a` y `b`.
    Doesn't consider the y component.
    """

    return math.sqrt(
        (a[0] - b[0]) ** 2 +
        (a[2] - b[2]) ** 2
    )

def nns(point: tuple[float], vertices: list[tuple]) -> tuple[float]:
    """
    Finds the vertex nearest to `point` using a simplified version of
    the NNS algotithm proposed by Tengler and Warwas.
    """

    root = Node(0) # The first node is not always the best option
                   # but it'll do.

    valid_nodes = vertices.copy()
    # start cutting nodes
    cut_x(point, root, vertices, valid_nodes)

    indexes = [-1, -1]
    node = root
    while node:
        # shift indexes!
        # to remember last two
        indexes[0] = indexes[1]
        indexes[1] = node.index

        # advance node
        node = node.child

    if valid_nodes:
        indexes.append(vertices.index(valid_nodes[0]))

    # compute distances
    distances = [
        (index, dist(vertices[index], point))
        for index in indexes
    ]

    # sort by nearest
    distances = sorted(distances, key=lambda n: n[1])

    # print(distances)

    return vertices[distances[0][0]]

class TerrainTests(unittest.TestCase):
    def test_chunk_key_with_integer_1(self):
        mc = (1, 1)
        k = 2

        self.assertEqual(get_chunk_key(mc, k), "0,2")

    def test_chunk_key_with_integer_2(self):
        mc = (1, -1)
        k = 2

        self.assertEqual(get_chunk_key(mc, k), "0,0")

    def test_chunk_key_with_integer_3(self):
        mc = (-1, -1)
        k = 2

        self.assertEqual(get_chunk_key(mc, k), "-2,0")

    def test_chunk_key_with_float_1(self):
        mc = (0.1, 0.5)
        k = 2

        self.assertEqual(get_chunk_key(mc, k), "0,2")

    def test_chunk_key_with_float_2(self):
        mc = (0.3, -0.9)
        # if ceil for z value this is the same as (0, 0)
        # which results on invalid test.
        k = 2

        self.assertEqual(get_chunk_key(mc, k), "0,0")

    def test_chunk_key_with_float_3(self):
        mc = (-0.5, -1.9)
        k = 2

        self.assertEqual(get_chunk_key(mc, k), "-2,0")

    def test_chunk_key_k_not_2_1(self):
        mc = (-1, -1)
        k = 3

        self.assertEqual(get_chunk_key(mc, k), "-3,0")

    def test_chunk_key_k_not_2_2(self):
        mc = (1, 1.2)
        k = 3

        self.assertEqual(get_chunk_key(mc, k), "0,3")

    def test_chunk_key_k_not_2_3(self):
        mc = (-0.1, 5)
        k = 3

        self.assertEqual(get_chunk_key(mc, k), "-3,6")

def from_2d_to_3d(points: list[tuple]) -> list[tuple]:
    return [
        (point[0], 0, point[1])
        for point in points
    ]

class NNSTests(unittest.TestCase):
    def test__cut_x_lt(self):
        lim = 2
        vertices = [(1, 1, 1), (0, 1, 2), (2, 1, 2), (1.2, 1, 2)]

        _cut_lt(lim, 0, vertices)
        self.assertEqual(vertices, [(2, 1, 2)])

    def test__cut_x_gt(self):
        lim = 1
        vertices = [(0, 1, 3), (1, 1, 1), (0, 1, 2), (2, 1, 2), (1.2, 1, 2)]

        _cut_gt(lim, 0, vertices)
        self.assertEqual(vertices, [(0, 1, 3), (1, 1, 1), (0, 1, 2)])

    def test__cut_z_lt(self):
        lim = 2
        vertices = [(1, 1, 1), (0, 1, 2), (2, 1, 2), (1.2, 1, 2)]

        _cut_lt(lim, 2, vertices)
        self.assertEqual(vertices, [(0, 1, 2), (2, 1, 2), (1.2, 1, 2)])

    def test__cut_z_gt(self):
        lim = 1
        vertices = [(0, 1, 3), (1, 1, 1), (0, 1, 2), (2, 1, 2), (1.2, 1, 2)]

        _cut_gt(lim, 2, vertices)
        self.assertEqual(vertices, [(1, 1, 1)])

    def test_single_child_tree_generation(self):
        point = (0, 0, 0)
        vertices = [(1, 1, 1), (1, 2, 2), (5, 0, 1), (0, 0, 1)]

        self.assertEqual(
            nns(point, vertices),
            (0, 0, 1)
        )

    def test_two_children_tree_generation(self):
        point = (0, 0, 0)
        vertices = [
            (-1, 1, 1), (3, 2, 3), (-2, 0, -2),
            (2, 0, -2), (1.5, 0, 0.5),
        ]

        self.assertEqual(
            nns(point, vertices),
            (-1, 1, 1)
        )

    def test_nns_1(self):
        point = (0, 0, 0)
        vertices = from_2d_to_3d([
            (2, 2), (4, 4), (3, 5), (7, 2),
            (8, 5), (9, 6), (11, 4), (11, 1),
        ])

        self.assertEqual(
            nns(point, vertices),
            (2, 0, 2)
        )

    def test_nns_2(self):
        point = (6, 0, 0)
        vertices = from_2d_to_3d([
            (2, 2), (4, 4), (3, 5), (7, 2),
            (8, 5), (9, 6), (11, 4), (11, 1),
        ])

        self.assertEqual(
            nns(point, vertices),
            (7, 0, 2)
        )

    def test_nns_3(self):
        point = (10, 0, 4)
        vertices = from_2d_to_3d([
            (2, 2), (4, 4), (3, 5), (7, 2),
            (8, 5), (9, 6), (11, 4), (11, 1),
        ])

        self.assertEqual(
            nns(point, vertices),
            (11, 0, 4)
        )

    def test_nns_4(self):
        point = (9.3, 0, 5.5)
        vertices = from_2d_to_3d([
            (2, 2), (4, 4), (3, 5), (7, 2),
            (8, 5), (9, 6), (11, 4), (11, 1),
        ])

        self.assertEqual(
            nns(point, vertices),
            (9, 0, 6)
        )

    def test_nns_5(self):
        point = (2, 0, 2)
        vertices = from_2d_to_3d([
            (2, 2), (4, 4), (3, 5), (7, 2),
            (8, 5), (9, 6), (11, 4), (11, 1),
        ])

        self.assertEqual(
            nns(point, vertices),
            (2, 0, 2)
        )

if __name__ == '__main__':
    unittest.main()
