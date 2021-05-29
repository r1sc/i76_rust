use nom::{call, count, do_parse, named, named_args, number::complete::le_u32, take};

pub struct CBK {
    pub num_entries: u32,
    pub patterns: Vec<Vec<u8>>
}

named_args!(fixed_vec(count: u32)<Vec<u8>>, 
    do_parse!(
        a: take!(count)
        >> (a.to_owned())
    )
);

impl super::common::Parsable<Self> for CBK {
    named!(parse<CBK>,
        do_parse!(
            num_entries: le_u32
            >> patterns: count!(call!(fixed_vec, 16), num_entries as usize)
            >> (CBK{
                num_entries,
                patterns
            })
        )
    );
}
