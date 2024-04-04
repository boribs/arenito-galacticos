from interfaces.jetson_interface import JetsonInterface
from arenito_com_consts import *
import argparse

parser = argparse.ArgumentParser()
parser.add_argument('port', nargs='?', type=str, default=None)
parser.add_argument('baudrate', nargs='?', type=int, default=115200)
args = parser.parse_args()

ji = JetsonInterface(args, True, True, True)
ji.send_instruction(Instruction.MoveForward)
