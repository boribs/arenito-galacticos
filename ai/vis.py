from arenito_logger import ArenitoLogger
from arenito_vision import ArenitoVision, WHITE
from arenito_com import AIMode
import argparse, cv2

parser = argparse.ArgumentParser()
parser.add_argument('image', type=str)
parser.add_argument('--save_images', '-s', type=str, default='')
parser.add_argument('--print_log', '-l', default=True)
parser.add_argument('--algorithm', '-a', type=str, default='min-rect')
args = parser.parse_args()

vis = ArenitoVision(AIMode.Jetson, args, ArenitoLogger(args))
image = cv2.imread(args.image)
detections = vis.find_cans(image)
print(detections)
for det in detections:
    cv2.circle(image, det.center, 10, WHITE, 10)
cv2.imwrite('img/adetections.jpg', image)

print(
    'Dump:',
    vis.detect_dumping_zone(image, True)
)
