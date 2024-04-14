import mmap

def create_file(name, size):
    with open(name, 'wb') as f:
        f.truncate(size)

class AISimMem:
    # sync constants
    AI_FRAME_REQUEST = 1
    SIM_FRAME_WAIT = 2
    AI_MOVE_INSTRUCTION = 3
    SIM_AKNOWLEDGE_INSTRUCTION = 4

    # movement instruction constants
    MOV_FORWARD = ord('a')
    MOV_LEFT = ord('i')
    MOV_RIGHT = ord('d')
    MOV_LONG_RIGHT = ord('D')

    # memory footprint
    IMG_SIZE = 3_145_728

    def __init__(self):
        with open('C:\\Users\\chris\\Downloads\\Arenito\\arenito-galacticos\\sim\\file', 'r+') as file:
            self.mem = mmap.mmap(file.fileno(), length=0)
        # should close self.mem somewhere

    def write_byte(self, val: int):
        self.mem[0] = val

    def write_mov_instruction(self, val: int):
        self.mem[1] = val

    def read_byte(self) -> int:
        return self.mem[0]

    def wait_confirmation(self):
        while self.mem[0] != AISimMem.SIM_AKNOWLEDGE_INSTRUCTION:
            pass

    def send_instruction(self, instruction: int):
        self.write_mov_instruction(instruction)
        self.write_byte(AISimMem.AI_MOVE_INSTRUCTION)
        self.wait_confirmation()

aisim = AISimMem()
for _ in range(10):
    aisim.send_instruction(AISimMem.MOV_FORWARD)


#create_file('file', 3_145_729)