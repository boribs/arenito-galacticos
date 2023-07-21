from autils import *
import unittest
import math

MAX_DIFF = 0.0001

class VectorTests(unittest.TestCase):
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

class RadianAndDegreeConversionTests(unittest.TestCase):
    def test_deg2rad_zero(self):
        self.assertEqual(0, deg2rad(0))

    def test_deg2rad_180_deg_to_pi_rad(self):
        self.assertEqual(math.pi, deg2rad(180))

    def test_deg2rad_270_deg_to_pi_rad(self):
        self.assertAlmostEqual(4.712389, deg2rad(270))

    def test_deg2rad_33_deg_to_pi_rad(self):
        self.assertAlmostEqual(0.5759587, deg2rad(33))

    def test_rad2deg_zero(self):
        self.assertEqual(0, rad2deg(0))

    def test_rad2deg_180_deg_to_pi_rad(self):
        self.assertEqual(180, rad2deg(math.pi))

    def test_rad2deg_270_deg_to_pi_rad(self):
        self.assertTrue(abs(270 - rad2deg(4.712389)) < MAX_DIFF)

    def test_rad2deg_33_deg_to_pi_rad(self):
        self.assertTrue(abs(33 -rad2deg(0.5759587)) < MAX_DIFF)

class PreviousClosestIntegerMultipleOfKTests(unittest.TestCase):
    def test_pcimk_positive_1(self):
        self.assertEqual(pcimk(5, 2), 4)

    def test_pcimk_positive_2(self):
        self.assertEqual(pcimk(1, 2), 0)

    def test_pcimk_positive_3(self):
        self.assertEqual(pcimk(5, 3), 3)

    def test_pcimk_positive_4(self):
        self.assertEqual(pcimk(9, 3), 9)

    def test_pcimk_negative_1(self):
        self.assertEqual(pcimk(-1, 2), -2)

    def test_pcimk_negative_2(self):
        self.assertEqual(pcimk(-2, 2), -2)

    def test_pcimk_negative_3(self):
        self.assertEqual(pcimk(-1, 3), -3)

    def test_pcimk_negative_4(self):
        self.assertEqual(pcimk(-5, 3), -6)

class NextClosestIntegerMultipleOfKTests(unittest.TestCase):
    def test_ncimk_positive_1(self):
        self.assertEqual(ncimk(1, 3), 3)

    def test_ncimk_positive_2(self):
        self.assertEqual(ncimk(5, 4), 8)

    def test_ncimk_positive_3(self):
        self.assertEqual(ncimk(3, 2), 4)

    def test_ncimk_positive_4(self):
        self.assertEqual(ncimk(2, 4), 4)

    def test_ncimk_negative_1(self):
        self.assertEqual(ncimk(-2, 4), 0)

    def test_ncimk_negative_2(self):
        self.assertEqual(ncimk(-3, 2), -2)

    def test_ncimk_negative_3(self):
        self.assertEqual(ncimk(-1, 2), 0)

    def test_ncimk_negative_4(self):
        self.assertEqual(ncimk(-5, 4), -4)

    @unittest.skip('I\'m not sure I want this')
    def test_ncimk_negative_multiple_returns_same_number(self):
        self.assertEqual(ncimk(-2, 2), -2)

if __name__ == '__main__':
    unittest.main()
