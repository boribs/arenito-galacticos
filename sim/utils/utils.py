from __future__ import annotations
import math
from random import random, choice

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
        return '({:.5f}, {:.5f}, {:.5f})'.format(self.x, self.y, self.z)

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

def deg2rad(a: float) -> float:
    return a * math.pi / 180

def rad2deg(a: float) -> float:
    return a * 180 / math.pi

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

def arenito_basic_movement_from_standstill_angle(angle: float):
    acc = Vec.from_angle(angle) * 4
    _, acc, vel, cen = calc_arenito_update(acc = acc)
    print(f'angle: {angle}: \n\tvel: {vel}\n\tacc: {acc}\n\tcen: {cen}')

def arenito_basic_movement_from_standstill(n: int):
    for i in range(n):
        angle = i * (math.pi / 4)
        arenito_basic_movement_from_standstill_angle(angle)

def random_angle_basic_movement_standstill(n: int):
    for a in random_angles(n):
        arenito_basic_movement_from_standstill_angle(a)

def arenito_basic_movement_from_motion(angle: float, vel_k: float):
    vel = Vec.from_angle(angle) * vel_k
    acc = Vec.from_angle(angle) * 4
    cen = Vec(0, 0.5, 0)

    _, nacc, nvel, ncen = calc_arenito_update(acc = acc, vel = vel, cen = cen)

    # I'm lazy enough to make the utility print the body of the test:)
    print(f'''
        let mut arenito = Arenito::vel_acc(
            Vec3::new{vel},
            Vec3::new{acc},
            Vec3::new{cen},
        );
        arenito.update_pos(16);

        let expected_vel = Vec3::new{nvel};
        let expected_acc = Vec3::new{nacc};
        let expected_center = Vec3::new{ncen};

        cmp_vec(arenito.vel, expected_vel);
        cmp_vec(arenito.acc, expected_acc);
        cmp_vec(arenito.center, expected_center);\n
    ''')

def random_basic_movement_from_motion(n: int):
    for a in random_angles(n):
        arenito_basic_movement_from_motion(a, 1 + random())

if __name__ == '__main__':
    pass
