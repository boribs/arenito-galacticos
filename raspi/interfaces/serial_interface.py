# pyright: strict

from arenito_com_consts import *
from serial import Serial
import subprocess
from argparse import Namespace

class SerialInterface:
    def __init__(self, args: Namespace):
        self.connect(args.port, args.baudrate)

    def connect(self, port: str, baudrate: int):
        """
        Establishes serial communication.
        """

        # if port is None: port = SerialInterface.find_port()
        self.serial = Serial(port=port, baudrate=baudrate)

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
