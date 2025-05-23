from interfaces.jetson_interface import JetsonInterface
from arenito_com_consts import *
from arenito_logger import ArenitoLogger
import argparse, time
from PIL import Image

parser = argparse.ArgumentParser()
parser.add_argument('port', nargs='?', type=str, default='/dev/ttyUSB0')
parser.add_argument('instr', type=str, default=None)
parser.add_argument('baudrate', nargs='?', type=int, default=115200)
parser.add_argument('--no_lcd', '-L', action='store_true', default=True)
parser.add_argument('--exposure', '-e', type=str, default='auto')
parser.add_argument('--no-button', action='store_true', default=True)
parser.add_argument('--save_images', '-s', type=str, default='')
parser.add_argument('--print_log', '-l', action='store_true', default=False)
args = parser.parse_args()

instr = eval(f'Instruction.{args.instr}')
cam = instr in (Instruction.RequestFrontCam, Instruction.RequestRearCam)

jetson_init = time.time()
ji = JetsonInterface(
    args,
    ArenitoLogger(args),
    no_cam=not cam,
    no_start=True,
)
jetson_init = time.time() - jetson_init
instruction_time = time.time()

if instr == Instruction.RequestProxSensor:
    print(ji.get_prox_sensors())

elif instr == Instruction.RequestFrontCam:
    for _ in range(5):
        ji.get_front_frame()
    img = ji.get_front_frame()
    Image.fromarray(img[:, :, ::-1]).save('ff.jpg')

elif instr == Instruction.RequestRearCam:
    for _ in range(5):
        ji.get_rear_frame()
    img = ji.get_rear_frame()
    Image.fromarray(img[:, :, ::-1]).save('rf.jpg')

else:
    ji.send_instruction(instr)

print('Jetson init time:', jetson_init)
print('Instruction time:', time.time() - instruction_time)
