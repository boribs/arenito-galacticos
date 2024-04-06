from interfaces.jetson_interface import JetsonInterface
from arenito_com_consts import *
import argparse, time

parser = argparse.ArgumentParser()
parser.add_argument('port', nargs='?', type=str, default=None)
parser.add_argument('baudrate', nargs='?', type=int, default=115200)
args = parser.parse_args()

jetson_init = time.time()
ji = JetsonInterface(args, True, True, True)
jetson_init = time.time() - jetson_init

instruction_time = time.time()
# ji.send_instruction(Instruction.MoveForward)
print(
    ji.get_prox_sensors()
)
print('Jetson init time:', jetson_init)
print('Instruction time:', time.time() - instruction_time)
