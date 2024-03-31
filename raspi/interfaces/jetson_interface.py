# pyright: strict

from arenito_com_consts import *
from cv2.typing import MatLike
import cv2
import Jetson.GPIO as GPIO # pyright: ignore

class ArenitoCameras:
    """
    Interface between video input and ArenitoComms.
    """

    def __init__(self):
        self.cameras: list[cv2.VideoCapture] = []

    def add_video_capture(self):
        """
        Initializes video capture device.
        """

        # must get indices in a better way...
        # https://stackoverflow.com/questions/57577445/list-available-cameras-opencv-python
        index = len(self.cameras)
        capture_device = cv2.VideoCapture(index)

        ok, _ = capture_device.read()
        if not ok or not capture_device.isOpened():
            raise Exception(f'Can\'t use capture device on index {index}')

        self.cameras.append(capture_device)

    def get_front_frame(self) -> MatLike:
        """
        Requests front camera's frame.
        """

        raise Exception('TODO')

    def get_rear_frame(self) -> MatLike:
        """
        Requests rear camera's frame.
        """

        raise Exception('TODO')

class JetsonInterface:
    """
    Sensor interaction through NVIDIA Jetson Nano.
    """

    BUTTON_CALIBRATION_PIN = 18

    def __init__(self):
        self.cameras = ArenitoCameras()
        # camera calibration
        GPIO.setmode(GPIO.BOARD) # pyright: ignore[reportUnknownMemberType]
        GPIO.setup(JetsonInterface.BUTTON_CALIBRATION_PIN, GPIO.IN) # pyright: ignore[reportUnknownMemberType]

        self.cameras.add_video_capture()

        print('añadiendo cámara trasera')
        GPIO.wait_for_edge(JetsonInterface.BUTTON_CALIBRATION_PIN, GPIO.FALLING) # pyright: ignore[reportUnknownMemberType]

        self.cameras.add_video_capture()

        cv2.imwrite('frontal.png', self.cameras.get_front_frame())
        cv2.imwrite('rear.png', self.cameras.get_rear_frame())

    def get_front_frame(self) -> MatLike:
        """
        Requests front camera's frame.
        """

        return self.cameras.get_front_frame()

    def get_rear_frame(self) -> MatLike:
        """
        Requests rear camera's frame.
        """

        return self.cameras.get_rear_frame()

    def get_prox_sensors(self) -> list[int]:
        """
        Returns proximity sensor reads.
        """

        raise Exception('TODO')

    def send_instruction(self, instr: Instruction):
        """
        Requests some instructions execution.
        """

        raise Exception('TODO')

    def dump_cans(self):
        """
        Dumps cans.
        """

        raise Exception('TODO')
