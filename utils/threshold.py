import cv2
import sys
import argparse
import numpy as np
import math
from PIL import Image

def nothing(x):
    pass

# Create a window
cv2.namedWindow('image')

# create trackbars for color change
cv2.createTrackbar('HMin', 'image', 0, 179, nothing) # Hue is from 0-179 for Opencv
cv2.createTrackbar('SMin', 'image', 0, 255, nothing)
cv2.createTrackbar('VMin', 'image', 0, 255, nothing)
cv2.createTrackbar('HMax', 'image', 0, 179, nothing)
cv2.createTrackbar('SMax', 'image', 0, 255, nothing)
cv2.createTrackbar('VMax', 'image', 0, 255, nothing)

# Set default value for MAX HSV trackbars.
cv2.setTrackbarPos('HMax', 'image', 179)
cv2.setTrackbarPos('SMax', 'image', 255)
cv2.setTrackbarPos('VMax', 'image', 255)

# Initialize to check if HSV min/max value changes
hMin = sMin = vMin = hMax = sMax = vMax = 0
phMin = psMin = pvMin = phMax = psMax = pvMax = 0

waitTime = 33

def img_matrix(path):
    w = h = math.ceil(math.sqrt(len(path)))
    total_width = total_height = 512 * w
    image = Image.new(Image.open(path[0]).mode, (total_width, total_height))

    done = False
    i = 0
    for y in range(h):
        for x in range(w):
            image.paste(Image.open(path[i]), (x * 512, y * 512))
            i += 1
            if i == len(path):
                done = True
                break
        if done:
            break

    if image.size[0] > 1000:
        image = image.resize((1000, 1000))

    return image

def image_loop(img):
    global hMin, sMin, vMin, hMax, sMax, vMax
    global phMin, psMin, pvMin, phMax, psMax, pvMax

    img = np.array(img)
    img = img[:, :, ::-1]

    while True:
        # get current positions of all trackbars
        hMin = cv2.getTrackbarPos('HMin', 'image')
        sMin = cv2.getTrackbarPos('SMin', 'image')
        vMin = cv2.getTrackbarPos('VMin', 'image')

        hMax = cv2.getTrackbarPos('HMax', 'image')
        sMax = cv2.getTrackbarPos('SMax', 'image')
        vMax = cv2.getTrackbarPos('VMax', 'image')

        # Set minimum and max HSV values to display
        lower = np.array([hMin, sMin, vMin])
        upper = np.array([hMax, sMax, vMax])

        # Create HSV Image and threshold into a range.
        hsv = cv2.cvtColor(img, cv2.COLOR_BGR2HSV)
        mask = cv2.inRange(hsv, lower, upper)
        output = cv2.bitwise_and(img, img, mask=mask)

        # Print if there is a change in HSV value
        if ((phMin != hMin) |
            (psMin != sMin) |
            (pvMin != vMin) |
            (phMax != hMax) |
            (psMax != sMax) |
            (pvMax != vMax)):
            print("(hMin = %d, sMin = %d, vMin = %d), (hMax = %d, sMax = %d, vMax = %d)" % (hMin, sMin, vMin, hMax, sMax, vMax))
            phMin = hMin
            psMin = sMin
            pvMin = vMin
            phMax = hMax
            psMax = sMax
            pvMax = vMax

        cv2.imshow('image', output)

        if cv2.waitKey(waitTime) & 0xFF == ord('q'):
            break

    cv2.destroyAllWindows()

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('path', nargs='*')
    args = parser.parse_args()

    image = img_matrix(args.path)
    image_loop(image)
