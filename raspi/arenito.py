# pyright: strict

import argparse
from arenito_ai import ArenitoAI

# Cuenta cuantas instrucciones lleva buscando latas

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('port', nargs='?', type=str, default=None)
    parser.add_argument('baudrate', nargs='?', type=int, default=115200)
    parser.add_argument('timeout', nargs='?', type=float, default=0.5)

    parser.add_argument('filename', nargs='?', type=str, default='../sim/file.mmap')
    parser.add_argument('--mode', '-m', type=str, default='s')
    parser.add_argument('--no_move', '-n', action=argparse.BooleanOptionalAction, default=False)
    parser.add_argument('--algorithm', '-a', type=str, default='min-rect')

    args = parser.parse_args()
    ArenitoAI(args).main()
