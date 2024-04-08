from __future__ import annotations
import time

class ArenitoTimer: # rename this to Timer
    """
    A convenient timer class.
    """

    def __init__(self):
        self.clock: float | None = None

    def start(self) -> ArenitoTimer:
        self.clock = time.time()
        return self

    def elapsed_time(self) -> float:
        return time.time() - self.clock if self.clock else 0

    def reset(self):
        self.clock = None

    def seconds(self) -> str:
        if not self.clock:
            return 'Not set'
        else:
            elapsed = self.elapsed_time()
            s = elapsed % 60
            return '{0:.2f}'.format(s)

    def full(self) -> str:
        if not self.clock:
            return 'Not set'
        else:
            elapsed = self.elapsed_time()
            m = elapsed // 60
            s = elapsed % 60
            return f'{int(m)}m {int(s)}s'
