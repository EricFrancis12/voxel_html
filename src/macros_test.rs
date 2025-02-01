use crate::{json_bridge_parse, json_bridge_stringify, macros::Vec3Bridge};
use bevy::math::Vec3;

#[test]
fn test_json_bridge() {
    let result = json_bridge_stringify!(
        &Vec3 {
            x: 22.0,
            y: 23.0,
            z: 24.0,
        },
        Vec3Bridge
    );

    assert!(result.is_ok());
    let s = result.unwrap();

    let v: Vec3 = json_bridge_parse!(&s, Vec3Bridge).unwrap();

    assert_eq!(v.x, 22.0);
    assert_eq!(v.y, 23.0);
    assert_eq!(v.z, 24.0);
}
