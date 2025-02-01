use crate::dom::VoxelData;
use bevy::{
    math::{Quat, Vec3},
    transform::components::Transform,
};
use scraper::Selector;
use std::str::FromStr;

const HTML_STR: &str = include_str!("../voxel_data.html");

#[test]
fn test_voxel_data_from_str() {
    let invalid_html_strs = vec![
        // missing <!DOCTYPE html>
        r#"
            <html>
                <body></body>
            </html>
        "#,
    ];

    let valid_html_strs = vec![
        r#"
            <!DOCTYPE html>
            <html></html>
        "#,
        r#"
            <!DOCTYPE html>
            <body></body>
        "#,
        r#"
            <!DOCTYPE html>
            <html>
                <body></body>
            </html>
        "#,
    ];

    let transform_tests = vec![
        ("", Transform::default()),
        (
            r#" data-transform='{"translation":{"x":17.0,"y":18.0,"z":19.0}, "rotation":{"m128":[0.0,0.0,0.0,0.0]}, "scale":{"x":2.0,"y":2.0,"z":2.0}}'>"#,
            Transform {
                translation: Vec3::new(17.0, 18.0, 19.0),
                rotation: Quat::default(),
                scale: Vec3::new(2.0, 2.0, 2.0),
            },
        ),
        (
            r#" data-transform.translation='{"x":17.0,"y":18.0,"z":19.0}'>"#,
            Transform {
                translation: Vec3::new(17.0, 18.0, 19.0),
                rotation: Quat::default(),
                scale: Vec3::default(),
            },
        ),
        // TODO: ...
        // (
        //     r#" data-transform.translation="x: 17.0; y: 18.0; z: 19.0">"#,
        //     Transform {
        //         translation: Vec3::new(17.0, 18.0, 19.0),
        //         rotation: Quat::default(),
        //         scale: Vec3::default(),
        //     },
        // ),
        (
            r#" data-transform.translation.x="7" data-transform.translation.y="8" data-transform.translation.z="9">"#,
            Transform {
                translation: Vec3::new(7.0, 8.0, 9.0),
                rotation: Quat::default(),
                scale: Vec3::default(),
            },
        ),
    ];

    for html_str in invalid_html_strs {
        assert!(VoxelData::from_str(html_str).is_err());
    }

    for html_str in valid_html_strs {
        assert!(VoxelData::from_str(html_str).is_ok());
    }

    for (s, t) in transform_tests {
        let html_str = format!(
            r#"
                <!DOCTYPE html>
                <html>
                    <body{}>
                        <div{}>
                        </div>
                    </body>
                </html>
            "#,
            s, s,
        );

        let result = VoxelData::from_str(&html_str);
        assert!(result.is_ok());

        let vd = result.unwrap();

        assert_eq!(t.translation.x, vd.root.transform.translation.x);
        assert_eq!(t.translation.y, vd.root.transform.translation.y);
        assert_eq!(t.translation.z, vd.root.transform.translation.z);

        assert_eq!(vd.elements.len(), 1);
        assert_eq!(t.translation.x, vd.elements[0].transform.translation.x);
        assert_eq!(t.translation.y, vd.elements[0].transform.translation.y);
        assert_eq!(t.translation.z, vd.elements[0].transform.translation.z);
    }
}

#[test]
fn test_voxel_data_from_str_with_selector() {
    assert!(
        VoxelData::from_str_with_selector(HTML_STR, &Selector::parse("div#root").unwrap()).is_ok()
    );
}
