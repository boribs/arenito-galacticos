# pyright: strict

import cv2
from argparse import Namespace
from cv2.typing import MatLike
from dataclasses import dataclass
from enum import Enum, auto
from arenito_com import *
from arenito_vision import *

@dataclass
class ScanResult:
    original: MatLike
    blurred: MatLike
    detections: list[Detection]
    # proximity sensors results

class ArenitoState(Enum):
    LookingForCans = auto()
    GrabbingCan = auto()
    # AligningWithDeposit = auto()
    # ThrowingCans = auto()

class ArenitoAI:
    def __init__(self, args: Namespace):
        mode = AIMode.Simulation if args.sim else AIMode.Real
        self.args = args
        self.com = ArenitoComms(mode, args)
        self.vis = ArenitoVision(mode, args)
        self.state = ArenitoState.LookingForCans

    def scan(self) -> ScanResult:
        original = self.com.get_image()
        blurred = self.vis.blur(original)
        detections = self.vis.find_cans(blurred)

        return ScanResult(
            original=original,
            blurred=blurred,
            detections=detections
        )

    def set_state(self, scan_results: ScanResult):
        pass

    def main(self):
        """
        Main loop.
        """

        while True:
            if cv2.waitKey(1) == 27:
                break

            scan_results = self.scan()

            self.get_state(scan_results)
            self.vis.add_markings(
                scan_results.original,
                scan_results.detections,
                self.state.name
            )
            cv2.imshow('arenito pov', scan_results.original)
            #   cv2.imshow('arenito pov - blurred', blurred)

            if self.args.no_move:
                continue

            # determine state:

            # if detections:
            #     det = detections[0]
            #     self.align(det.center)
            #     self.com.send_instruction(Instruction.MoveForward)
            # else:
            #     hsv_frame = cv2.cvtColor(frame, cv2.COLOR_BGR2HSV)
            #     send_roam_instruction(self.com, self.vis, hsv_frame)

    def align(self):
        # TODO: make this a loop
        # get image
        # get detections
        # align

        if self.vis.center_x_max <= x:
            self.com.send_instruction(Instruction.MoveRight)
        elif self.vis.center_x_min >= x:
            self.com.send_instruction(Instruction.MoveLeft)

def send_move_instruction(com: ArenitoComms, vis: ArenitoVision, det: Point):
    """
    Sends a move to left, right or forward instruction
    to the Arduino board, depending on the detection's position.
    """

    x, _ = det

    if vis.center_x_max <= x:
        com.send_instruction(Instruction.MoveRight)
    elif vis.center_x_min >= x:
        com.send_instruction(Instruction.MoveLeft)
    else: # está centrado, avanza
        com.send_instruction(Instruction.MoveForward)

def send_roam_instruction(com: ArenitoComms, vis: ArenitoVision, hsv_frame: MatLike):
    """
    Function strictly responsible for determining movement
    when no can detections are made.
    """

    if vis.reachable(hsv_frame, vis.r_dot):           # si puede, avanza
        com.send_instruction(Instruction.MoveForward)
    else:                                             # si no, gira
        com.send_instruction(Instruction.MoveRight)

    # lr_count += 1

    # if lr_count == LR_COUNT_MAX:
    #     com.send_instruction(Instruction.MoveLongRight)
    #     lr_count = 0

