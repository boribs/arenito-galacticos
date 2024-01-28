import mmap, time

with open('file', 'r+') as f:
    with mmap.mmap(f.fileno(), length=0) as mmap_obj:
        print(mmap_obj[0:100])
