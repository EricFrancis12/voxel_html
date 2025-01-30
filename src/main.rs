use voxel_html::dom::VoxelData;

fn main() {
    let voxel_data = VoxelData::try_from_file("voxel_data.html");
    println!("{:?}", voxel_data);
}
