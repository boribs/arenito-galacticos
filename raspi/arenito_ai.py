# pyright: strict

import cv2
from cv2.typing import MatLike
from arenito_com import *
from arenito_vision import *

lr_count = 0
LR_COUNT_MAX = 20

def send_move_instruction(com: ArenitoComms, vis: ArenitoVision, det: Point):
    """
    Sends a move to left, right or forward instruction
    to the Arduino board, depending on the detection's position.
    """

    global lr_count

    x, _ = det

    if vis.center_x_max <= x:
        com.send_instruction(Instruction.MoveRight)
    elif vis.center_x_min >= x:
        com.send_instruction(Instruction.MoveLeft)
    else: # está centrado, avanza
        com.send_instruction(Instruction.MoveForward)

    lr_count = 0

def send_roam_instruction(com: ArenitoComms, vis: ArenitoVision, hsv_frame: MatLike):
    """
    Function strictly responsible for determining movement
    when no can detections are made.
    """

    global lr_count

    if vis.reachable(hsv_frame, vis.r_dot):           # si puede, avanza
        com.send_instruction(Instruction.MoveForward)
    else:                                             # si no, gira
        com.send_instruction(Instruction.MoveRight)

    lr_count += 1

    if lr_count == LR_COUNT_MAX:
        com.send_instruction(Instruction.MoveLongRight)
        lr_count = 0

def main(com: ArenitoComms, vis: ArenitoVision, no_move: bool):
    """
    Main loop.
    """

    while True:
        frame = com.get_image()

        if cv2.waitKey(1) == 27:
            break

        original = frame.copy()
        blurred = vis.blur(frame)

        detections = vis.find_cans(blurred)
        vis.add_markings(original, detections)

        cv2.imshow('arenito pov', original)
        cv2.imshow('arenito pov - blurred', blurred)

        if no_move:
            continue

        if detections:
            det = detections[0]
            send_move_instruction(com, vis, det.center)
        else:
            hsv_frame = cv2.cvtColor(frame, cv2.COLOR_BGR2HSV)
            send_roam_instruction(com, vis, hsv_frame)
