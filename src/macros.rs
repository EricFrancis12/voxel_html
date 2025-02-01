use bevy::{
    math::{Quat, Vec3},
    transform::components::Transform,
};
use serde::{Deserialize, Serialize};

#[macro_export]
macro_rules! or_else {
    ($ex: expr, $op: expr) => {
        match $ex {
            Ok(val) => val,
            Err(_) => $op,
        }
    };
}

#[derive(Deserialize, Serialize)]
pub struct Vec3Bridge {
    x: f32,
    y: f32,
    z: f32,
}

impl From<Vec3> for Vec3Bridge {
    fn from(value: Vec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl Into<Vec3> for Vec3Bridge {
    fn into(self) -> Vec3 {
        Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct QuatBridge {
    m128: [f32; 4],
}

impl From<Quat> for QuatBridge {
    fn from(value: Quat) -> Self {
        Self {
            m128: value.to_array(),
        }
    }
}

impl Into<Quat> for QuatBridge {
    fn into(self) -> Quat {
        Quat::from_vec4(self.m128.into())
    }
}

#[derive(Deserialize, Serialize)]
pub struct TransformBridge {
    translation: Vec3Bridge,
    rotation: QuatBridge,
    scale: Vec3Bridge,
}

impl From<Transform> for TransformBridge {
    fn from(value: Transform) -> Self {
        Self {
            translation: value.translation.into(),
            rotation: value.rotation.into(),
            scale: value.scale.into(),
        }
    }
}

impl Into<Transform> for TransformBridge {
    fn into(self) -> Transform {
        Transform {
            translation: self.translation.into(),
            rotation: self.rotation.into(),
            scale: self.scale.into(),
        }
    }
}

#[macro_export]
macro_rules! json_bridge_stringify {
    ($v:expr, $bridge_t:tt) => {
        serde_json::to_string(&$bridge_t::from(*$v))
    };
}
#[macro_export]
macro_rules! json_bridge_parse {
    ($s:expr, $bridge_t:tt) => {
        serde_json::from_str::<$bridge_t>($s).map(|a| a.into())
    };
}

#[macro_export]
macro_rules! inline_bridge_parse {
    ($s:expr, $bridge_t:tt) => {
        todo!()
    };
}

#[macro_export]
macro_rules! json_bridge_impl {
    ($t:tt, $bridge_t:tt) => {
        impl $bridge_t {
            pub fn json_stringify(val: &$t) -> Result<String, serde_json::error::Error> {
                crate::json_bridge_stringify!(val, $bridge_t)
            }

            pub fn json_parse(s: &str) -> Result<$t, serde_json::error::Error> {
                crate::json_bridge_parse!(s, $bridge_t)
            }

            pub fn inline_parse(s: &str) -> Result<$t, serde_json::error::Error> {
                crate::inline_bridge_parse!(s, $bridge_t)
            }
        }
    };
}

json_bridge_impl!(Vec3, Vec3Bridge);
json_bridge_impl!(Quat, QuatBridge);
json_bridge_impl!(Transform, TransformBridge);
