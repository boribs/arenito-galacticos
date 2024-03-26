# pyright: strict

import cv2
from argparse import Namespace
from cv2.typing import MatLike
from dataclasses import dataclass
from enum import Enum, auto
from arenito_com import *
from arenito_vision import *
from time import time

@dataclass
class ScanResult:
    original: MatLike
    blurred: MatLike
    detections: list[Detection]
    proximities: list[int]

class ArenitoState(Enum):
    LookingForCans = auto()
    GrabbingCan = auto()
    # DumpingCans = auto()

class ArenitoAI:
    """
    AI class, the brains of it all.
    """

    def __init__(self, args: Namespace):
        mode = AIMode.Simulation if args.sim else AIMode.Real
        self.args = args
        self.com = ArenitoComms(mode, args)
        self.vis = ArenitoVision(mode, args)

        self.state = ArenitoState.LookingForCans

        # Can tracking stuff
        self.timer: float | None = None
        self.can_counter = 0
        self.can_in_critical_region = False

    def scan(self) -> ScanResult:
        """
        Gets data from every sensor.
        """

        original, proximities = self.com.get_data()
        blurred = self.vis.blur(original)
        detections = self.vis.find_cans(blurred)

        return ScanResult(
            original=original,
            blurred=blurred,
            detections=detections,
            proximities=proximities
        )

    def get_state(self, scan_results: ScanResult):
        """
        Determines Arenito's current state based on sensor scan results.
        """

        if scan_results.detections:
            self.state = ArenitoState.GrabbingCan
        else:
            self.state = ArenitoState.LookingForCans

    def start_timer(self):
        """
        Starts the timer.
        """

        if not self.timer:
            self.timer = time()

    def get_timer_elapsed(self) -> float:
        """
        Returns elapsed time since the timer was started.
        """

        current_time = time() - self.timer if self.timer else 0
        return current_time

    def clear_timer(self):
        """
        Clears timer.
        """

        self.timer = None

    def main(self):
        """
        Main loop.
        """

        while True:
            if cv2.waitKey(1) == 27:
                break

            scan_results = self.scan()

            self.get_state(scan_results)

            state_str = self.state.name
            if self.timer and self.state == ArenitoState.LookingForCans:
                state_str += ': {0:.2f}'.format(time() - self.timer)

            self.vis.add_markings(
                scan_results.original,
                scan_results.detections,
                state_str,
                self.can_counter,
                self.can_in_critical_region,
            )
            cv2.imshow('arenito pov', scan_results.original)
            #   cv2.imshow('arenito pov - blurred', blurred)

            if self.args.no_move:
                continue

            if self.vis.can_in_critical_region(scan_results.detections):
                self.can_in_critical_region = True
            elif self.can_in_critical_region:
                self.can_in_critical_region = False
                self.can_counter += 1

            if self.state == ArenitoState.GrabbingCan:
                self.get_can(scan_results)
                self.clear_timer()
            elif self.state == ArenitoState.LookingForCans:
                self.start_timer()
                self.search_cans(scan_results)

    def align(self, scan_results: ScanResult):
        """
        Aligns with closest (first) detection.
        """

        while scan_results.detections:
            x = scan_results.detections[0].center.x

            if self.vis.center_x_max <= x:
                self.com.send_instruction(Instruction.MoveRight)
            elif self.vis.center_x_min >= x:
                self.com.send_instruction(Instruction.MoveLeft)
            else:
                break

            scan_results = self.scan()

    def get_can(self, scan_results: ScanResult):
        """
        Can-getter routine.
        """

        self.align(scan_results)
        self.com.send_instruction(Instruction.MoveForward)

    def search_cans(self, scan_results: ScanResult):
        """
        Can-search routine.
        """

        MAX_SEARCH_SECONDS = 30

        hsv = cv2.cvtColor(scan_results.blurred, cv2.COLOR_BGR2HSV)

        if self.vis.reachable(hsv, self.vis.r_dot):
            self.com.send_instruction(Instruction.MoveForward)

        if self.get_timer_elapsed() > MAX_SEARCH_SECONDS:
            self.com.send_instruction(Instruction.MoveLongRight)
            self.clear_timer()
        else:
            self.com.send_instruction(Instruction.MoveRight)
