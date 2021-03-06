use crate::{
    widget::{
        unit::{WidgetUnit, WidgetUnitData},
        utils::{Rect, Vec2},
        WidgetId,
    },
    Scalar,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ContentBoxItemLayout {
    #[serde(default)]
    pub anchors: Rect,
    #[serde(default)]
    pub margin: Rect,
    #[serde(default)]
    pub align: Vec2,
    #[serde(default)]
    pub offset: Vec2,
    #[serde(default)]
    pub depth: Scalar,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ContentBoxItem {
    #[serde(default)]
    pub slot: WidgetUnit,
    #[serde(default)]
    pub layout: ContentBoxItemLayout,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ContentBox {
    #[serde(default)]
    pub id: WidgetId,
    #[serde(default)]
    pub items: Vec<ContentBoxItem>,
    #[serde(default)]
    pub clipping: bool,
}

impl WidgetUnitData for ContentBox {
    fn id(&self) -> &WidgetId {
        &self.id
    }

    fn get_children<'a>(&'a self) -> Vec<&'a WidgetUnit> {
        self.items.iter().map(|item| &item.slot).collect()
    }
}
