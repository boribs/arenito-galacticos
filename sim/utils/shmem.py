from multiprocessing import shared_memory, resource_tracker
import time
from PIL import Image
import cv2
import numpy as np

def remove_shm_from_resource_tracker():
    """Monkey-patch multiprocessing.resource_tracker so SharedMemory won't be tracked

    More details at: https://bugs.python.org/issue38119
    """

    def fix_register(name, rtype):
        if rtype == "shared_memory":
            return
        return resource_tracker._resource_tracker.register(self, name, rtype)
    resource_tracker.register = fix_register

    def fix_unregister(name, rtype):
        if rtype == "shared_memory":
            return
        return resource_tracker._resource_tracker.unregister(self, name, rtype)
    resource_tracker.unregister = fix_unregister

    if "shared_memory" in resource_tracker._CLEANUP_FUNCS:
        del resource_tracker._CLEANUP_FUNCS["shared_memory"]

class AISimMem:
    # sync constants
    AI_FRAME_REQUEST = 1
    SIM_FRAME_WAIT = 2
    AI_MOVE_INSTRUCTION = 3
    SIM_AKNOWLEDGE_INSTRUCTION = 4

    # movement instruction constants
    MOV_FORWARD = 10
    MOV_LEFT = 11
    MOV_RIGHT = 12

    # memory footprint
    IMG_SIZE = 3_145_728

    def __init__(self, mem: shared_memory.SharedMemory):
        self.mem = mem

    def write_byte(self, val: int):
        self.mem.buf[0] = val

    def write_mov_instruction(self, val: int):
        self.mem.buf[1] = val

    def read_byte(self) -> int:
        return self.mem.buf[0]

    def wait_confirmation(self):
        while self.mem.buf[0] != AISimMem.SIM_AKNOWLEDGE_INSTRUCTION:
            pass

def mv(aisim: AISimMem, mov: int):
    aisim.write_mov_instruction(mov)
    aisim.write_byte(AISimMem.AI_MOVE_INSTRUCTION)
    aisim.wait_confirmation()

remove_shm_from_resource_tracker() # Python 3.12 and under

with open('../shmem_arenito') as f:
    osid = f.read()[1:]

mem = shared_memory.SharedMemory(create=False, name=osid)
aisim = AISimMem(mem)

# constantly ask for images!
while True:
    # aisim.write_mov_instruction(AISimMem.MOV_FORWARD)
    # aisim.write_byte(AISimMem.AI_MOVE_INSTRUCTION)

    # # I don't like this delay
    # if cv2.waitKey(1) == 27:
    #     break

    # aisim.wait_confirmation()

    for _ in range(3):
        mv(aisim, AISimMem.MOV_FORWARD)

    for _ in range(10):
        mv(aisim, AISimMem.MOV_LEFT)

    # im = Image.frombytes('RGB', (1024, 1024), aisim.mem.buf[1:3145728 + 1])
    # # for some reason blue and red channels are swapped?
    # r, g, b = im.split()
    # im = Image.merge('RGB', (b, g, r))

    # cv2.imshow('sakldhjf', np.array(im))
