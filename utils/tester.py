import cv2, sys, time, argparse
from tflite_support.task import core
from tflite_support.task import processor
from tflite_support.task import vision

def detecta_latas(img: cv2.Mat, detector: vision.ObjectDetector) -> str:
    rgb_img = cv2.cvtColor(img, cv2.COLOR_BGR2RGB)
    input_tensor = vision.TensorImage.create_from_array(rgb_img)
    detecciones = detector.detect(input_tensor)
    imgname = f'{time.process_time()}.jpg'

    dets = []
    for det in detecciones.detections:
        bbox = det.bounding_box
        a = (bbox.origin_x, bbox.origin_y)
        b = (a[0] + bbox.width, a[1] + bbox.height)
        c = ((a[0] + b[0]) // 2, (a[1] + b[1]) // 2)
        dets.extend(c)

        cv2.rectangle(img, a, b, thickness=4, color=(255, 0, 0))
        cv2.circle(img, c, radius=5, thickness=4, color=(0, 0, 255))

    cv2.imwrite('det' + imgname, img) # Imagen con anotaciones de detecciones

    return '{' + ','.join(map(str, dets)) + ',}'

def main(model: str, image: str):
    base_options = core.BaseOptions(
        file_name=model, use_coral=False, num_threads=4)
    detection_options = processor.DetectionOptions(
        max_results=10, score_threshold=0.6)
    options = vision.ObjectDetectorOptions(
        base_options=base_options, detection_options=detection_options)
    detector = vision.ObjectDetector.create_from_options(options)

    img = cv2.imread(image)
    detecta_latas(img, detector)

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument(
        '--model',
        type=str,
        default='../raspi/modelos/latas.tflite',
    )
    parser.add_argument(
        'image',
        type=str,
    )
    args = parser.parse_args()

    # TODO: Poder tomar foto con este mismo script
    main(args.model, args.image)
