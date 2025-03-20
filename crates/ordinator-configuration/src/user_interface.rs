// TODO [ ]
// Import this if necessary
// use rgb::Rgb;
use serde::Deserialize;
use serde::Serialize;

// This should be a configuration type not a backend type!
#[derive(Serialize, Deserialize, Debug)]
pub struct EventColors
{
    wrench_time: (u8, u8, u8),
    work_break: (u8, u8, u8),
    toolbox: (u8, u8, u8),
    off_shift: (u8, u8, u8),
    non_productive_time: (u8, u8, u8),
    unavailable: (u8, u8, u8),
}
