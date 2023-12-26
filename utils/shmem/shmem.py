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


def printmem(mem: shared_memory.SharedMemory):
    print(bytes(mem.buf[:10]))

remove_shm_from_resource_tracker() # Python 3.12 and under
with open('shmem/shmem_test_2') as f:
    osid = f.read()[1:]

mem = shared_memory.SharedMemory(create=False, name=osid)
val = 100

mem.buf[0] = 45

while True:
    arr = bytearray(mem.buf)
    if arr[0] == 75:
        print(arr[:10])
        arr[2] = val
        val = (val + 1) % 255

        print(f'rust left val: {arr[1]}')
        arr[0] = 45
        mem.buf[:] = arr

    elif arr[0] == 100:
        break

mem.close()
