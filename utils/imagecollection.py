# My laptop can't really handle notebooks, so I'm doing it here.

import cv2, uuid, os, time, sys

IMAGES_PATH = ''
IMAGES_PATH = os.path.join('Users', 'boristoteles', 'Documents', 'tmr23', 'arenito', 'images')
SCALE = 1 / 2

# if 'path' in sys.argv:
#     if not os.path.exists(IMAGES_PATH):
#         if os.name == 'posix':
#             os.system(f'mkdir -p {IMAGES_PATH}')
#         if os.name == 'nt':
#             os.system(f'mkdir {IMAGES_PATH}')

#     for label in labels:
#         path = os.path.join(IMAGES_PATH, label)
#         if not os.path.exists(path):
#             os.system(f'mkdir {path}')

if __name__ == '__main__':
    cap = cv2.VideoCapture(0)

    while True:
        ret, frame = cap.read()

        size = (int(frame.shape[1] * SCALE), int(frame.shape[0] * SCALE))
        frame = cv2.resize(frame, size)

        cv2.imshow('frame', frame)

        if cv2.waitKey(1) & 0xFF == ord(' '):
            imgname = '{}.jpg'.format(str(uuid.uuid1()))
            cv2.imwrite(imgname, frame)
            print(f'guardando {imgname}')

        if cv2.waitKey(1) & 0xFF == ord('q'):
            break

    cap.release()
    cv2.destroyAllWindows()
