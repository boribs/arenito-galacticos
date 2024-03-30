# pyright: strict

from arenito_com_consts import *
from cv2.typing import MatLike
import cv2

class ArenitoCameras:
    """
    Interface between video input and ArenitoComms.
    """

    def __init__(self):
        self.cameras: list[cv2.VideoCapture] = []

    def init_video_capture(self, index: int) -> cv2.VideoCapture:
        """
        Initializes video capture device.
        """

        return cv2.VideoCapture(index)

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

    def __init__(self):
        # initialize cameras
        # cam0 -> front
        # cam1 -> rear

        self.cameras = ArenitoCameras()

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
