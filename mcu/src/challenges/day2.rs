use shared::Reader;

use super::{intcode::IntCode, ChallengeResponse};
use crate::rtc::RTC;

fn load_program(mem: &mut [u32]) {
    let mut input = Reader::open(include_bytes!("../../../inputs/aoc_1902.bin")).unwrap();
    let mut buf = [0; 3];
    let mut mem = mem.iter_mut();
    while let Some((record, dst)) = input.next_record(&mut buf).unwrap().zip(mem.next()) {
        *dst = core::str::from_utf8(record).unwrap().parse().unwrap();
    }
}

pub fn run(rtc: &RTC) -> ChallengeResponse {
    let start = rtc.now();

    let mut orig_memory = [0_u32; 140];
    load_program(&mut orig_memory);

    // Part 1
    let mut memory = orig_memory.clone();
    memory[1] = 12;
    memory[2] = 2;

    let mut cpu = IntCode::new(&mut memory);
    while !cpu.is_halted() {
        cpu.step().unwrap();
    }

    let p1_res = memory[0];

    // Part 2
    let mut p2_res = None;
    'outer: for noun in 0..100 {
        for verb in 0..100 {
            memory.copy_from_slice(&orig_memory);
            memory[1] = noun;
            memory[2] = verb;

            let mut cpu = IntCode::new(&mut memory);
            while !cpu.is_halted() {
                cpu.step().unwrap();
            }

            if memory[0] == 19690720 {
                p2_res = Some(noun * 100 + verb);
                break 'outer;
            }
        }
    }

    let p2_res = p2_res.expect("No answer found for D2P2");

    let duration = rtc.now().elapsed_since(&start);
    ChallengeResponse {
        duration,
        part1: Some(p1_res.into()),
        part2: Some(p2_res.into()),
    }
}
