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

        if args.print_log:
            console = logging.StreamHandler()
            console.setLevel(logging.INFO)
            logging.getLogger().addHandler(console)

        self.classes: dict[str, int] = {}
        self.generation = 0

    def add_classname(self, classname: str):
        """
        Adds a name to the class dict.
        """

        self.classes[classname] = 0

    def info(self, msg: str):
        """
        Logs a message.
        """

        self.l.info(msg)

    def img(self, img: MatLike, classname: str):
        """
        Logs an immage.
        """

        if self.classes.get(classname, None) is None:
            self.add_classname(classname)

        filename = f'{classname}_{self.generation}_{self.classes[classname]}'
        self.l.info(f'Saved image "{filename}".')
        cv2.imwrite(f'img/{filename}.jpg', img)

        self.classes[classname] += 1

    def advance_gen(self):
        """
        Advances loggger generation.
        """

        self.generation += 1
        for key in self.classes:
            self.classes[key] = 0
