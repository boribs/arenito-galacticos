# pyright: strict

from argparse import Namespace
from cv2.typing import MatLike
from arenito_com_consts import *
from interfaces.sim_interface import SimInterface
from interfaces.jetson_interface import JetsonInterface
import time

# TODO: Benchmark as arg

class ArenitoComms:
    """
    Interface between Arenito's AI and other devices.
    This class is responisble for capturing (raw) images from the camera
    and communicating with the Arduino board.
    Also gets information from and to the simulation.
    """

    def __init__(self, mode: AIMode, args: Namespace):
        self.sim_interface: SimInterface | None = None
        self.jetson_interface: JetsonInterface | None = None

        if mode == AIMode.Simulation:
            self.connect_simulation(args.filename)
        elif mode == AIMode.Jetson:
            self.connect_jetson(args)
        else:
            raise Exception(f'Unsupported mode {mode}.')

    def connect_jetson(self, args: Namespace):
        """
        Initializes JetsonInterface.
        """

        self.jetson_interface = JetsonInterface(args)

    def connect_simulation(self, filename: str):
        """
        Attaches to simulation's shared memory.
        """

        self.sim_interface = SimInterface(filename)

    def get_front_frame(self) -> MatLike:
        """
        Gets the image from the front camera.
        """

        t = time.time()
        if self.jetson_interface:
            r = self.jetson_interface.get_front_frame()
            print(f'got frame in: {time.time() - t}')
            return r
        elif self.sim_interface:
            return self.sim_interface.get_front_frame()
        else:
            raise Exception('No valid interface.')

    def get_rear_frame(self) -> MatLike:
        """
        Gets the image from the rear camera.
        """

        if self.jetson_interface:
            return self.jetson_interface.get_rear_frame()
        elif self.sim_interface:
            return self.sim_interface.get_rear_frame()
        else:
            raise Exception('No valid interface.')

    def get_prox_sensors(self) -> list[int]:
        """
        Returns proximity sensor reads. Only for Sim.
        """

        t = time.time()
        if self.jetson_interface:
            r = self.jetson_interface.get_prox_sensors()
            print(f'got sensor feedback: {time.time() - t}')
            return r
        elif self.sim_interface:
            return self.sim_interface.get_prox_sensors()
        else:
            raise Exception('No valid interface.')

    def send_instruction(self, instr: Instruction):
        """
        Sends instruction to arduino board through serial interface.
        """

        t = time.time()
        if self.jetson_interface:
            self.jetson_interface.send_instruction(instr)
        elif self.sim_interface:
            self.sim_interface.send_instruction(instr)
        else:
            raise Exception('No valid interface.')

        print(f'sent instruction {instr}: {time.time() - t}')

    def dump_cans(self, ammount: int):
        """
        Dumps cans.
        """

        if self.jetson_interface:
            self.jetson_interface.dump_cans()
        elif self.sim_interface:
            self.sim_interface.dump_cans(ammount)
        else:
            raise Exception('No valid interface.')

    def lcd_show(self, msg: str, line: int):
        """
        Shows a message on the LCD display. Only for JetsonInterface.
        """

        if self.jetson_interface:
            self.jetson_interface.lcd_show(msg, line)
