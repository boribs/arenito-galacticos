from utils import *
import unittest

class UtilsTest(unittest.TestCase):
    def test_simple_vector_creation(self):
        vec = Vec(1, 1, 1)

        self.assertEqual(vec.x, 1)
        self.assertEqual(vec.y, 1)
        self.assertEqual(vec.z, 1)

    def test_vector_creation_from_iterable(self):
        vec = Vec(*(1, 1.4, 5))

        self.assertEqual(vec.x, 1)
        self.assertEqual(vec.y, 1.4)
        self.assertEqual(vec.z, 5)

    def test_vector_product_by_scalar_operation(self):
        vec = Vec(4, 3, 2) * -1

        self.assertEqual(vec.x, -4)
        self.assertEqual(vec.y, -3)
        self.assertEqual(vec.z, -2)

    def test_vector_product_by_scalar_operation_2(self):
        vec = Vec(-1, 7, 2) * 0.5

        self.assertEqual(vec.x, -0.5)
        self.assertEqual(vec.y, 3.5)
        self.assertEqual(vec.z, 1)

    def test_vector_magnitude(self):
        vec = Vec(1, 1, 1)
        self.assertEqual(vec.magnitude(), 3**0.5)

    def test_vector_magnitude_2(self):
        vec = Vec(3, 1.2, -2)
        self.assertEqual(vec.magnitude(), 3.8)

    def test_vector_magnitude_3(self):
        vec = Vec(-3, -1.2, 2)
        self.assertEqual(vec.magnitude(), 3.8)

    def test_vector_normalization(self):
        vec = Vec(1, 0, 0)
        self.assertEqual(vec.normalize(), Vec(1, 0, 0))

    def test_vector_normalization_2(self):
        vec = Vec(1, 1, 0)
        self.assertEqual(vec.normalize(), Vec(1, 1, 0) / 2**0.5)

    def test_vector_from_angle(self):
        vec = Vec.from_angle(math.pi)
        self.assertEqual(vec, Vec(-1, 0, math.sin(math.pi)))

    def test_vector_from_angle_2(self):
        vec = Vec.from_angle(2 * math.pi)
        self.assertAlmostEqual(vec.z, Vec(1, 0, math.sin(2 * math.pi)).z)

    def test_vector_from_angle_3(self):
        vec = Vec.from_angle(math.pi / 3)
        self.assertAlmostEqual(vec.z, Vec(0.5, 0, math.sin(math.pi / 3)).z)

if __name__ == '__main__':
    unittest.main()
