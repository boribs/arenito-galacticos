import subprocess, cv2
import numpy as np
from cv2.typing import MatLike
from enum import Enum, auto
from serial import Serial
from multiprocessing.shared_memory import SharedMemory
from multiprocessing import resource_tracker
from PIL import Image

class Instruction(Enum):
    FORWARD = auto()
    LEFT = auto()
    RIGHT = auto()
    BACK = auto()
    LONG_RIGHT = auto()
    SCREENSHOT = auto() # I don't like your name

INSTRUCTION_MAP = {
    Instruction.FORWARD: 'a',
    Instruction.LEFT: 'i',
    Instruction.RIGHT: 'd',
    Instruction.BACK: 'r',
    Instruction.LONG_RIGHT: 'l',
    Instruction.SCREENSHOT: 'ss',
}

class ArenitoComms:
    """
    Interface between Arenito's AI and other devices.
    This class is responisble for capturing (raw) images from the camera
    and communicating with the Arduino board.
    Also gets information from and to the simulation.
    """

    def __init__(self):
        self.serial: SerialInterface = None
        self.video_capture: cv2.VideoCapture = None
        self.sim_interface: SimInterface = None

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

    def connect_simulation(self, flink: str):
        """
        Attaches to simulation's shared memory.
        """

        self.sim_interface = SimInterface(flink)

    def get_image(self) -> MatLike:
        """
        Gets the image from the camera.
        """

        if self.video_capture:
            ok, frame = self.video_capture.read()
            if not ok:
                raise Exception('Couldn\'t get frame.')

            return frame
        else:
            return self.sim_interface.get_image()

    def send_instruction(self, instr: Instruction):
        """
        Sends instruction to arduino board through serial interface.
        """

        if self.serial:
            self.serial.send_instruction(instr)
        else:
            self.sim_interface.send_instruction(instr)

class SerialInterface:
    def __init__(self, port: str | None, baudrate: int, timeout: float = 0.0):
        self.serial: Serial = None
        self.connect(port, baudrate, timeout)

    def connect(self, port: str | None, baudrate: int, timeout: float):
        """
        Establishes serial communication.
        """

        if port is None: port = SerialInterface.find_port()
        self.serial = Serial(port=port, baudrate=baudrate, timeout=timeout)

    def find_port() -> str:
        """
        Finds out where the Arduino borad is connected. Requires `arduino-cli`.
        """

        out = subprocess.run(["arduino-cli", "board", "list"], capture_output=True, text=True)
        ports = []
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

    # memory footprint
    IMAGE_SIZE = 3_145_728 # image size (bytes)

    def __init__(self, flink: str):
        SimInterface.remove_shm_from_resource_tracker() # python 3.12 or less

        self.mem: SharedMemory = None
        self.attach(flink)

    def attach(self, flink: str):
        """
        """

        with open(flink, 'r') as f:
            osid = f.read()[1:]

        self.mem = SharedMemory(create=False, name=osid)

    def close(self):
        """
        """

        self.mem.close()

    def get_sync_byte(self) -> int:
        """
        """

        return self.mem.buf[0]

    def set_sync_byte(self, val: int):
        """
        """

        self.mem.buf[0] = val

    def set_mov_instruction(self, val: int):
        """
        """

        self.mem.buf[1] = val

    def get_image(self) -> memoryview:
        """
        """

        self.send_instruction(Instruction.SCREENSHOT)

        raw_img = self.mem.buf[1 : SimInterface.IMAGE_SIZE + 1]
        im = Image.frombytes('RGB', (1024, 1024), raw_img)

        # for some reason blue and red channels are swapped?
        r, g, b = im.split()
        return np.array(Image.merge('RGB', (b, g, r)))

        # this also works but seems to cause some weird side effects on
        # the spatial awareness side on the simulation...
        # it is faster, though
        # return cv2.cvtColor(np.array(im), cv2.COLOR_BGR2RGB)

    def wait_confirmation(self):
        """
        """

        while self.mem.buf[0] != SimInterface.SIM_AKNOWLEDGE_INSTRUCTION:
            pass

    def send_instruction(self, instr: Instruction):
        """
        """

        if instr == Instruction.SCREENSHOT:
            self.set_sync_byte(SimInterface.AI_FRAME_REQUEST)

        elif instr in (Instruction.FORWARD, Instruction.LEFT, Instruction.RIGHT):
            self.set_sync_byte(SimInterface.AI_MOVE_INSTRUCTION)
            self.set_mov_instruction(ord(INSTRUCTION_MAP[instr]))
            self.wait_confirmation()

        elif instr == Instruction.LONG_RIGHT: # long right = right
            self.set_sync_byte(SimInterface.AI_MOVE_INSTRUCTION)
            self.set_mov_instruction(ord(INSTRUCTION_MAP[Instruction.RIGHT]))
            self.wait_confirmation()

        else:
            raise Exception(f'unsoported instruction {instr}')


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
