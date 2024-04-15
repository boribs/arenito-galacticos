from interfaces.jetson_interface import JetsonInterface
from arenito_com_consts import *
import argparse, time

parser = argparse.ArgumentParser()
parser.add_argument('port', nargs='?', type=str, default=None)
parser.add_argument('baudrate', nargs='?', type=int, default=115200)
parser.add_argument('--no_lcd', action=argparse.BooleanOptionalAction, default=True)
args = parser.parse_args()

ji = JetsonInterface(
    args,
    no_cam=True,
    no_start=True,
)

while True:
    reads = ji.get_prox_sensors()
    time.sleep(0.2)
    lu, ru = reads[0:2]
    ir, il = reads[5:7]

    print(reads, ir, il)

    if (lu < 10 and ru < 10) or (ir == 1 or il == 1):
        break

    if abs(lu - ru) > 10:
        if lu > ru:
            ji.send_instruction(Instruction.MoveRight)
        else:
            ji.send_instruction(Instruction.MoveLeft)
    else:
        ji.send_instruction(Instruction.MoveBack)

    time.sleep(0.2)
    ji.send_instruction(Instruction.StopAll)
    time.sleep(0.1)
