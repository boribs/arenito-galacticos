import cv2, sys

cam = cv2.VideoCapture(int(sys.argv[1]))

while True:
    if cv2.waitKey(1) == ord('q'):
        break

    ok, frame = cam.read()

    cv2.imshow('asf', frame)
