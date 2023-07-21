from autils import *
from math import ceil, floor
import unittest

# Terrain polygons' indexing by coordinate: Finding out what chunk some triangle
# belongs to, given that each chunk is `k` units wide and there isn't space
# between chunks.
#
# A polygon belongs to a chunk when it's center of mass `cm` (x, y) is inside
# the limit [xk, (x + 1)k) for its x and [zk, (z + 1)k) for its z.
#
# Chunks are a collection of polygons. They're referenced by the coordinate
# that describes their position on a grid where one unit is equal to `k`.

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

if __name__ == '__main__':
    unittest.main()
