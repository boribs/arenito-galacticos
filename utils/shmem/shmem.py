"""
Prueba de sincronización entre procesos.
"""

from multiprocessing import shared_memory, resource_tracker
import time

def remove_shm_from_resource_tracker():
    """Monkey-patch multiprocessing.resource_tracker so SharedMemory won't be tracked

    More details at: https://bugs.python.org/issue38119
    """

    def fix_register(name, rtype):
        if rtype == "shared_memory":
            return
        return resource_tracker._resource_tracker.register(self, name, rtype)
    resource_tracker.register = fix_register

    def fix_unregister(name, rtype):
        if rtype == "shared_memory":
            return
        return resource_tracker._resource_tracker.unregister(self, name, rtype)
    resource_tracker.unregister = fix_unregister

    if "shared_memory" in resource_tracker._CLEANUP_FUNCS:
        del resource_tracker._CLEANUP_FUNCS["shared_memory"]

class AISimMem:
    CAN_WRITE_RUST = 45
    CAN_WRITE_PYTHON = 75
    END = 100

    def __init__(self, mem: shared_memory.SharedMemory):
        self.mem = mem
        self.done_writing() # signal ready!

    def can_write(self) -> bool:
        return self.mem.buf[0] == AISimMem.CAN_WRITE_PYTHON

    def done_writing(self):
        self.mem.buf[0] = AISimMem.CAN_WRITE_RUST

    def signaled_end(self) -> bool:
        return self.mem.buf[0] == AISimMem.END

    def write_byte(self, val: int):
        self.mem.buf[2] = val

    def read_byte(self) -> int:
        return self.mem.buf[1]

    def show(self):
        print(bytes(self.mem.buf[:10]))

remove_shm_from_resource_tracker() # Python 3.12 and under
with open('shmem/shmem_test') as f:
    osid = f.read()[1:]

mem = shared_memory.SharedMemory(create=False, name=osid)
val = 100

aisim = AISimMem(mem)

while True:
    if aisim.can_write():
        aisim.show()
        aisim.write_byte(val)

        print(f'rust left val: {aisim.read_byte()}')
        val = (val + 1) % 255

        aisim.done_writing()

    elif aisim.signaled_end():
        break

mem.close()
