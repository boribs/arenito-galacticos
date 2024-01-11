import cv2
import numpy as np

img = cv2.imread('img.png')

cv2.imshow('Binary image', img)
cv2.waitKey(0)
cv2.destroyAllWindows()

mask = cv2.cvtColor(img, cv2.COLOR_RGB2GRAY)

# this also happens to filter black!
ret, thresh = cv2.threshold(mask, 50, 255, cv2.RETR_EXTERNAL)

cv2.imshow('Binary image', thresh)
cv2.waitKey(0)
cv2.destroyAllWindows()

contours, hierarchy = cv2.findContours(
    image=thresh,
    mode=cv2.RETR_TREE,
    method=cv2.CHAIN_APPROX_NONE
)

# first contour is the whole image?
contours = contours[1:]

image_copy = img.copy()
for cnt in contours:
    rect = cv2.minAreaRect(cnt)

    w, h = rect[1]
    if w * h > 200:
        box = np.int0(cv2.boxPoints(rect))
        cv2.drawContours(image_copy, [box], -1, (0, 255, 0), 1, cv2.LINE_AA)

cv2.imshow('None approximation', image_copy)
cv2.waitKey(0)
cv2.destroyAllWindows()

# rect = cv2.minAreaRect(cnts)
