# pyright: strict

from arenito_com_consts import *
from cv2.typing import MatLike
import cv2
from mmap import mmap
from PIL import Image
import numpy as np

class SimInterface:
    """
    Class responsible for interacting with the simulation's shared memory.
    There are a bunch of rules that I'll write later...
    """

    # sync constants
    AI_FRONT_CAM_REQUEST = 1
    AI_REAR_CAM_REQUEST = 6
    SIM_FRAME_WAIT = 2
    AI_MOVE_INSTRUCTION = 3
    SIM_AKNOWLEDGE_INSTRUCTION = 4
    AI_PROX_SENSOR_READ_REQUEST = 5
    AI_DUMP_CANS = 7

    # memory layout
    SYNC_SIZE = 1
    INCOMING_IMAGE_RES = (512, 512)
    IMAGE_SIZE = 786_432
    SENSOR_COUNT_SIZE = 1
    MAX_PROXIMITY_SENSOR_COUNT = 7

    def __init__(self, filename: str):
        self.attach(filename)

    def attach(self, filename: str):
        """
        Attaches to simulation's shared memory.
        """

        # Simulation creates a file on flink (path).
        # Its contents are the name of the shared memory mapping.
        with open(filename, 'r+') as f:
            self.mem = mmap(f.fileno(), length=0)

        # Clear memory!
        self.mem.write(bytes([0] * len(self.mem)))

    def close(self):
        """
        Closes access to shared memory.
        """

        self.mem.close()

    def get_sync_byte(self) -> int:
        """
        Reads sync byte.
        """

        return self.mem[0]

    def set_sync_byte(self, val: int):
        """
        Sets sync byte.
        """

        self.mem[0] = val

    def set_mov_instruction(self, val: int):
        """
        Sets second byte. This should be called only
        when setting a movement instruction.
        """

        self.mem[1] = val

    def get_frame(self, instruction: Instruction) -> MatLike:
        """
        Requests a frame and does some processing for the image
        to be usable by AI.
        """

        self.send_instruction(instruction)

        raw_img = self.mem[1 : SimInterface.IMAGE_SIZE + 1]
        im = Image.frombytes('RGB', SimInterface.INCOMING_IMAGE_RES, raw_img) # pyright: ignore[reportUnknownMemberType]

        # for some reason blue and red channels are swapped?
        # r, g, b = im.split()
        # return np.array(Image.merge('RGB', (b, g, r)))
        return cv2.cvtColor(np.array(im), cv2.COLOR_BGR2RGB)

    def get_front_frame(self) -> MatLike:
        """
        Requests front camera's frame.
        """

        return self.get_frame(Instruction.RequestFrontCam)

    def get_rear_frame(self) -> MatLike:
        """
        Requests rear camera's frame.
        """

        return self.get_frame(Instruction.RequestRearCam)

    def get_prox_sensors(self) -> list[int]:
        """
        Returns proximity sensor reads.
        """

        self.send_instruction(Instruction.RequestProxSensor)
        sensor_count = self.mem[1]

        if sensor_count > SimInterface.MAX_PROXIMITY_SENSOR_COUNT:
            print('Corrup data when reading sensors.')
            return [255] * SimInterface.MAX_PROXIMITY_SENSOR_COUNT

        return list(self.mem[2 : sensor_count + 2])

    def wait_confirmation(self):
        """
        Stalls until sync byte equals SimInterface.SIM_AKNOWLEDGE_INSTRUCTION.
        """

        while self.mem[0] != SimInterface.SIM_AKNOWLEDGE_INSTRUCTION:
            pass

    def send_instruction(self, instr: Instruction):
        """
        Sends an instruction to the simulation.
        """

        if instr == Instruction.RequestFrontCam:
            self.set_sync_byte(SimInterface.AI_FRONT_CAM_REQUEST)
        elif instr == Instruction.RequestRearCam:
            self.set_sync_byte(SimInterface.AI_REAR_CAM_REQUEST)
        elif instr == Instruction.RequestProxSensor:
            self.set_sync_byte(SimInterface.AI_PROX_SENSOR_READ_REQUEST)
        elif instr == Instruction.DumpCans:
            raise Exception('Must use proper dump_cans() method!')
        elif instr == Instruction.ExtendBackdoor:
            return
        else:
            self.set_sync_byte(SimInterface.AI_MOVE_INSTRUCTION)
            self.set_mov_instruction(ord(INSTRUCTION_MAP[instr]))

        self.wait_confirmation()

    def dump_cans(self, ammount: int):
        """
        Dumps cans.
        """

        self.mem[1] = ammount
        self.set_sync_byte(SimInterface.AI_DUMP_CANS)
        self.wait_confirmation()
