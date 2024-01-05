# pyright: strict

import cv2
from cv2.typing import MatLike
import argparse
from arenito_com import *
from arenito_vision import *

RES_X = 640
RES_Y = 380

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

    if vis.centro_x_max <= x:
        com.send_instruction(Instruction.MOVE_LEFT)
    elif vis.centro_x_min >= x:
        com.send_instruction(Instruction.MOVE_RIGHT)
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

def get_image(com: ArenitoComms) -> MatLike:
    """
    Gets an image from ArenitoComms and resizes it to be RES_X x RES_Y.
    """

    return cv2.resize(com.get_image(), (RES_X, RES_Y), interpolation=cv2.INTER_LINEAR)

def main(com: ArenitoComms, vis: ArenitoVision):
    """
    Main loop.

    TODO: Make every step more explicit.
    """

    while True:
        frame = get_image(com)

        if cv2.waitKey(1) == 27:
            break

        det_img, detections = vis.find_blobs(frame)
        vis.add_markings(det_img)

        cv2.imshow('asdf', det_img)

        if detections:
            det = detections[0]
            send_move_instruction(com, vis, det)
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

    com = ArenitoComms()
    vis = ArenitoVision(RES_X, RES_Y, int(RES_X * 0.2))
    args = parser.parse_args()

    if args.sim:
        com.connect_simulation(args.flink)
    else:
        com.connect_serial(args.port, args.baudrate, args.timeout)
        com.init_video()

    try:
        main(com, vis)
    except Exception as e:
        print(e)

    if com.sim_interface:
        com.sim_interface.close()
