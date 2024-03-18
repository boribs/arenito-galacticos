# pyright: strict

import cv2
from argparse import Namespace
from cv2.typing import MatLike
from arenito_com import *
from arenito_vision import *

class ArenitoAI:
    def __init__(self, args: Namespace):
        mode = AIMode.Simulation if args.sim else AIMode.Real
        self.args = args
        self.com = ArenitoComms(mode, args)
        self.vis = ArenitoVision(mode, args)

    def main(self):
        """
        Main loop.
        """

        while True:
            frame = self.com.get_image()

            if cv2.waitKey(1) == 27:
                break

            original = frame.copy()
            blurred = self.vis.blur(frame)

            detections = self.vis.find_cans(blurred)
            self.vis.add_markings(original, detections)

            cv2.imshow('arenito pov', original)
            cv2.imshow('arenito pov - blurred', blurred)

            if self.args.no_move:
                continue

            if detections:
                det = detections[0]
                send_move_instruction(self.com, self.vis, det.center)
            else:
                hsv_frame = cv2.cvtColor(frame, cv2.COLOR_BGR2HSV)
                send_roam_instruction(self.com, self.vis, hsv_frame)

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

