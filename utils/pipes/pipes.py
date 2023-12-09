# This script should manage pipes!
# By default pipes.py is writer and rust app is reader.
# Once it writes something, this process becomes reader.

from enum import Enum, auto
import posix, time

class State(Enum):
    READER = auto()
    WRITER = auto()

def create_pipe(path):
    try:
        posix.mkfifo(path)
        print(f'created {path}')
    except FileExistsError:
        print(f'{path} already exists')
    except:
        print(f'Can\'t create {path}')
        exit(1)

PIPE_PATH = './pipe'
create_pipe(PIPE_PATH)

state = State.WRITER

while True:
    print(state)
    match state:
        case State.WRITER:
            with open(PIPE_PATH, 'w') as pout:
                for i in range(10):
                    time.sleep(.1)
                    pout.write(f'{i}')
                    print('writing', i)

            state = State.READER

        case State.READER:
            with open(PIPE_PATH, 'r') as pipe:
                print('read:', pipe.read())

            state = State.WRITER
