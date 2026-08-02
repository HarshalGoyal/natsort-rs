#![no_main]

use libfuzzer_sys::fuzz_target;

use natsort::{
    natsorted, natsorted_bytes, natsorted_mixed, natsorted_rev, natsorted_with, os_sorted,
    realsorted, Item, NatsortKey, NsFlags,
};

// Representative flag set to drive as many regex/transform branches as possible.
const FLAGS: [NsFlags; 8] = [
    NsFlags::INT,
    NsFlags::REAL,
    NsFlags::IGNORECASE,
    NsFlags::PATH,
    NsFlags::LOCALE,
    NsFlags::NUMAFTER,
    NsFlags::PRESORT,
    NsFlags::GROUPLETTERS,
];

fn bytes_to_items(data: &[u8]) -> Vec<String> {
    data.split(|&b| b == b',')
        .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
        .collect()
}

fuzz_target!(|data: &[u8]| {
    let items: Vec<String> = bytes_to_items(data);
    let strs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();

    // Exercise every sort entry point with the same input.
    let _ = natsorted(&strs);
    let _ = natsorted_rev(&strs);
    let _ = realsorted(&strs);
    let _ = os_sorted(&strs);

    // Raw bytes path.
    let byte_slices: Vec<&[u8]> = items.iter().map(|s| s.as_bytes()).collect();
    let _ = natsorted_bytes(&byte_slices);

    // Mixed-type path (first element forced to a float to cover Float/Int branches).
    let mixed: Vec<Item> = items
        .iter()
        .enumerate()
        .map(|(i, s)| {
            if i == 0 {
                Item::Float(s.parse::<f64>().unwrap_or(0.0))
            } else {
                Item::Str(s.clone())
            }
        })
        .collect();
    let _ = natsorted_mixed(&mixed);

    // Keygen core across representative flags (where panics/UB would land).
    let joined = strs.join(",");
    for flags in FLAGS {
        let _ = NatsortKey::new(flags).key(&joined);
        let _ = natsorted_with(&strs, flags);
    }
});