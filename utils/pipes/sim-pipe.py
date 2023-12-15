import posix, time
from PIL import Image

INSTRUCTION_PIPE_PATH = '../../pipes/instrpipe'
IMAGE_PIPE_PATH = '../../pipes/imgpipe'

def create_pipe(path):
    try:
        posix.mkfifo(path)
        print(f'created {path}')
    except FileExistsError:
        print(f'{path} already exists')
    except:
        print(f'Can\'t create {path}')
        exit(1)

create_pipe(INSTRUCTION_PIPE_PATH)
create_pipe(IMAGE_PIPE_PATH)

# while True:
#     for _ in range(10):
#         with open(PIPE_PATH, 'w') as pipe:
#             pipe.write('mv:fw')
#         time.sleep(0.01)

#     for _ in range(15):
#         with open(PIPE_PATH, 'w') as pipe:
#             pipe.write('mv:r')
#         time.sleep(0.01)

#     # time.sleep(0.1)

with open(INSTRUCTION_PIPE_PATH, 'w') as pout:
    pout.write('ss')

with open(IMAGE_PIPE_PATH, 'rb') as pin:
    im = Image.frombytes('RGB', (1024, 1024), pin.read())
    im.save('img.jpg')
