use voxel_html::dom::VoxelData;

fn main() {
    // let voxel_data = VoxelData::try_from_file_with_selector(
    //     "voxel_data.html",
    //     &Selector::parse("div#root").unwrap(),
    // )
    // .unwrap();

    let voxel_data = VoxelData::try_from_file("voxel_data.html");

    println!("{:?}", voxel_data);
}
