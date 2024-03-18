# pyright: strict

import argparse
from arenito_com import AIMode, ArenitoComms
from arenito_vision import ArenitoVision
from arenito_ai import main

# Cuenta cuantas instrucciones lleva buscando latas

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('port', nargs='?', type=str, default=None)
    parser.add_argument('baudrate', nargs='?', type=int, default=115200)
    parser.add_argument('timeout', nargs='?', type=float, default=0.5)

    parser.add_argument('filename', nargs='?', type=str, default='../sim/file.mmap')
    parser.add_argument('--sim', '-s', action=argparse.BooleanOptionalAction, default=False)
    parser.add_argument('--no_move', '-n', action=argparse.BooleanOptionalAction, default=False)
    parser.add_argument('--algorithm', '-a', type=str, default='min-rect')

    args = parser.parse_args()
    mode = AIMode.Simulation if args.sim else AIMode.Real
    com = ArenitoComms(mode, args)
    vis = ArenitoVision(mode, args)

    try:
        main(com, vis, args.no_move)
    except Exception as e:
        print(e)

    if com.sim_interface:
        com.sim_interface.close()
