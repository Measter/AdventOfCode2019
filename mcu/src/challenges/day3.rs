use core::convert::TryInto;

use shared::Reader;
use tinyvec::ArrayVec;

use super::ChallengeResponse;
use crate::rtc::RTC;

#[derive(Copy, Clone, Default)]
struct Point {
    x: i16,
    y: i16,
}

impl Point {
    fn distance(self) -> i16 {
        self.x.abs().saturating_add(self.y.abs())
    }
}

#[derive(Copy, Clone, Default)]
struct LineSegment {
    start: Point,
    end: Point,
}

impl LineSegment {
    fn get_intersection(self, other: Self) -> Option<Point> {
        // I'm going to assume they're not evil, and that intersections are at right angles.

        let (hor_line, vert_line) = if self.start.y == self.end.y {
            (self, other)
        } else {
            (other, self)
        };

        // They only intersect if hor_line's y is contained within vert_line's y-range and
        // vert_line's x is contained within hor_line's x-range.
        // But we need to normalize the ranges.
        let vert_start = vert_line.start.y.min(vert_line.end.y);
        let vert_end = vert_line.start.y.max(vert_line.end.y);
        let vert_range = vert_start..=vert_end;

        let hor_start = hor_line.start.x.min(hor_line.end.x);
        let hor_end = hor_line.start.x.max(hor_line.end.x);
        let hor_range = hor_start..=hor_end;

        if vert_range.contains(&hor_line.start.y) && hor_range.contains(&vert_line.start.x) {
            Some(Point {
                x: vert_line.start.x,
                y: hor_line.start.y,
            })
        } else {
            None
        }
    }

    fn length(self) -> u16 {
        let (start, end) = if self.start.y == self.end.y {
            (self.start.x, self.end.x)
        } else {
            (self.start.y, self.end.y)
        };

        (start.max(end) - start.min(end)) as u16
    }
}

pub fn run(rtc: &RTC) -> ChallengeResponse {
    let start = rtc.now();

    let mut input = Reader::open(include_bytes!("../../../inputs/aoc_1903.bin")).unwrap();

    let mut wire1_points = ArrayVec::<[Point; 302]>::new();
    let mut wire2_points = ArrayVec::<[Point; 302]>::new();
    let mut cur_point = Point::default();
    wire1_points.push(cur_point);
    wire2_points.push(cur_point);

    let mut buf = [0; 4];
    let mut dst = &mut wire1_points;
    while let Some(record) = input.next_record(&mut buf).unwrap() {
        // End of wire 1 instructions.
        if record == b"-" {
            dst = &mut wire2_points;
            cur_point = Point::default();
            continue;
        }

        let magnitude: i16 = core::str::from_utf8(&record[1..]).unwrap().parse().unwrap();
        let direction = record[0];

        match direction {
            b'U' => cur_point.y -= magnitude,
            b'D' => cur_point.y += magnitude,
            b'L' => cur_point.x -= magnitude,
            b'R' => cur_point.x += magnitude,
            _ => panic!("Invalid direction in Day 3 input."),
        }

        dst.push(cur_point);
    }

    // Part 1
    let mut closest = Point {
        x: i16::MAX,
        y: i16::MAX,
    };

    for pair in wire2_points.windows(2) {
        let w2_seg = LineSegment {
            end: pair[1],
            start: pair[0],
        };

        // Hmmm.... O(n^2)...
        for pair in wire1_points.windows(2) {
            let w1_seg = LineSegment {
                end: pair[1],
                start: pair[0],
            };

            match w1_seg.get_intersection(w2_seg) {
                Some(Point { x: 0, y: 0 }) => continue,
                Some(p) if p.distance() < closest.distance() => closest = p,
                _ => continue,
            }
        }
    }

    // Part 2
    let mut shortest = u16::MAX;
    let mut w2_distance = 0;
    for pair in wire2_points.windows(2) {
        let w2_seg = LineSegment {
            end: pair[1],
            start: pair[0],
        };

        // Hmmm.... O(n^2)...
        let mut w1_distance = 0;
        for pair in wire1_points.windows(2) {
            let w1_seg = LineSegment {
                end: pair[1],
                start: pair[0],
            };

            match w1_seg.get_intersection(w2_seg) {
                None | Some(Point { x: 0, y: 0 }) => {
                    w1_distance += w1_seg.length();
                }
                Some(p) => {
                    let w2_int = LineSegment { end: p, ..w2_seg };
                    let w1_int = LineSegment { end: p, ..w1_seg };
                    let distance = w2_distance + w1_distance + w2_int.length() + w1_int.length();

                    shortest = shortest.min(distance);
                }
            }
        }

        w2_distance += w2_seg.length();
    }

    let duration = rtc.now().elapsed_since(&start);
    ChallengeResponse {
        duration,
        part1: Some(closest.distance().try_into().unwrap()),
        part2: Some(shortest.into()),
    }
}
