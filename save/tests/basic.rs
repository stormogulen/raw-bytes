use save::{save_game, load_game};
use packed_struct_container::PackedStructContainer;
use bytemuck_derive::{Pod, Zeroable};
//use bytemuck::Pod;
use std::fs;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable, PartialEq)]
struct SaveData {
    player_id: u32,
    score: u32,
    level: u32,
}

impl SaveData {
    fn new(player_id: u32, score: u32, level: u32) -> Self {
        SaveData { player_id, score, level }
    }
}

#[test]
fn round_trip_save_load() {
    let container = PackedStructContainer::from_slice(&[
        SaveData::new(1, 9999, 7),
        SaveData::new(2, 1234, 2),
    ]);

    let path = "test_save.bin";
    save_game(path, &container).unwrap();

    let loaded = load_game::<_, SaveData>(path).unwrap();

    assert_eq!(loaded.len(), 2);
    let loaded_slice = loaded.as_slice();
    assert_eq!(loaded_slice[0], SaveData::new(1, 9999, 7));
    assert_eq!(loaded_slice[1], SaveData::new(2, 1234, 2));

    fs::remove_file(path).unwrap();
}

#[test]
fn detect_corrupt_save() {
    let container = PackedStructContainer::from_slice(&[
        SaveData::new(1, 9999, 7),
    ]);

    let path = "corrupt_test_save.bin";
    save_game(path, &container).unwrap();

    // Corrupt the file
    let mut bytes = fs::read(path).unwrap();
    bytes[33] ^= 0xFF; // flip a byte
    fs::write(path, &bytes).unwrap();

    let result = load_game::<_, SaveData>(path);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidData);

    fs::remove_file(path).unwrap();
}
