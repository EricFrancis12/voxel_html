use crate::errors::Error;
use ego_tree::NodeRef;
use scraper::{ElementRef, Html, Node, Selector};
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
    VXStyle,
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
            "vx_style" => Ok(Self::VXStyle),
            _ => Err(Error::VoxelAttributeNameParseError(format!(
                "unknown attribute: {}",
                s,
            ))),
        }
    }
}

enum VXStyleName {
    XLength,
    YLength,
    ZLength,
}

impl FromStr for VXStyleName {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "x-length" => Ok(Self::XLength),
            "y-length" => Ok(Self::YLength),
            "z-length" => Ok(Self::ZLength),
            _ => Err(Error::VXStyleNameParseError(format!(
                "unknown vx style name: {}",
                s
            ))),
        }
    }
}

#[derive(Debug, Default)]
struct VXStyle {
    x_length: f32,
    y_length: f32,
    z_length: f32,
}

#[derive(Debug)]
struct VoxelElement {
    tag: VoxelTagName,
    vx_style: VXStyle,
    children: Vec<VoxelElement>,
}

impl<'a> VoxelElement {
    fn new(tag: VoxelTagName) -> Self {
        Self {
            tag,
            vx_style: VXStyle::default(),
            children: Vec::new(),
        }
    }
}

type NR<'a> = NodeRef<'a, Node>;

crate::children_from_impl!(VoxelElement, NR, children);

impl<'a> TryFrom<NodeRef<'a, Node>> for VoxelElement {
    type Error = Error;

    fn try_from(value: NodeRef<'a, Node>) -> Result<Self, Self::Error> {
        let node = value.value();

        if let Node::Element(element) = node {
            let mut ve = Self::new(VoxelTagName::from_str(element.name())?);

            for (key, val) in element.attrs() {
                if let Ok(name) = VoxelAttributeName::from_dataset_str(key) {
                    match name {
                        VoxelAttributeName::VXStyle => {
                            let pairs = val
                                .split(";")
                                .into_iter()
                                .map(|s| s.trim())
                                .collect::<Vec<&str>>();

                            for pair in pairs {
                                if let [k, v] =
                                    pair.split(":").map(|s| s.trim()).collect::<Vec<&str>>()[..]
                                {
                                    if let Ok(sn) = VXStyleName::from_str(k) {
                                        // TODO: refactor:
                                        let l = v.parse::<f32>().unwrap_or_default();
                                        if l < 0.0 {
                                            continue;
                                        }

                                        match sn {
                                            VXStyleName::XLength => ve.vx_style.x_length = l,
                                            VXStyleName::YLength => ve.vx_style.y_length = l,
                                            VXStyleName::ZLength => ve.vx_style.z_length = l,
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

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
    root: VoxelElement,
    elements: Vec<VoxelElement>,
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
