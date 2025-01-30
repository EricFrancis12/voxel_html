use crate::dom::VoxelData;
use scraper::Selector;
use std::str::FromStr;

const HTML_STR: &str = include_str!("../voxel_data.html");

#[test]
fn test_voxel_data_from_str() {
    VoxelData::from_str(HTML_STR).unwrap();
    assert!(VoxelData::from_str(HTML_STR).is_ok());
}

#[test]
fn test_voxel_data_from_str_with_selector() {
    assert!(
        VoxelData::from_str_with_selector(HTML_STR, &Selector::parse("div#root").unwrap()).is_ok()
    );
}
