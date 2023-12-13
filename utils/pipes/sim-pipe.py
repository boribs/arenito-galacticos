import posix, time

PIPE_PATH = '../../pipes/pipe'

def create_pipe(path):
    try:
        posix.mkfifo(path)
        print(f'created {path}')
    except FileExistsError:
        print(f'{path} already exists')
    except:
        print(f'Can\'t create {path}')
        exit(1)

create_pipe(PIPE_PATH)

while True:
    for _ in range(10):
        with open(PIPE_PATH, 'w') as pipe:
            pipe.write('mv:fw')
        time.sleep(0.01)

    for _ in range(15):
        with open(PIPE_PATH, 'w') as pipe:
            pipe.write('mv:r')
        time.sleep(0.01)

    # time.sleep(0.1)
