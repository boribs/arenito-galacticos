import posix

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

with open(PIPE_PATH, 'w') as pipe:
    pipe.write('alksdflksjhf')
