# pyright: strict

from arenito_com_consts import *
from serial import Serial
import subprocess

class SerialInterface:
    def __init__(self, port: str, baudrate: int):
        self.connect(port, baudrate)

    def connect(self, port: str, baudrate: int):
        """
        Establishes serial communication.
        """

        # if port is None: port = SerialInterface.find_port()
        self.serial = Serial(port=port, baudrate=baudrate)
        # wait for arduino to be ready
        self.wait_confirmation()

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

    def get_prox_sensors(self) -> list[int]:
        """
        Returns proximity sensor reads.
        """

        self.serial.write(INSTRUCTION_MAP[Instruction.RequestProxSensor].encode('utf-8'))
        sensor_reads = self.serial.readline().decode('utf-8').strip()[:-1]
        self.wait_confirmation()

        return list(map(int, sensor_reads.split(',')))

    def dump_cans(self):
        """
        Dumps cans.
        """

        self.send_instruction(Instruction.DumpCans)
        self.wait_confirmation()

    def send_instruction(self, instr: Instruction):
        """
        Sends instruction to arduino board through serial interface.
        """

        self.serial.write(INSTRUCTION_MAP[instr].encode('utf-8'))
        self.wait_confirmation()

    def wait_confirmation(self):
        """
        Waits for confirmation from Arduino.
        """

        self.serial.readline()
