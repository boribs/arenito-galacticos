# My laptop can't really handle notebooks, so I'm doing it here.

import cv2, uuid, argparse, time

def main(scale: float):
    cap = cv2.VideoCapture(0)

    while cap.isOpened():
        ok, frame = cap.read()

        if not ok:
            print('Error comunicating with camera.')
            exit(1)

        size = (int(frame.shape[1] * scale), int(frame.shape[0] * scale))
        frame = cv2.resize(frame, size)

        cv2.imshow('frame', frame)

        if cv2.waitKey(1) & 0xFF == ord(' '):
            imgname = '{}.jpg'.format(str(uuid.uuid1()))
            cv2.imwrite(imgname, frame)
            print(f'guardando {imgname}')

        if cv2.waitKey(1) & 0xFF == 27:
            break

    cap.release()
    cv2.destroyAllWindows()

def main_timed(timer: int, scale: float): # timed?
    cap = cv2.VideoCapture(0)

    while cap.isOpened():
        ok, frame = cap.read()

        if not ok:
            print('Error comunicating with camera.')
            exit(1)

        size = (int(frame.shape[1] * scale), int(frame.shape[0] * scale))
        frame = cv2.resize(frame, size)

        cv2.imshow('frame', frame)

        imgname = '{}.jpg'.format(str(uuid.uuid1()))
        cv2.imwrite(imgname, frame)
        print(f'guardando {imgname}')

        time.sleep(timer)

        if cv2.waitKey(1) & 0xFF == 27:
            break

    cap.release()
    cv2.destroyAllWindows()


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument(
        '--timer',
        type=float,
        default=0,
    )
    parser.add_argument(
        '--scale',
        type=float,
        default=0,
    )

    args = parser.parse_args()

    if args.timer == 0:
        main(args.scale)
    else:
        main_timed(args.timer, args.scale)
