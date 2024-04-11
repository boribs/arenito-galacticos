# pyright: strict

from __future__ import annotations
import cv2, time, logging
from argparse import Namespace
from cv2.typing import MatLike
from dataclasses import dataclass
from enum import Enum, auto
from arenito_com import *
from arenito_vision import *
from arenito_timer import ArenitoTimer
from typing import Callable, Iterable

@dataclass
class ScanResult:
    original: MatLike
    blurred: MatLike
    detections: list[Detection]
    proximities: list[int]
    dumping_zone: None | Detection

class ArenitoState(Enum):
    LookingForCans = auto()
    GrabbingCan = auto()
    DumpingCans = auto()

MODE_DICT = {
    's' : AIMode.Simulation,
    'j' : AIMode.Jetson,
}

class ArenitoAI:
    """
    AI class, the brains of it all.
    """

    MIN_PROX_REACT_RANGE = 14
    TEST_TIME_SECS = 3 * 60
    BRUSH_ON_SECS = 7

    def __init__(self, args: Namespace):
        mode = MODE_DICT[args.mode]
        self.no_move = args.no_move
        self.headless = args.headless
        self.no_backdoor_extension = args.no_backdoor_extension

        self.create_logger(args.print_log)

        args_str = '\n'.join(
            f'    {arg}: {args.__dict__[arg]}'
            for arg in args.__dict__
        )
        self.logger.info(f'Started AI with args [\n{args_str}\n]')

        self.com = ArenitoComms(mode, args, self.logger)
        self.vis = ArenitoVision(mode, args, self.logger)

        self.state = ArenitoState.LookingForCans

        # Can tracking stuff
        self.can_search_timer = ArenitoTimer()
        self.can_counter = 0
        self.dumped_can_counter = 0
        self.can_in_critical_region = False

        # Brush stuff
        self.brush_on_timer = ArenitoTimer()

        # Clock
        self.clock = ArenitoTimer().start()

    def create_logger(self, print_log: bool):
        """
        Creates a logger.
        """

        self.logger = logging.getLogger()
        logging.basicConfig(
            filename='arenito.log',
            filemode='w',
            encoding='utf-8',
            level=logging.INFO
        )

        if print_log:
            console = logging.StreamHandler()
            console.setLevel(logging.INFO)
            logging.getLogger().addHandler(console)

    def scan(self) -> ScanResult:
        """
        Gets data from every sensor.
        """

        original = self.com.get_front_frame()
        blurred = self.vis.blur(original)
        detections = self.vis.find_cans(blurred)
        proximities = self.com.get_prox_sensors()
        dumping_zone = self.vis.detect_dumping_zone(blurred)

        return ScanResult(
            original=original,
            blurred=blurred,
            detections=detections,
            proximities=proximities,
            dumping_zone=dumping_zone
        )

    def get_state(self, scan_results: ScanResult):
        """
        Determines Arenito's current state based on sensor scan results.
        """

        prev_state = self.state

        if scan_results.detections:
            self.state = ArenitoState.GrabbingCan
            self.com.lcd_show('Recogiendo lata ', 1)
        elif scan_results.dumping_zone and self.can_counter > 0:
            self.state = ArenitoState.DumpingCans
            self.com.lcd_show('Depositando     ', 1)
        else:
            self.state = ArenitoState.LookingForCans
            self.com.lcd_show('Buscando lata   ', 1)

        if self.state != prev_state:
            self.logger.info(f'New state: {self.state}')

    def main(self):
        """
        Main loop.
        """

        test_timer = ArenitoTimer().start()

        # drop backdoor
        if not self.no_backdoor_extension:
            self.com.send_instruction(Instruction.ExtendBackdoor)

        while test_timer.elapsed_time() < ArenitoAI.TEST_TIME_SECS:
            if cv2.waitKey(1) == 27:
                break

            scan_results = self.scan()

            self.get_state(scan_results)

            state_str = self.state.name
            if self.can_search_timer.clock and self.state == ArenitoState.LookingForCans:
                state_str += f': {self.can_search_timer.seconds()}'

            if self.brush_on_timer.elapsed_time() >= ArenitoAI.BRUSH_ON_SECS:
                self.brush_on_timer.reset()
                self.com.send_instruction(Instruction.BrushOff)

            self.vis.add_markings(
                scan_results.original,
                scan_results.detections,
                state_str,
                self.can_counter,
                self.can_in_critical_region,
                scan_results.dumping_zone,
                self.clock.full()
            )

            if not self.headless:
                cv2.imshow('arenito pov', scan_results.original)

            if self.no_move:
                continue

            if min(scan_results.proximities[:2]) < ArenitoAI.MIN_PROX_REACT_RANGE:
                self.evade()
                continue

            if self.vis.can_in_critical_region(scan_results.detections):
                self.can_in_critical_region = True
            elif self.can_in_critical_region:
                self.can_in_critical_region = False
                self.can_counter += 1

            if self.state == ArenitoState.GrabbingCan:
                self.get_can(scan_results)
                self.can_search_timer.reset()

                if not self.brush_on_timer.clock:
                    self.com.send_instruction(Instruction.BrushOn)

                self.brush_on_timer.start()

            elif self.state == ArenitoState.DumpingCans:
                self.dump_cans(scan_results)
                self.can_search_timer.reset()

            elif self.state == ArenitoState.LookingForCans:
                if not self.can_search_timer.clock:
                    self.can_search_timer.start()
                self.search_cans(scan_results)

            self.vis.img_counter += 1

        # stats
        print(f'Tiempo de ejecución: {test_timer.full()}')
        print(f'Arenito depositó {self.dumped_can_counter} latas'
              f', se quedó con {self.can_counter} latas dentro.')

    def get_can(self, scan_results: ScanResult):
        """
        Can-getter routine.
        """

        def can_aligner(ai: ArenitoAI) -> int:
            original = self.com.get_front_frame()
            blurred = self.vis.blur(original)
            detections = self.vis.find_cans(blurred)

            if not detections:
                return 256
            return detections[0].center.x

        self.align( # pyright: ignore[reportUnknownMemberType]
            scan_results.detections[0].center.x,
            self.vis.can_threshold_x,
            15,
            can_aligner,
            [self]
        )
        self.com.send_instruction(Instruction.MoveForward)

    def search_cans(self, scan_results: ScanResult):
        """
        Can-search routine.
        """

        MAX_SEARCH_SECONDS = 20

        hsv = cv2.cvtColor(scan_results.blurred, cv2.COLOR_BGR2HSV)

        if self.can_search_timer.elapsed_time() > MAX_SEARCH_SECONDS:
            self.com.send_instruction(Instruction.MoveLongRight)
            self.can_search_timer.reset()
        elif self.vis.reachable(hsv, self.vis.r_dot):
            self.com.send_instruction(Instruction.MoveForward)
        else:
            self.com.send_instruction(Instruction.MoveRight)

    def evade(self):
        """
        Evasion routine.
        """

        for _ in range(10):
            # Don't go back if on the border
            img = self.com.get_rear_frame()
            img = cv2.cvtColor(img, cv2.COLOR_BGR2HSV)
            if not self.vis.reachable(img, self.vis.r_dot):
                break

            self.com.send_instruction(Instruction.MoveBack)

        self.com.send_instruction(Instruction.MoveLongRight)

    def dump_cans(self, scan_results: ScanResult):
        """
        Can-dumping routine.
        """

        def get_dump(ai: ArenitoAI, frame: MatLike) -> Detection | None:
            img = ai.vis.blur(frame)
            return ai.vis.detect_dumping_zone(img)

        def front_cam_align(ai: ArenitoAI) -> int:
            dump = get_dump(ai, ai.com.get_front_frame())
            if not dump:
                return 256
            return dump.center.x

        def rear_cam_align(ai: ArenitoAI) -> int:
            dump = get_dump(ai, ai.com.get_rear_frame())
            if not dump:
                return 256
            return dump.center.x

        def rear_sensor_align() -> tuple[int, int]:
            SENSOR_ALIGN_THRESHOLD = 2
            while True:
                der, izq = self.com.get_prox_sensors()[2:4]
                if abs(der - izq) <= SENSOR_ALIGN_THRESHOLD:
                    break

                if der > izq:
                    self.com.send_instruction(Instruction.MoveRight)
                elif izq > der:
                    self.com.send_instruction(Instruction.MoveLeft)

            return der, izq

        # get close (front cam)
        if not scan_results.dumping_zone: return

        MAX_SEARCH_TIME = 20

        dump_x = scan_results.dumping_zone.center.x
        t = time.time()
        while time.time() - t < MAX_SEARCH_TIME:
            self.align( # pyright: ignore[reportUnknownMemberType]
                dump_x,
                self.vis.can_threshold_x,
                15,
                front_cam_align,
                [self]
            )
            self.com.send_instruction(Instruction.MoveForward)
            dump = get_dump(self, self.com.get_front_frame())

            if not dump:
                break
            elif self.vis.deposit_critical_region.point_inside(dump.center):
                break
            else:
                dump_x = dump.center.x

        # align (rear cam)
        t = time.time()
        while time.time() - t < MAX_SEARCH_TIME:
            dump = get_dump(self, self.com.get_rear_frame())
            if dump:
                dump_x = dump.center.x
                break

            self.com.send_instruction(Instruction.MoveRight)

        self.align( # pyright: ignore[reportUnknownMemberType]
            dump_x,
            self.vis.deposit_threshold_x,
            15,
            rear_cam_align,
            [self]
        )

        # get close (sensors)
        MAX_SENSOR_DIST = 4

        t = time.time()
        while time.time() - t < MAX_SEARCH_TIME:
            sensor, _ = rear_sensor_align()
            if sensor < MAX_SENSOR_DIST:
                break
            else:
                self.com.send_instruction(Instruction.MoveBack)

        # dump cans
        self.com.dump_cans(self.can_counter)
        self.dumped_can_counter += self.can_counter
        self.can_counter = 0

    def align(
        self,
        initial_x: int,
        threshold: tuple[int, int],
        timeout: int,
        callback: Callable[[ArenitoAI], int],
        callback_args: Iterable[any] # pyright: ignore
    ):
        """
        Alignment function. Calls callback to update x value.
        """

        tmin, tmax = threshold
        x = initial_x
        aligned = False
        t = time.time()
        while not aligned and time.time() - t < timeout:
            # mínimo y máximo como parámetro
            if tmax <= x:
                self.com.send_instruction(Instruction.MoveRight)
            elif tmin >= x:
                self.com.send_instruction(Instruction.MoveLeft)
            else:
                aligned = True

            x = callback(*callback_args) # pyright: ignore[reportUnknownArgumentType]
