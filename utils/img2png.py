from PIL import Image
from pathlib import PurePath
import sys

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print('must provide input images!')

    for path in sys.argv[1:]:
        print(f'converting {path}')
        p = PurePath(path)
        Image.open(path).save(p.stem + '.png')
