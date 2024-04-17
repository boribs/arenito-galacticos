# pyright: strict

import argparse
from arenito_ai import ArenitoAI

# Cuenta cuantas instrucciones lleva buscando latas

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('port', nargs='?', type=str, default=None)
    parser.add_argument('baudrate', nargs='?', type=int, default=115200)

    parser.add_argument('filename', nargs='?', type=str, default='../sim/file.mmap')
    parser.add_argument('--mode', '-m', type=str, default='s')
    parser.add_argument('--no_move', '-n', action=argparse.BooleanOptionalAction, default=False)
    parser.add_argument('--algorithm', '-a', type=str, default='min-rect')
    parser.add_argument('--headless', '-H', action=argparse.BooleanOptionalAction, default=False)
    parser.add_argument('--save_images', '-s', type=str, default='')
    parser.add_argument('--print_log', '-l', action=argparse.BooleanOptionalAction, default=False)
    parser.add_argument('--no_backdoor_extension', '-B', action=argparse.BooleanOptionalAction, default=False)
    parser.add_argument('--no_lcd', '-L', action=argparse.BooleanOptionalAction, default=False)

    args = parser.parse_args()
    arenito_ai = ArenitoAI(args)

    try:
        arenito_ai.main()
    except KeyboardInterrupt:
        pass
    except Exception as e:
        print(f"{type(e).__name__} at line {e.__traceback__.tb_lineno} of {__file__}: {e}") # pyright: ignore[reportOptionalMemberAccess]
    finally:
        arenito_ai.stop_all()
        arenito_ai.print_stats()
