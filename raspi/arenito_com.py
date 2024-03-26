# pyright: strict

import subprocess, cv2
import numpy as np
from argparse import Namespace
from cv2.typing import MatLike
from enum import Enum, auto
from serial import Serial
from PIL import Image
from mmap import mmap

class AIMode(Enum):
    Simulation = auto()
    Real = auto()

class Instruction(Enum):
    MoveForward = auto()
    MoveLeft = auto()
    MoveRight = auto()
    MoveBack = auto()
    MoveLongRight = auto()
    RequestScan = auto()

INSTRUCTION_MAP = {
    Instruction.MoveForward: 'a',
    Instruction.MoveLeft: 'i',
    Instruction.MoveRight: 'd',
    Instruction.MoveBack: 'r',
    Instruction.MoveLongRight: 'D',
}

class ArenitoComms:
    """
    Interface between Arenito's AI and other devices.
    This class is responisble for capturing (raw) images from the camera
    and communicating with the Arduino board.
    Also gets information from and to the simulation.
    """

    def __init__(self, mode: AIMode, args: Namespace):
        self.serial: SerialInterface | None = None
        self.video_capture: cv2.VideoCapture | None = None
        self.sim_interface: SimInterface | None = None

        if mode == AIMode.Simulation:
            self.connect_simulation(args.filename)
        elif mode == AIMode.Real:
            self.connect_serial(args.port, args.baudrate, args.timeout)
            self.init_video()
        else:
            raise Exception(f'No such mode {mode}')

    def init_video(self, device_index: int = 0):
        """
        Initializes the capture device.
        """

        self.video_capture = cv2.VideoCapture(device_index)

    def connect_serial(self, port: str | None, baudrate: int, timeout: float = 0.0):
        """
        Establishes serial communication.
        """

        self.serial = SerialInterface(port, baudrate, timeout)

    def connect_simulation(self, filename: str):
        """
        Attaches to simulation's shared memory.
        """

        self.sim_interface = SimInterface(filename)

    def get_data(self) -> tuple[MatLike, list[int]]:
        """
        Gets the image from the camera.
        """

        if self.video_capture:
            ok, frame = self.video_capture.read()
            if not ok:
                raise Exception('Couldn\'t get frame.')

            return (frame, [])
        else:
            return self.sim_interface.get_data() # pyright: ignore[reportOptionalMemberAccess]

    def send_instruction(self, instr: Instruction):
        """
        Sends instruction to arduino board through serial interface.
        """

        if self.serial:
            self.serial.send_instruction(instr)
        else:
            self.sim_interface.send_instruction(instr) # pyright: ignore[reportOptionalMemberAccess]

class SerialInterface:
    def __init__(self, port: str | None, baudrate: int, timeout: float = 0.0):
        self.connect(port, baudrate, timeout)

    def connect(self, port: str | None, baudrate: int, timeout: float):
        """
        Establishes serial communication.
        """

        if port is None: port = SerialInterface.find_port()
        self.serial = Serial(port=port, baudrate=baudrate, timeout=timeout)

    @staticmethod
    def find_port() -> str:
        """
        Finds out where the Arduino borad is connected. Requires `arduino-cli`.
        """

        out = subprocess.run(["arduino-cli", "board", "list"], capture_output=True, text=True)
        ports: list[list[str]] = []

        for line in out.stdout.split('\n')[1:]:
            if line:
                line = list(map(lambda n: n.strip(), line.split()))
                if 'Unknown' not in line:
                    ports.append(line)

        return ports[0][0]

    def send_instruction(self, instr: Instruction):
        """
        Sends instruction to arduino board through serial interface.
        """

        # Arduino sends an ok message when its ready to receive an instruction.
        # Wait for ok message
        p = self.serial.read()

        # Then send instruction
        if p:
            print(f'Enviando {INSTRUCTION_MAP[instr]}::{p}')
            self.serial.write(bytes(
                INSTRUCTION_MAP[instr],
                'utf-8'
            ))

class SimInterface:
    """
    Class responsible for interacting with the simulation's shared memory.
    There are a bunch of rules that I'll write later...
    """

    # sync constants
    AI_FRAME_REQUEST = 1
    SIM_FRAME_WAIT = 2
    AI_MOVE_INSTRUCTION = 3
    SIM_AKNOWLEDGE_INSTRUCTION = 4

    # memory layout
    SYNC_SIZE = 1
    INCOMING_IMAGE_RES = (512, 512)
    IMAGE_SIZE = 786_432
    SENSOR_COUNT_SIZE = 1
    MAX_PROXIMITY_SENSOR_COUNT = 5

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

    def get_data(self) -> tuple[MatLike, list[int]]:
        """
        Requests a frame and does some processing for the image
        to be usable by AI.
        """

        self.send_instruction(Instruction.RequestScan)
        self.wait_confirmation()

        # sync + sensor count + sensor reads
        img_offset = 1 + 1 + 5 + 1
        raw_img = self.mem[img_offset : SimInterface.IMAGE_SIZE + 1 + img_offset]
        im = Image.frombytes('RGB', SimInterface.INCOMING_IMAGE_RES, raw_img) # pyright: ignore[reportUnknownMemberType]

        # for some reason blue and red channels are swapped?
        # r, g, b = im.split()
        # return np.array(Image.merge('RGB', (b, g, r)))

        original = cv2.cvtColor(np.array(im), cv2.COLOR_BGR2RGB)
        proximities = list(map(int, self.mem[2 : 2 + SimInterface.MAX_PROXIMITY_SENSOR_COUNT]))

        return (original, proximities)

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

        if instr == Instruction.RequestScan:
            self.set_sync_byte(SimInterface.AI_FRAME_REQUEST)
        elif instr == Instruction.MoveBack:
            raise Exception(f'unsoported instruction {instr}')
        else:
            self.set_sync_byte(SimInterface.AI_MOVE_INSTRUCTION)
            self.set_mov_instruction(ord(INSTRUCTION_MAP[instr]))
            self.wait_confirmation()
