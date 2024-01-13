# pyright: strict

import cv2
from cv2.typing import MatLike
import argparse
from arenito_com import *
from arenito_vision import *

# Cuenta cuantas instrucciones lleva buscando latas
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
        com.send_instruction(Instruction.MOVE_RIGHT)
    elif vis.center_x_min >= x:
        com.send_instruction(Instruction.MOVE_LEFT)
    else: # está centrado, avanza
        com.send_instruction(Instruction.MOVE_FORWARD)

    lr_count = 0

def send_roam_instruction(com: ArenitoComms, vis: ArenitoVision, hsv_frame: MatLike):
    """
    Function strictly responsible for determining movement
    when no can detections are made.
    """

    global lr_count

    if vis.reachable(hsv_frame, vis.r_dot):           # si puede, avanza
        com.send_instruction(Instruction.MOVE_FORWARD)
    else:                                             # si no, gira
        com.send_instruction(Instruction.MOVE_RIGHT)

    lr_count += 1

    if lr_count == LR_COUNT_MAX:
        com.send_instruction(Instruction.MOVE_LONG_RIGHT)
        lr_count = 0

def get_image(com: ArenitoComms, vis: ArenitoVision) -> MatLike:
    """
    Gets an image from ArenitoComms and resizes it.
    """

    return vis.resize(com.get_image())

def main(com: ArenitoComms, vis: ArenitoVision, no_move: bool):
    """
    Main loop.
    """

    while True:
        frame = get_image(com, vis)

        if cv2.waitKey(1) == 27:
            break

        blurred = vis.blur(frame)

        detections = vis.find_cans(blurred)
        pov = frame.copy()
        vis.add_markings(pov, detections)

        cv2.imshow('arenito pov', pov)

        if no_move:
            continue

        if detections:
            det = detections[0]
            send_move_instruction(com, vis, det.center)
        else:
            hsv_frame = cv2.cvtColor(frame, cv2.COLOR_BGR2HSV)
            send_roam_instruction(com, vis, hsv_frame)

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('port', nargs='?', type=str, default=None)
    parser.add_argument('baudrate', nargs='?', type=int, default=115200)
    parser.add_argument('timeout', nargs='?', type=float, default=0.5)

    parser.add_argument('flink', nargs='?', type=str, default='../sim/shmem_arenito')
    parser.add_argument('--sim', '-s', action=argparse.BooleanOptionalAction, default=False)
    parser.add_argument('--no_move', '-n', action=argparse.BooleanOptionalAction, default=False)

    args = parser.parse_args()
    mode = AIMode.Simulation if args.sim else AIMode.Real
    com = ArenitoComms(mode, args)
    vis = ArenitoVision(mode)

    try:
        main(com, vis, args.no_move)
    except Exception as e:
        print(e)

    if com.sim_interface:
        com.sim_interface.close()
