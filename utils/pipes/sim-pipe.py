import posix, time
from PIL import Image

SIMOUT_PIPE_PATH = '../../pipes/simout'
AIOUT_PIPE_PATH = '../../pipes/aiout'

def create_pipe(path):
    try:
        posix.mkfifo(path)
        print(f'created {path}')
    except FileExistsError:
        print(f'{path} already exists')
    except:
        print(f'Can\'t create {path}')
        exit(1)

create_pipe(SIMOUT_PIPE_PATH)
create_pipe(AIOUT_PIPE_PATH)

while True:
    with open(AIOUT_PIPE_PATH, 'w') as pipe:
        pipe.write('mv:a;')

    with open(SIMOUT_PIPE_PATH, 'r') as pipe:
        pipe.read()

    with open(AIOUT_PIPE_PATH, 'w') as pipe:
        pipe.write('mv:i;')

    with open(SIMOUT_PIPE_PATH, 'r') as pipe:
        pipe.read()
