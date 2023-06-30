from __future__ import annotations
import math

"""
Just random utilities to facilitate testing :)
"""

class Vec:
    """
    A very simple Vector implementation.
    """

    def __init__(self, x, y, z):
        self.x = x
        self.y = y
        self.z = z

    def __add__(self, other) -> Vec:
        if type(other) == type(self):
            return Vec(self.x + other.x, self.y + other.y, self.z + other.z)
        else:
            raise Exception(f'Can\'t add Vector and {type(other)}.')

    def __eq__(self, other) -> Vec:
        if type(other) == type(self):
            return (
                self.x == other.x and
                self.y == other.y and
                self.z == other.z
            )
        else:
            return False

    def __mul__(self, other) -> Vec:
        if type(other) == int or type(other) == float:
            return Vec(self.x * other, self.y * other, self.z * other)
        else:
            raise Exception(f'Can\'t add Vector and {type(other)}.')

    def __truediv__(self, other) -> Vec:
        if type(other) == int or type(other) == float:
            return Vec(self.x / other, self.y / other, self.z / other)
        else:
            raise Exception(f'Can\'t add Vector and {type(other)}.')

    def __repr__(self):
        return f'({self.x}, {self.y}, {self.z})'

    @staticmethod
    def zero():
        return Vec(0, 0, 0)

    @staticmethod
    def from_angle(angle: float):
        return Vec(math.cos(angle), 0, math.sin(angle))

    def magnitude(self) -> float:
        return math.sqrt(self.x**2 + self.y**2 + self.z**2)

    def normalize(self) -> Vec:
        m = self.magnitude()

        if m == 0:
            return self.zero()

        return self / m

def calc_arenito_update(
        acc: Vec = Vec.zero(),
        vel: Vec = Vec.zero(),
        cen: Vec = Vec.zero(),
        fric_k: float = 0.5,
        time: float = 0.016
) -> tuple[Vec, Vec, Vec]:
    """
    Calculates the "next position" of Arenito.
    Prints calculated values: Acceleration, Velocity, Center (position) and Friction.

    This is mainly for my tests. I don't want to test using the same code I'm testing
    so this is another way of calculating the same thing.
    """

    fric = acc.normalize() * -fric_k
    acc = acc + fric
    vel = (acc * time) + vel
    cen = (vel * time) + (acc * time**2 * 0.5) + cen

    if acc.magnitude() < fric_k:
        acc = Vec.zero()
        vel = Vec.zero()

    return (fric, acc, vel, cen)

def random_angles(n: int) -> tuple[float]:
    """
    Returns a tuple with n angles (in radians) in the range [-2π, 2π].
    """

    NEG = (True, False)
    return (random() * math.tau * (-1 if choice(NEG) else 1) for _ in range(n))

    acc = Vec.from_angle(angle) * 4
    _, acc, vel, cen = calc_arenito_update(acc = acc)
    print(f'angle: {angle}: \n\tacc: {acc}\n\tvel: {vel}\n\tcen: {cen}')

def arenito_basic_movement_from_standstill():
    for i in range(9):
        angle = i * (math.pi / 4)
        arenito_basic_movement_from_standstill_angle(angle)

def random_angle_basic_movement_standstill(n: int):
    for a in random_angles(n):
        arenito_basic_movement_from_standstill_angle(a)

if __name__ == '__main__':
    pass
