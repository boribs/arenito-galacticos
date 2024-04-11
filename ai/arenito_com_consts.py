from enum import Enum, auto

class AIMode(Enum):
    Simulation = auto()
    Real = auto() # don't like you
    Jetson = auto()

class Instruction(Enum):
    MoveForward       = auto()
    MoveLeft          = auto()
    MoveRight         = auto()
    MoveBack          = auto()
    MoveLongRight     = auto()
    RequestFrontCam   = auto()
    RequestRearCam    = auto()
    RequestProxSensor = auto()
    DumpCans          = auto()
    BrushOn           = auto()
    BrushOff          = auto()
    ExtendBackdoor    = auto()
    StopAll           = auto()

INSTRUCTION_MAP = {
    Instruction.MoveForward      : 'a',
    Instruction.MoveLeft         : 'i',
    Instruction.MoveRight        : 'd',
    Instruction.MoveBack         : 'r',
    Instruction.MoveLongRight    : 'D',
    Instruction.RequestProxSensor: 's',
    Instruction.DumpCans         : 'c',
    Instruction.BrushOn          : 'P',
    Instruction.BrushOff         : 'p',
    Instruction.ExtendBackdoor   : 'e',
    Instruction.StopAll          : 'S',
}
