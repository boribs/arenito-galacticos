import subprocess
from cv2 import VideoCapture
from cv2.typing import MatLike
from enum import Enum, auto
from serial import Serial

class Instruction(Enum):
    FORWARD = auto()
    LEFT = auto()
    RIGHT = auto()
    BACK = auto()
    LONG_RIGHT = auto()
    SCREENSHOT = auto()

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
    """

    def __init__(self):
        self.serial: serial.Serial = None
        self.video_capture: VideoCapture = None
        self.sim_interface: SimInterface = None

    def init_video(self, device_index: int = 0):
        """
        Initializes the capture device.
        """

        self.video_capture = VideoCapture(device_index)

    def connect_serial(self, port: str | None, baudrate: int, timeout: float = 0.0):
        """
        Establishes serial communication.
        """

        self.serial = SerialInterface(port, baudrate, timeout)

    def get_image(self) -> MatLike:
        """
        Gets the image from the camera.
        """

        ok, frame = self.video_capture.read()
        if not ok:
            raise Exception('Couldn\'t get frame.')

        return frame

    def send_instruction(self, instr: Instruction):
        """
        Sends instruction to arduino board through serial interface.
        """

        self.serial.send_instruction(instr)

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

