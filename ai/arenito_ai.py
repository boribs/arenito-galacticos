# pyright: strict

from __future__ import annotations
import cv2, time
from argparse import Namespace
from cv2.typing import MatLike
from dataclasses import dataclass
from enum import Enum, auto
from arenito_com import *
from arenito_vision import *
from arenito_timer import ArenitoTimer
from arenito_logger import ArenitoLogger
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

    TEST_TIME_SECS = 5 * 60 + 30
    BRUSH_ON_SECS = 7
    BRUSH_OFF_DIST = 80

    def __init__(self, args: Namespace):
        mode = MODE_DICT[args.mode]
        self.no_move = args.no_move
        self.headless = args.headless
        self.no_backdoor_extension = args.no_backdoor_extension

        self.log = ArenitoLogger(args)

        args_str = '\n'.join(
            f'    {arg}: {args.__dict__[arg]}'
            for arg in args.__dict__
        )
        self.log.info(f'Started AI with args [\n{args_str}\n]')

        self.com = ArenitoComms(mode, args, self.log)
        self.vis = ArenitoVision(mode, args, self.log)

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

        # pause mode
        self.pause = False
        self.turn_dir = Instruction.MoveRight if args.turn_dir == 'right' else Instruction.MoveLeft

    def scan(self) -> ScanResult:
        """
        Gets data from every sensor.
        """

        original = self.com.get_front_frame()
        blurred = self.vis.blur(original)
        detections = self.vis.find_cans(blurred)
        proximities = self.com.get_prox_sensors()
        dumping_zone = self.vis.detect_dumping_zone(blurred, False)

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
            self.log.info(f'New state: {self.state}')

    def main(self):
        """
        Main loop.
        """

        self.clock.start()

        # drop backdoor
        if not self.no_backdoor_extension:
            self.com.lcd_show('Extendiendo tapa', 1)
            self.com.send_instruction(Instruction.ExtendBackdoor)

        while self.clock.elapsed_time() < ArenitoAI.TEST_TIME_SECS:
            if cv2.waitKey(1) == 27:
                break

            self.check_pause()
            if self.pause:
                self.com.lcd_show('En pausa        ', 1)
                self.com.send_instruction(Instruction.StopAll)
                time.sleep(3)

            while self.pause:
                time.sleep(0.2)
                self.can_counter = 0
                self.check_pause()

                if not self.pause:
                    c = 5
                    while c > 0:
                        self.com.lcd_show(f'Continuando: {c}', 1)
                        time.sleep(1)
                        c -= 1
                    break

            scan_results = self.scan()

            self.get_state(scan_results)

            state_str = self.state.name
            if self.can_search_timer.clock and self.state == ArenitoState.LookingForCans:
                state_str += f': {self.can_search_timer.seconds()}'

            if self.brush_on_timer.elapsed_time() >= ArenitoAI.BRUSH_ON_SECS:
                self.brush_on_timer.reset()
                self.log.info('Brush timed out, turning off.')
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
                self.log.advance_gen()
                continue

            close_to_obstacle = 1 in scan_results.proximities[2:5]
            if close_to_obstacle:
                self.evade()
                continue

            if self.vis.can_in_critical_region(scan_results.detections):
                self.can_in_critical_region = True
            elif self.can_in_critical_region:
                self.can_in_critical_region = False
                self.can_counter += 1
                self.log.info('Recogio lata')

            if self.state == ArenitoState.GrabbingCan:
                self.get_can(scan_results)
                self.can_search_timer.reset()

                # better log: separate actions
                if not self.brush_on_timer.clock and not self.dump_too_close(scan_results.dumping_zone):
                    self.com.send_instruction(Instruction.BrushOn)

                self.brush_on_timer.start()

            elif self.state == ArenitoState.DumpingCans:
                self.dump_cans(scan_results)
                self.can_search_timer.reset()

            elif self.state == ArenitoState.LookingForCans:
                if not self.can_search_timer.clock:
                    self.can_search_timer.start()
                self.search_cans(scan_results)

            # Turn brush off if too close to dump
            if self.dump_too_close(scan_results.dumping_zone):
                self.log.info('Dump too close, turning off.')
                self.com.send_instruction(Instruction.BrushOff)

            self.log.advance_gen()

    def get_can(self, scan_results: ScanResult):
        """
        Can-getter routine.
        """

        def can_aligner(ai: ArenitoAI) -> Point:
            original = self.com.get_front_frame()
            blurred = self.vis.blur(original)
            detections = self.vis.find_cans(blurred)

            if not detections:
                return Point(256, 0)
            return detections[0].center

        self.align( # pyright: ignore[reportUnknownMemberType]
            scan_results.detections[0].center,
            self.vis.can_threshold,
            15,
            can_aligner,
            [self]
        )
        # tmin, tmax = self.vis.can_threshold_x
        # x = scan_results.detections[0].center.x
        # if tmax <= x:
        #     self.com.send_instruction(Instruction.MoveRight)
        # elif tmin >= x:
        #     self.com.send_instruction(Instruction.MoveLeft)
        # else:
        self.com.send_instruction(Instruction.MoveForward)

    def search_cans(self, scan_results: ScanResult):
        """
        Can-search routine.
        """

        MAX_SEARCH_SECONDS = 30

        hsv = cv2.cvtColor(scan_results.blurred, cv2.COLOR_BGR2HSV)

        if self.can_search_timer.elapsed_time() > MAX_SEARCH_SECONDS:
            self.log.info(f'Didn\'t find anything in {MAX_SEARCH_SECONDS}, beginning search routine.')
            self.com.lcd_show('Buscando latas', 1)

            for _ in range(24):
                for _ in range(10):
                    self.com.send_instruction(Instruction.MoveRight)
                scan_results = self.scan()

                if scan_results.detections:
                    self.log.info('Found can.')
                    return
                if scan_results.dumping_zone:
                    self.log.info('Found dump.')
                    return

            self.can_search_timer.reset()
        elif self.vis.reachable(hsv, self.vis.blue_r_dot, secondary_det=self.vis.dump_r_dot):
            self.com.send_instruction(Instruction.MoveForward)
        else:
            self.com.send_instruction(self.turn_dir)

    def evade(self):
        """
        Evasion routine.
        """

        self.log.info('Evading!')
        for _ in range(10):
            # Don't go back if on the border
            img = self.com.get_rear_frame()
            img = cv2.cvtColor(img, cv2.COLOR_BGR2HSV)
            if not self.vis.reachable(img, self.vis.blue_r_dot, secondary_det=self.vis.dump_r_dot):
                # self.log.info('Can\'t go back anymore, turning.')
                self.com.lcd_show('Evadiendo       ', 1)
                break

            self.com.send_instruction(Instruction.MoveBack)

        self.com.send_instruction(Instruction.MoveLongRight)

    def dump_cans(self, scan_results: ScanResult):
        """
        Can-dumping routine.
        """

        def get_dump(ai: ArenitoAI, frame: MatLike, rear: bool = False) -> Detection | None:
            img = ai.vis.blur(frame)
            return ai.vis.detect_dumping_zone(img, rear)

        def front_cam_align(ai: ArenitoAI) -> Point:
            dump = get_dump(ai, ai.com.get_front_frame())
            self.log.info(f'Aligning with front cam: {dump}')
            ai.log.advance_gen()

            self.check_pause()
            if self.pause:
                return Point(256, 0)

            if not dump:
                return Point(256, 0)
            return dump.center

        def rear_cam_align(ai: ArenitoAI) -> Point:
            dump = get_dump(ai, ai.com.get_rear_frame(), True)
            self.log.info(f'Aligning with rear cam: {dump}')
            ai.log.advance_gen()
            if not dump:
                return Point(256, 0)

            self.check_pause()
            if self.pause:
                Point(256, 0)
            return dump.center

        # def rear_sensor_align() -> tuple[int, int]:
        #     # SENSOR_ALIGN_THRESHOLD = 10
        #     while True:
        #         reads = self.com.get_prox_sensors()
        #         lu, ru = reads[0:2]
        #         if (lu < 10 and ru < 10): # or (ir == 1 or il == 1):
        #             break

        #         if abs(lu - ru) > 10:
        #             if lu > ru:
        #                 self.com.send_instruction(Instruction.MoveRight)
        #             else:
        #                 self.com.send_instruction(Instruction.MoveLeft)

        #     return lu, ru

        # get close (front cam)
        if not scan_results.dumping_zone: return

        self.com.send_instruction(Instruction.BrushOn)
        MAX_SEARCH_TIME = 10

        self.log.info(f'Getting close to dump.')
        dump = None
        dump_pos = scan_results.dumping_zone.center
        t = time.time()
        while time.time() - t < MAX_SEARCH_TIME:

            self.check_pause()
            if self.pause:
                return

            self.align( # pyright: ignore[reportUnknownMemberType]
                dump_pos,
                self.vis.deposit_threshold,
                15,
                front_cam_align,
                [self]
            )
            self.com.send_instruction(Instruction.MoveForward)
            front = self.com.get_front_frame()

            if self.pause:
                return

            dump = get_dump(self, front)

            if not dump:
                self.log.info('Lost dump? This shouldn\'t happen')
                break
            elif self.vis.deposit_critical_region.point_inside(dump.center):
                if self.dump_too_close(dump):
                    self.log.info('Dump too close, stepping back.')
                    self.com.send_instruction(Instruction.MoveBack)
                    time.sleep(0.4)
                    self.com.send_instruction(Instruction.StopAll)

                self.log.info('Front aligned.')
                self.com.send_instruction(Instruction.BrushOff)
                break
            # else:
                # don't go for dump if cans visible?
                # detections = self.vis.find_cans(front)
                # if detections:
                #     det_dist = self.vis.dist_from_center(detections[0].center)
                #     dump_dist = self.vis.dist_from_center(dump.center)

                #     if det_dist < dump_dist:
                #         return

                # dump_pos = dump.center

        self.com.send_instruction(Instruction.StopAll)
        time.sleep(0.5)

        self.log.info('Aligning with rear cam.')

        # step back, if possible
        rear = self.com.get_rear_frame()
        rear_hsv = cv2.cvtColor(rear, cv2.COLOR_BGR2HSV)
        if not dump:
            self.com.send_instruction(Instruction.MoveBack)
            time.sleep(2)
        elif self.vis.reachable(rear_hsv, self.vis.blue_r_dot, secondary_det=self.vis.dump_r_dot):
            self.log.info('Enough space in back, stepping back.')
            self.com.send_instruction(Instruction.MoveBack)
            time.sleep(0.7)

        self.com.send_instruction(Instruction.StopAll)
        time.sleep(0.2)

        # align (rear cam)
        t = time.time()
        while True:
            if time.time() - t >= MAX_SEARCH_TIME:
                self.log.info('Rear cam align timed out, terminating deposit routine.')
                return

            self.check_pause()
            if self.pause:
                return

            dump = get_dump(self, self.com.get_rear_frame(), True)
            self.log.advance_gen()
            if dump:
                self.log.info('Dump found with rear cam.')
                dump_pos = dump.center
                break
            else:
                self.com.send_instruction(Instruction.MoveRight)

        self.com.send_instruction(Instruction.StopAll)

        self.align( # pyright: ignore[reportUnknownMemberType]
            dump_pos,
            self.vis.deposit_threshold,
            15, # TODO: Make this a constant
            rear_cam_align,
            [self]
        )

        self.log.info(f'Getting close with proximity sensors.')
        # get close (sensors)
        # MAX_SENSOR_DIST = 4

        # t = time.time()
        # while time.time() - t < MAX_SEARCH_TIME:
        #     sensor, _ = rear_sensor_align()
        #     if sensor < MAX_SENSOR_DIST:
        #         break
        #     else:
        #         self.com.send_instruction(Instruction.MoveBack)

        t = time.time()
        while True:
            if time.time() - t >= 17:
                self.log.info('Rear sensor align timeout, terminating deposit routine.')
                return

            self.check_pause()
            if self.pause:
                return

            reads = self.com.get_prox_sensors()
            time.sleep(0.2)

            lu, ru = reads[0:2]
            ir, il = reads[5:7]

            self.log.info(f'Read {reads}. U:{lu},{ru}, Ir:{il},{ir}')

            if (lu < 10 and ru < 10) or (ir == 1 and il == 1):
                self.log.info('Aligned with proximity sensors.')
                break

            if abs(lu - ru) > 10:
                if (lu > ru) or (il > ir):
                    self.com.send_instruction(Instruction.MoveRight)
                elif (lu < ru) or (il < ir):
                    self.com.send_instruction(Instruction.MoveLeft)
            else:
                self.com.send_instruction(Instruction.MoveBack)

            time.sleep(0.2)
            self.com.send_instruction(Instruction.StopAll)
            time.sleep(0.1)

        self.log.info('Tiny step back.')
        self.com.send_instruction(Instruction.StopAll)
        time.sleep(0.2)
        self.com.send_instruction(Instruction.MoveBack)
        time.sleep(0.1)
        self.com.send_instruction(Instruction.StopAll)
        time.sleep(0.2)

        self.check_pause()
        if self.pause:
            return

        # dump cans
        self.log.info(f'Dumping {self.can_counter} cans.')
        self.com.dump_cans(self.can_counter)
        self.dumped_can_counter += self.can_counter
        self.can_counter = 0

        self.com.send_instruction(Instruction.MoveForward)
        time.sleep(0.7)
        self.log.info('Done dumping.')

    def align(
        self,
        det: Point,
        threshold: Threshold,
        timeout: int,
        callback: Callable[[ArenitoAI], Point],
        callback_args: Iterable[any] # pyright: ignore
    ):
        """
        Alignment function. Calls callback to update x value.
        """

        tmin, tmax = threshold.minmax(det)

        aligned = False
        t = time.time()
        while not aligned and time.time() - t < timeout:
            # mínimo y máximo como parámetro
            if tmax <= det.x:
                self.com.send_instruction(Instruction.MoveRight)
            elif tmin >= det.x:
                self.com.send_instruction(Instruction.MoveLeft)
            else:
                aligned = True

            det = callback(*callback_args) # pyright: ignore[reportUnknownArgumentType]

    def stop_all(self):
        """
        Stops everything
        """

        self.com.send_instruction(Instruction.StopAll)

    def print_stats(self):
        """
        Prints arenito stats.
        """

        self.log.info(f'Tiempo de ejecución: {self.clock.full()}')
        self.log.info(
            f'Arenito depositó {self.dumped_can_counter} latas'
            f', se quedó con {self.can_counter} latas dentro.'
        )

    def dump_too_close(self, dump: Detection | None) -> bool:
        """
        Returns True when the dump is too close.
        """

        if not dump: return False
        return self.vis.dist_from_center(dump.center) < ArenitoAI.BRUSH_OFF_DIST

    def check_pause(self):
        """"""

        if self.com.jetson_interface:
            if self.com.jetson_interface.check_pause():
                self.pause = not self.pause
