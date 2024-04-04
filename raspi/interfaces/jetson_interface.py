# pyright: strict

from arenito_com_consts import *
from cv2.typing import MatLike
import cv2, os
from argparse import Namespace

if os.getenv('IS_JETSON'):
    import Jetson.GPIO as GPIO # pyright: ignore
    from interfaces.serial_interface import SerialInterface
    import utils.I2C_LCD_driver as I2C_LCD_driver

class ArenitoCameras:
    """
    Interface between video input and ArenitoComms.
    """

    def __init__(self):
        self.cameras: list[cv2.VideoCapture] = []

    def add_video_capture(self):
        """
        Initializes video capture device.
        """

        # must get indices in a better way...
        # https://stackoverflow.com/questions/57577445/list-available-cameras-opencv-python
        index = len(self.cameras)
        capture_device = cv2.VideoCapture(index)

        ok, _ = capture_device.read()
        if not ok or not capture_device.isOpened():
            raise Exception(f'Can\'t use capture device on index {index}')

        self.cameras.append(capture_device)

    def get_front_frame(self) -> MatLike:
        """
        Requests front camera's frame.
        """

        _, frame = self.cameras[0].read()
        return frame

    def get_rear_frame(self) -> MatLike:
        """
        Requests rear camera's frame.
        """

        _, frame = self.cameras[1].read()
        return frame

class JetsonInterface:
    """
    Sensor interaction through NVIDIA Jetson Nano.
    The Jetson has an Arduino board as a slave, to facilitate sensor data retrieval.
    These boards communicate through serial interface (Jetson Nano port /dev/ttyTHS1).
    """

    BUTTON_CALIBRATION_PIN = 18
    BUTTON_START_AI = 16

    def __init__(
        self,
        args: Namespace,
        no_cam: bool = False,
        no_start: bool = False,
        no_lcd: bool = False,
    ):
        GPIO.setmode(GPIO.BOARD) # pyright: ignore[reportUnknownMemberType, reportPossiblyUnboundVariable]
        GPIO.setup([ # pyright: ignore[reportUnknownMemberType, reportPossiblyUnboundVariable]
                JetsonInterface.BUTTON_CALIBRATION_PIN,
                JetsonInterface.BUTTON_START_AI,
            ],
            GPIO.IN # pyright: ignore[reportUnknownMemberType, reportPossiblyUnboundVariable]
        )

        # Start button, required by rules.
        if not no_start:
            self.lcd_show('Esperando inicio', 1)
            GPIO.wait_for_edge(JetsonInterface.BUTTON_START_AI, GPIO.FALLING) # pyright: ignore[reportUnknownMemberType, reportPossiblyUnboundVariable]

        self.serial_interface = SerialInterface(args.port, args.baudrate) # pyright: ignore[reportPossiblyUnboundVariable]

        # LCD1602 with i2c shield
        # can be any LCD with i2c, though
        if not no_lcd:
            self.lcd = I2C_LCD_driver.lcd() # pyright: ignore[reportPossiblyUnboundVariable]
            self.lcd.lcd_clear()

        # Camera setup:
        # 1. Connect the front camera
        # 2. Start the AI script
        # 3. Connect rear camera
        # 4. Press camera config button
        if not no_cam:
            self.cameras = ArenitoCameras()
            self.init_cameras()

    def lcd_show(self, msg: str, line: int):
        """
        Displays some text on the mounted LCD display.
        """

        self.lcd.lcd_display_string(msg, line) # pyright: ignore[reportUnknownMemberType]
        print(f'[INFO] {msg}')

    def init_cameras(self):
        """
        Camera initialization routine: first camera -> front camera, second camera -> rear camera.
        """

        self.cameras.add_video_capture()

        self.lcd_show('Conecte camara trasera', 1)
        self.lcd_show('y oprima el boton', 2)
        GPIO.wait_for_edge(JetsonInterface.BUTTON_CALIBRATION_PIN, GPIO.FALLING) # pyright: ignore[reportUnknownMemberType, reportPossiblyUnboundVariable]

        self.cameras.add_video_capture()

        # maybe don't do this?
        cv2.imwrite('frontal.png', self.cameras.get_front_frame())
        cv2.imwrite('rear.png', self.cameras.get_rear_frame())

    def get_front_frame(self) -> MatLike:
        """
        Requests front camera's frame.
        """

        return self.cameras.get_front_frame()

    def get_rear_frame(self) -> MatLike:
        """
        Requests rear camera's frame.
        """

        return self.cameras.get_rear_frame()

    def get_prox_sensors(self) -> list[int]:
        """
        Returns proximity sensor reads.
        """

        return self.serial_interface.get_prox_sensors()

    def send_instruction(self, instr: Instruction):
        """
        Requests some instructions execution.
        """

        self.serial_interface.send_instruction(instr)

    def dump_cans(self):
        """
        Dumps cans.
        """

        self.serial_interface.dump_cans()
