# pyright: strict

import cv2
from argparse import Namespace
from cv2.typing import MatLike
from arenito_com_conts import *
from interfaces.sim_interface import SimInterface


class ArenitoComms:
    """
    Interface between Arenito's AI and other devices.
    This class is responisble for capturing (raw) images from the camera
    and communicating with the Arduino board.
    Also gets information from and to the simulation.
    """

    def __init__(self, mode: AIMode, args: Namespace):
        # self.serial: SerialInterface | None = None
        # self.video_capture: cv2.VideoCapture | None = None
        self.sim_interface: SimInterface | None = None

        if mode == AIMode.Simulation:
            self.connect_simulation(args.filename)
        # elif mode == AIMode.Real:
            # self.connect_serial(args.port, args.baudrate, args.timeout)
            # self.init_video()
        else:
            raise Exception(f'Unsupported mode {mode}.')

    # def init_video(self, device_index: int = 0):
    #     """
    #     Initializes the capture device.
    #     """

    #     self.video_capture = cv2.VideoCapture(device_index)

    # def connect_serial(self, port: str | None, baudrate: int, timeout: float = 0.0):
    #     """
    #     Establishes serial communication.
    #     """

    #     self.serial = SerialInterface(port, baudrate, timeout)

    def connect_simulation(self, filename: str):
        """
        Attaches to simulation's shared memory.
        """

        self.sim_interface = SimInterface(filename)

    def get_front_frame(self) -> MatLike:
        """
        Gets the image from the front camera.
        """

        # if self.video_capture:
        #     raise Exception('Real camera input not supported')
            # ok, frame = self.video_capture.read()
            # if not ok:
            #     raise Exception('Couldn\'t get frame.')

            # return frame
        # else:
        return self.sim_interface.get_front_frame() # pyright: ignore[reportOptionalMemberAccess]

    def get_rear_frame(self) -> MatLike:
        """
        Gets the image from the rear camera.
        """

        # if self.video_capture:
        #     raise Exception('Real camera input not supported')
            # ok, frame = self.video_capture.read()
            # if not ok:
            #     raise Exception('Couldn\'t get frame.')

            # return frame
        # else:
        return self.sim_interface.get_rear_frame() # pyright: ignore[reportOptionalMemberAccess]

    def get_prox_sensors(self) -> list[int]:
        """
        Returns proximity sensor reads. Only for Sim.
        """

        # if self.serial:
        #     raise Exception('Proximity sensors not implemented for serial interface')

        return self.sim_interface.get_proximity_sensor_reads() # pyright: ignore[reportOptionalMemberAccess]

    def send_instruction(self, instr: Instruction):
        """
        Sends instruction to arduino board through serial interface.
        """

        # if self.serial:
        #     self.serial.send_instruction(instr)
        if self.sim_interface:
            self.sim_interface.send_instruction(instr)

    def dump_cans(self, ammount: int):
        """
        Dumps cans.
        """

        # if self.serial:
        #     raise Exception('Instruction not implemented for Serial interface')
        if self.sim_interface:
            self.sim_interface.dump_cans(ammount)
