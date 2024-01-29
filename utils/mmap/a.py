import mmap, time

with open('file', 'r+') as f:
    with mmap.mmap(f.fileno(), length=0) as mmap_obj:
        #mmap_obj[0:100] = b"b" * 100
        print(mmap_obj[0:100])
        time.sleep(10)
