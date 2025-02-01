use crate::{
    errors::Error,
    macros::{QuatBridge, TransformBridge, Vec3Bridge},
    or_else,
};
use bevy::transform::components::Transform;
use ego_tree::NodeRef;
use scraper::{node::Element, ElementRef, Html, Node, Selector};
use std::{fs::File, io::Read, path::PathBuf, str::FromStr};

#[derive(Debug)]
enum VoxelTagName {
    Body,
    Div,
}

impl FromStr for VoxelTagName {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "body" => Ok(Self::Body),
            "div" => Ok(Self::Div),
            _ => Err(Error::VoxelTagNameParseError(format!("unknown tag: {}", s))),
        }
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
enum VoxelAttributeName {
    Transform,
}

impl VoxelAttributeName {
    fn from_dataset_str(s: &str) -> Result<Self, Error> {
        if let Some(_s) = s.strip_prefix("data-") {
            return Self::from_str(_s);
        }
        Err(Error::VoxelAttributeNameParseError(
            "expected prefix `data-`".to_owned(),
        ))
    }
}

impl FromStr for VoxelAttributeName {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "transform" => Ok(Self::Transform),
            _ => Err(Error::VoxelAttributeNameParseError(format!(
                "unknown attribute: {}",
                s,
            ))),
        }
    }
}

enum Vec3PropName {
    X,
    Y,
    Z,
}

impl FromStr for Vec3PropName {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "x" => Ok(Self::X),
            "y" => Ok(Self::Y),
            "z" => Ok(Self::Z),
            _ => Err(Error::VXStyleNameParseError(format!(
                "unknown Vec3 property: {}",
                s,
            ))),
        }
    }
}

#[derive(Debug)]
pub struct VoxelElement {
    tag: VoxelTagName,
    children: Vec<VoxelElement>,
    pub transform: Transform,
}

impl<'a> VoxelElement {
    fn new(tag: VoxelTagName) -> Self {
        Self {
            tag,
            children: Vec::new(),
            transform: Transform::default(),
        }
    }
}

impl TryFrom<&Element> for VoxelElement {
    type Error = Error;

    fn try_from(value: &Element) -> Result<Self, Self::Error> {
        let mut ve = Self::new(VoxelTagName::from_str(value.name())?);

        for (key, val) in value.attrs() {
            let key_parts: Vec<&str> = key.split(".").collect();
            if let Some(part_0) = key_parts.get(0) {
                if let Ok(name) = VoxelAttributeName::from_dataset_str(part_0) {
                    match name {
                        VoxelAttributeName::Transform => {
                            if let Some(part_1) = key_parts.get(1) {
                                match *part_1 {
                                    "translation" => {
                                        if let Some(part_2) = key_parts.get(2) {
                                            match *part_2 {
                                                "x" => {
                                                    ve.transform.translation.x =
                                                        or_else!(val.parse::<f32>(), continue);
                                                }
                                                "y" => {
                                                    ve.transform.translation.y =
                                                        or_else!(val.parse::<f32>(), continue);
                                                }
                                                "z" => {
                                                    ve.transform.translation.z =
                                                        or_else!(val.parse::<f32>(), continue);
                                                }
                                                _ => {}
                                            }
                                        } else {
                                            ve.transform.translation = or_else!(
                                                // TODO: ...
                                                // Vec3Bridge::inline_parse(val)
                                                //     .or(Vec3Bridge::json_parse(val)),
                                                Vec3Bridge::json_parse(val),
                                                continue
                                            );
                                        }
                                    }
                                    "rotation" => {
                                        if let Some(part_2) = key_parts.get(2) {
                                            match *part_2 {
                                                _ => todo!(),
                                            }
                                        } else {
                                            ve.transform.rotation =
                                                or_else!(QuatBridge::json_parse(val), continue);
                                        }
                                    }
                                    "scale" => {
                                        if let Some(part_2) = key_parts.get(2) {
                                            match *part_2 {
                                                "x" => {
                                                    ve.transform.scale.x =
                                                        or_else!(val.parse::<f32>(), continue);
                                                }
                                                "y" => {
                                                    ve.transform.scale.y =
                                                        or_else!(val.parse::<f32>(), continue);
                                                }
                                                "z" => {
                                                    ve.transform.scale.z =
                                                        or_else!(val.parse::<f32>(), continue);
                                                }
                                                _ => {}
                                            }
                                        } else {
                                            ve.transform.scale =
                                                or_else!(Vec3Bridge::json_parse(val), continue);
                                        }
                                    }
                                    _ => {}
                                }
                            } else {
                                ve.transform = or_else!(TransformBridge::json_parse(val), continue);
                            }
                        }
                    }
                }
            }
        }

        return Ok(ve);
    }
}

type NR<'a> = NodeRef<'a, Node>;

crate::children_from_impl!(VoxelElement, NR, children);

impl<'a> TryFrom<NodeRef<'a, Node>> for VoxelElement {
    type Error = Error;

    fn try_from(value: NodeRef<'a, Node>) -> Result<Self, Self::Error> {
        let node = value.value();

        if let Node::Element(element) = node {
            let mut ve = Self::try_from(element)?;
            ve.children_from(value)?;
            return Ok(ve);
        }

        Err(Error::VoxelElementParseError(format!(
            "expected element node, but got: {:?}",
            node,
        )))
    }
}

#[derive(Debug)]
pub struct VoxelData {
    pub root: VoxelElement,
    pub elements: Vec<VoxelElement>,
}

impl VoxelData {
    fn new(tag: VoxelTagName) -> Self {
        Self {
            root: VoxelElement::new(tag),
            elements: Vec::new(),
        }
    }

    pub fn try_from_file(path: impl Into<PathBuf>) -> Result<Self, Error> {
        Self::try_from(path.into())
    }

    pub fn try_from_file_with_selector(
        path: impl Into<PathBuf>,
        selector: &Selector,
    ) -> Result<Self, Error> {
        let mut file = File::open::<PathBuf>(path.into())?;
        let mut s = String::new();
        file.read_to_string(&mut s)?;
        Self::from_str_with_selector(s, selector)
    }

    pub fn from_str_with_selector(
        s: impl Into<String>,
        selector: &Selector,
    ) -> Result<Self, Error> {
        let html = Html::parse_document(&s.into());
        if html.errors.len() > 0 {
            return Err(Error::VoxelDataParseError(format!(
                "received ({}) html parsing error(s):\n{:?}",
                html.errors.len(),
                html.errors
                    .iter()
                    .enumerate()
                    .map(|(i, e)| format!("[{}]{}", i + 1, e))
                    .collect::<Vec<String>>()
                    .join("\n"),
            )));
        }

        match html.select(selector).next() {
            Some(element) => Self::try_from(element),
            None => Err(Error::VoxelDataParseError(format!(
                "selector ({:?}) returned no results",
                selector,
            ))),
        }
    }
}

crate::children_from_impl!(VoxelData, ElementRef, elements);

impl FromStr for VoxelData {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let html = Html::parse_document(s);
        if html.errors.len() == 0 {
            return Self::try_from(html);
        }
        Err(Error::VoxelDataParseError(format!(
            "received ({}) html parsing error(s):\n{:?}",
            html.errors.len(),
            html.errors
                .iter()
                .enumerate()
                .map(|(i, e)| format!("[{}]{}", i + 1, e))
                .collect::<Vec<String>>()
                .join("\n"),
        )))
    }
}

impl TryFrom<Html> for VoxelData {
    type Error = Error;

    fn try_from(value: Html) -> Result<Self, Self::Error> {
        match value.select(&Selector::parse("body").unwrap()).next() {
            Some(body) => Self::try_from(body),
            None => Err(Error::VoxelDataParseError(
                "a <body> element is required".to_owned(),
            )),
        }
    }
}

impl TryFrom<ElementRef<'_>> for VoxelData {
    type Error = Error;

    fn try_from(value: ElementRef) -> Result<Self, Self::Error> {
        let mut vd = Self::new(VoxelTagName::from_str(value.value().name())?);

        vd.root = VoxelElement::try_from(value.value())?;

        vd.children_from(value)?;
        return Ok(vd);
    }
}

impl TryFrom<PathBuf> for VoxelData {
    type Error = Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        Self::try_from_file_with_selector(value, &Selector::parse("body").unwrap())
    }
}

#[macro_export]
macro_rules! children_from_impl {
    ($t:tt, $v:tt, $prop:ident) => {
        impl $t {
            fn children_from(&mut self, value: $v) -> Result<(), Error> {
                for node in value.children() {
                    if let Node::Text(_) = node.value() {
                        // ignore all text nodes without returning an error
                        continue;
                    }

                    self.$prop.push(VoxelElement::try_from(node)?);
                }

                Ok(())
            }
        }
    };
}
