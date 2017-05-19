extern crate rand;

use self::rand::Rng;

pub fn adjective() -> &'static str {
    const ADJECTIVES: &'static [&'static str] = &[
        "awesome",
        "edible",
        "delicious",
        "sick",
        "tasty",
        "yummy",
        "savory",
        "nourishing",
        "nutritious",
        "calorific",
        "soylent",
    ];

    rand::thread_rng().choose(ADJECTIVES).unwrap()
}

pub fn noun() -> &'static str {
    const NOUNS: &'static [&'static str] = &[
        "edible",
        "fishstick",
        "food",
        "treat",
        "digestible",
        "grub",
        "chow",
        "subsistence",
        "provision",
        "mouthful",
        "fodder",
    ];

    rand::thread_rng().choose(NOUNS).unwrap()
}

pub fn affirm() -> &'static str {
    const STRS: &'static [&'static str] = &[
        "I'll get you",
        "I'm taking that down as",
        "I'mma get you",
        "You're getting",
    ];

    rand::thread_rng().choose(STRS).unwrap()
}
