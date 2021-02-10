use shared::{decompress::Decompress, RunLengthEncoded};

pub enum Input {
    Compressed(Decompress<'static>),
    Raw {
        num_records: usize,
        data: &'static str,
    },
}

impl Input {
    pub fn new(input: &'static [u8]) -> Input {
        let (is_compressed, data) = input.split_first().expect("Invalid input");

        if *is_compressed == 1 {
            Input::Compressed(Decompress::open(data))
        } else {
            let (len, data) = RunLengthEncoded::decode(data).expect("Invalid input");

            Input::Raw {
                num_records: len as usize,
                data: core::str::from_utf8(data).expect("Invalid input"),
            }
        }
    }

    pub fn num_records(&self) -> usize {
        match self {
            Input::Compressed(d) => d.num_records(),
            Input::Raw { num_records, .. } => *num_records,
        }
    }
}
