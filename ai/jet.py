from interfaces.jetson_interface import JetsonInterface
from arenito_com_consts import *
import argparse, time

parser = argparse.ArgumentParser()
parser.add_argument('instr', type=str, default=None)
parser.add_argument('port', nargs='?', type=str, default=None)
parser.add_argument('baudrate', nargs='?', type=int, default=115200)
args = parser.parse_args()

instr = eval(f'Instruction.{args.instr}')

jetson_init = time.time()
ji = JetsonInterface(
    args,
    no_cam=True,
    no_start=True,
    no_lcd=True,
)
jetson_init = time.time() - jetson_init

instruction_time = time.time()

if instr == Instruction.RequestProxSensor:
    print(ji.get_prox_sensors())
else:
    ji.send_instruction(instr)

print('Jetson init time:', jetson_init)
print('Instruction time:', time.time() - instruction_time)
