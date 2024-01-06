import os, argparse
from PIL import Image

def resize(
    path: str,
    extension: str = 'png',
    scale: int | None = 1,
    res: tuple[int] | None = None,
    replace: bool = True
):
    img = Image.open(path)
    img = img.convert('RGB')

    filename = path[path.rfind('/') + 1 : path.rfind('.')]

    size = res if res else (img.size[0] * scale, img.size[1] * scale)
    outname = filename if replace else 'c-'+ filename
    outpath = path[:path.find('/') + 1] + outname + '.' + extension

    img.thumbnail(size)
    img.save(outpath)

if __name__ == '__main__':
    parser = argparse.ArgumentParser(
        prog='Image scaler!',
        usage='scaler.py [OPTIONS] folder(s)',
        description='Utility to scale images.'
    )
    parser.add_argument(
        '-s', '--scale',
        default=None,
        nargs='?',
    )
    parser.add_argument(
        '--width',
        default=None,
        nargs='?',
    )
    parser.add_argument(
        '--height',
        default=None,
        nargs='?',
    )
    parser.add_argument(
        'files',
        nargs='*'
    )
    parser.add_argument(
        '--no-replace',
        action='store_true'
    )
    args = parser.parse_args()

    if not args.width and not args.height and not args.scale:
        print('Must specify either width and height or scale.')
        exit(1)

    if not(args.height and args.width) and not (args.height == None and args.width == None):
        print('Must specify both width and height.')
        exit(1)

    if args.scale and args.width:
        print('Must specify only resolution or scale.')
        exit(1)

    if not args.scale and not args.width and not args.height:
        print('Must specify some way of resizing the image!')
        exit(1)

    res = (float(args.height), float(args.width)) if args.height else None
    scale = float(args.scale) if args.scale else None

    for file in args.files:
        print(f'Resizing {file}')
        resize(
            file,
            scale=scale,
            res=res,
            replace=not args.no_replace
        )
