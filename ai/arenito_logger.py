import cv2, logging
from cv2.typing import MatLike
from argparse import Namespace

class ArenitoLogger:
    """
    Commodity logging class.
    """

    def __init__(self, args: Namespace):
        self.l = logging.getLogger()
        logging.basicConfig(
            filename='arenito.log',
            filemode='w',
            encoding='utf-8',
            level=logging.INFO
        )

        if args.pring_log:
            console = logging.StreamHandler()
            console.setLevel(logging.INFO)
            logging.getLogger().addHandler(console)

    def info(self, msg: str):
        """
        Logs a message.
        """

        self.l.info(msg)

    def img(self, img: MatLike, filename: str):
        """
        Logs an immage.
        """

        self.l.info(f'Saved image "{filename}".')
        cv2.imwrite(f'img/{filename}.jpg', img)
