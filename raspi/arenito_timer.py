import time

class ArenitoTimer:
    """
    A convenient timer class.
    """

    def __init__(self):
        self.clock: float | None = None

    def start(self):
        self.clock = time.time()

    def elapsed_time(self) -> float:
        return time.time() - self.clock if self.clock else 0

    def reset(self):
        self.clock = None

    def seconds(self) -> str:
        if not self.clock:
            return 'Not set'
        else:
            elapsed = self.elapsed_time()
            m = elapsed // 60
            s = elapsed % 60
            return '{0:.2f}'.format(s)
