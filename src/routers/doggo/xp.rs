pub fn get_xp_threshold(level: u32) -> u32 {
    1000 * (level) * (level + 1) / 2
}

pub fn get_level(xp: u32) -> u32 {
    (f64::sqrt((1 + 8 * (xp / 1000)) as f64) as u32 - 1) / 2
}

pub fn get_xp_remainder(xp: u32) -> u32 {
    let level = get_level(xp);
    let last_threshold = get_xp_threshold(level);
    xp - last_threshold
}

pub fn get_next_xp_target(xp: u32) -> u32 {
    let level = get_level(xp);
    (level + 1) * 1000
}
