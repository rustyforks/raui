use crate::{
    layout::{Layout, LayoutEngine, LayoutItem, LayoutNode},
    widget::{
        unit::{
            content::ContentBox,
            flex::FlexBox,
            grid::GridBox,
            image::{ImageBox, ImageBoxSizeValue},
            size::{SizeBox, SizeBoxSizeValue},
            text::{TextBox, TextBoxSizeValue},
            WidgetUnit,
        },
        utils::{lerp, Rect, Vec2},
        WidgetId,
    },
    Scalar,
};
use std::collections::HashMap;

#[derive(Debug, Default, Copy, Clone)]
pub struct DefaultLayoutEngine;

impl DefaultLayoutEngine {
    pub fn layout_node(size_available: Vec2, unit: &WidgetUnit) -> Option<LayoutNode> {
        match unit {
            WidgetUnit::ContentBox(b) => Some(Self::layout_content_box(size_available, b)),
            WidgetUnit::FlexBox(b) => Some(Self::layout_flex_box(size_available, b)),
            WidgetUnit::GridBox(b) => Self::layout_grid_box(size_available, b),
            WidgetUnit::SizeBox(b) => Some(Self::layout_size_box(size_available, b)),
            WidgetUnit::ImageBox(b) => Some(Self::layout_image_box(size_available, b)),
            WidgetUnit::TextBox(b) => Some(Self::layout_text_box(size_available, b)),
            _ => None,
        }
    }

    pub fn layout_content_box(size_available: Vec2, unit: &ContentBox) -> LayoutNode {
        let children = unit
            .items
            .iter()
            .filter_map(|item| {
                let left = lerp(0.0, size_available.x, item.layout.anchors.left);
                let left = left + item.layout.margin.left + item.layout.offset.x;
                let right = lerp(0.0, size_available.x, item.layout.anchors.right);
                let right = right - item.layout.margin.right + item.layout.offset.x;
                let top = lerp(0.0, size_available.y, item.layout.anchors.top);
                let top = top + item.layout.margin.top + item.layout.offset.y;
                let bottom = lerp(0.0, size_available.y, item.layout.anchors.bottom);
                let bottom = bottom - item.layout.margin.bottom + item.layout.offset.y;
                let width = (right - left).max(0.0);
                let height = (bottom - top).max(0.0);
                let size = Vec2 {
                    x: width,
                    y: height,
                };
                if let Some(mut child) = Self::layout_node(size, &item.slot) {
                    let diff = child.local_space.width() - width;
                    let ox = lerp(0.0, diff, item.layout.align.x);
                    child.local_space.left += left - ox;
                    child.local_space.right += left - ox;
                    let diff = child.local_space.height() - height;
                    let oy = lerp(0.0, diff, item.layout.align.y);
                    child.local_space.top += top - oy;
                    child.local_space.bottom += top - oy;
                    Some(child)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        LayoutNode {
            id: unit.id.to_owned(),
            local_space: Rect {
                left: 0.0,
                right: size_available.x,
                top: 0.0,
                bottom: size_available.y,
            },
            children,
        }
    }

    pub fn layout_flex_box(size_available: Vec2, unit: &FlexBox) -> LayoutNode {
        if unit.wrap {
            Self::layout_flex_box_wrapping(size_available, unit)
        } else {
            Self::layout_flex_box_no_wrap(size_available, unit)
        }
    }

    pub fn layout_flex_box_wrapping(size_available: Vec2, unit: &FlexBox) -> LayoutNode {
        let main_available = if unit.direction.is_horizontal() {
            size_available.x
        } else {
            size_available.y
        };
        let (lines, count) = {
            let mut main = 0.0;
            let mut cross: Scalar = 0.0;
            let mut grow = 0.0;
            let items = unit
                .items
                .iter()
                .filter(|item| item.slot.is_some())
                .collect::<Vec<_>>();
            let count = items.len();
            let mut lines = vec![];
            let mut line = vec![];
            for item in items {
                let local_main = item.basis.unwrap_or_else(|| {
                    if unit.direction.is_horizontal() {
                        Self::calc_unit_min_width(&item.slot)
                    } else {
                        Self::calc_unit_min_height(&item.slot)
                    }
                });
                let local_main = local_main
                    + if unit.direction.is_horizontal() {
                        item.margin.left + item.margin.right
                    } else {
                        item.margin.top + item.margin.bottom
                    };
                let local_cross = if unit.direction.is_horizontal() {
                    Self::calc_unit_min_height(&item.slot)
                } else {
                    Self::calc_unit_min_width(&item.slot)
                };
                let local_cross = local_cross
                    + if unit.direction.is_horizontal() {
                        item.margin.top + item.margin.bottom
                    } else {
                        item.margin.left + item.margin.right
                    };
                if !line.is_empty() && main + local_main > main_available {
                    main += line.len().checked_sub(1).unwrap_or(0) as Scalar * unit.separation;
                    lines.push((main, cross, grow, std::mem::replace(&mut line, vec![])));
                    main = 0.0;
                    cross = 0.0;
                    grow = 0.0;
                }
                main += local_main;
                cross = cross.max(local_cross);
                grow += item.grow;
                line.push((item, local_main, local_cross));
            }
            main += line.len().checked_sub(1).unwrap_or(0) as Scalar * unit.separation;
            lines.push((main, cross, grow, line));
            (lines, count)
        };
        let mut children = Vec::with_capacity(count);
        let mut main_max: Scalar = 0.0;
        let mut cross_max = 0.0;
        for (main, cross_available, grow, items) in lines {
            let diff = main_available - main;
            let mut new_main = 0.0;
            let mut new_cross: Scalar = 0.0;
            for (item, local_main, local_cross) in items {
                let child_main = if main < main_available {
                    local_main
                        + if grow > 0.0 {
                            diff * item.grow / grow
                        } else {
                            0.0
                        }
                } else {
                    local_main
                };
                let child_main = (child_main
                    - if unit.direction.is_horizontal() {
                        item.margin.left + item.margin.right
                    } else {
                        item.margin.top + item.margin.bottom
                    })
                .max(0.0);
                let child_cross = (local_cross
                    - if unit.direction.is_horizontal() {
                        item.margin.top + item.margin.bottom
                    } else {
                        item.margin.left + item.margin.right
                    })
                .max(0.0);
                let child_cross = lerp(child_cross, cross_available, item.fill);
                let rect = if unit.direction.is_horizontal() {
                    Vec2 {
                        x: child_main,
                        y: child_cross,
                    }
                } else {
                    Vec2 {
                        x: child_cross,
                        y: child_main,
                    }
                };
                if let Some(mut child) = Self::layout_node(rect, &item.slot) {
                    if unit.direction.is_horizontal() {
                        if unit.direction.is_order_ascending() {
                            child.local_space.left += new_main + item.margin.left;
                            child.local_space.right += new_main + item.margin.left;
                        } else {
                            let left = child.local_space.left;
                            let right = child.local_space.right;
                            child.local_space.left =
                                size_available.x - right - new_main - item.margin.right;
                            child.local_space.right =
                                size_available.x - left - new_main - item.margin.right;
                        }
                        new_main += rect.x;
                        let diff = lerp(
                            0.0,
                            cross_available - child.local_space.height(),
                            item.align,
                        );
                        child.local_space.top += cross_max + item.margin.top + diff;
                        child.local_space.bottom += cross_max + item.margin.top + diff;
                        new_cross = new_cross.max(rect.y);
                    } else {
                        if unit.direction.is_order_ascending() {
                            child.local_space.top += new_main + item.margin.top;
                            child.local_space.bottom += new_main + item.margin.top;
                        } else {
                            let top = child.local_space.top;
                            let bottom = child.local_space.bottom;
                            child.local_space.top =
                                size_available.y - bottom - new_main - item.margin.bottom;
                            child.local_space.bottom =
                                size_available.y - top - new_main - item.margin.bottom;
                        }
                        new_main += rect.y;
                        let diff =
                            lerp(0.0, cross_available - child.local_space.width(), item.align);
                        child.local_space.left += cross_max + item.margin.left + diff;
                        child.local_space.right += cross_max + item.margin.left + diff;
                        new_cross = new_cross.max(rect.x);
                    }
                    new_main += unit.separation;
                    children.push(child);
                }
            }
            new_main = (new_main - unit.separation).max(0.0);
            main_max = main_max.max(new_main);
            cross_max += new_cross + unit.separation;
        }
        cross_max = (cross_max - unit.separation).max(0.0);
        let local_space = if unit.direction.is_horizontal() {
            Rect {
                left: 0.0,
                right: main_max,
                top: 0.0,
                bottom: cross_max,
            }
        } else {
            Rect {
                left: 0.0,
                right: cross_max,
                top: 0.0,
                bottom: main_max,
            }
        };
        LayoutNode {
            id: unit.id.to_owned(),
            local_space,
            children,
        }
    }

    pub fn layout_flex_box_no_wrap(size_available: Vec2, unit: &FlexBox) -> LayoutNode {
        let (main_available, cross_available) = if unit.direction.is_horizontal() {
            (size_available.x, size_available.y)
        } else {
            (size_available.y, size_available.x)
        };
        let mut main = 0.0;
        let mut cross: Scalar = 0.0;
        let mut grow = 0.0;
        let mut shrink = 0.0;
        let items = unit
            .items
            .iter()
            .filter(|item| item.slot.is_some())
            .collect::<Vec<_>>();
        let axis_sizes = items
            .iter()
            .map(|item| {
                let local_main = item.basis.unwrap_or_else(|| {
                    if unit.direction.is_horizontal() {
                        Self::calc_unit_min_width(&item.slot)
                    } else {
                        Self::calc_unit_min_height(&item.slot)
                    }
                });
                let local_main = local_main
                    + if unit.direction.is_horizontal() {
                        item.margin.left + item.margin.right
                    } else {
                        item.margin.top + item.margin.bottom
                    };
                let local_cross = if unit.direction.is_horizontal() {
                    Self::calc_unit_min_height(&item.slot)
                } else {
                    Self::calc_unit_min_width(&item.slot)
                };
                let local_cross = local_cross
                    + if unit.direction.is_horizontal() {
                        item.margin.top + item.margin.bottom
                    } else {
                        item.margin.left + item.margin.right
                    };
                let local_cross = lerp(local_cross, cross_available, item.fill);
                main += local_main;
                cross = cross.max(local_cross);
                grow += item.grow;
                shrink += item.shrink;
                (local_main, local_cross)
            })
            .collect::<Vec<_>>();
        main += items.len().checked_sub(1).unwrap_or(0) as Scalar * unit.separation;
        let diff = main_available - main;
        let mut new_main = 0.0;
        let mut new_cross: Scalar = 0.0;
        let children = items
            .into_iter()
            .zip(axis_sizes.into_iter())
            .filter_map(|(item, axis_size)| {
                let child_main = if main < main_available {
                    axis_size.0
                        + if grow > 0.0 {
                            diff * item.grow / grow
                        } else {
                            0.0
                        }
                } else if main > main_available {
                    axis_size.0
                        + if shrink > 0.0 {
                            diff * item.shrink / shrink
                        } else {
                            0.0
                        }
                } else {
                    axis_size.0
                };
                let child_main = (child_main
                    - if unit.direction.is_horizontal() {
                        item.margin.left + item.margin.right
                    } else {
                        item.margin.top + item.margin.bottom
                    })
                .max(0.0);
                let child_cross = (axis_size.1
                    - if unit.direction.is_horizontal() {
                        item.margin.top + item.margin.bottom
                    } else {
                        item.margin.left + item.margin.right
                    })
                .max(0.0);
                let rect = if unit.direction.is_horizontal() {
                    Vec2 {
                        x: child_main,
                        y: child_cross,
                    }
                } else {
                    Vec2 {
                        x: child_cross,
                        y: child_main,
                    }
                };
                if let Some(mut child) = Self::layout_node(rect, &item.slot) {
                    if unit.direction.is_horizontal() {
                        if unit.direction.is_order_ascending() {
                            child.local_space.left += new_main + item.margin.left;
                            child.local_space.right += new_main + item.margin.left;
                        } else {
                            let left = child.local_space.left;
                            let right = child.local_space.right;
                            child.local_space.left =
                                size_available.x - right - new_main - item.margin.right;
                            child.local_space.right =
                                size_available.x - left - new_main - item.margin.right;
                        }
                        new_main += rect.x;
                        let diff = lerp(
                            0.0,
                            cross_available - child.local_space.height(),
                            item.align,
                        );
                        child.local_space.top += item.margin.top + diff;
                        child.local_space.bottom += item.margin.top + diff;
                        new_cross = new_cross.max(rect.y);
                    } else {
                        if unit.direction.is_order_ascending() {
                            child.local_space.top += new_main + item.margin.top;
                            child.local_space.bottom += new_main + item.margin.top;
                        } else {
                            let top = child.local_space.top;
                            let bottom = child.local_space.bottom;
                            child.local_space.top =
                                size_available.y - bottom - new_main - item.margin.bottom;
                            child.local_space.bottom =
                                size_available.y - top - new_main - item.margin.bottom;
                        }
                        new_main += rect.y;
                        let diff =
                            lerp(0.0, cross_available - child.local_space.width(), item.align);
                        child.local_space.left += item.margin.left + diff;
                        child.local_space.right += item.margin.left + diff;
                        new_cross = new_cross.max(rect.x);
                    }
                    new_main += unit.separation;
                    Some(child)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        new_main = (new_main - unit.separation).max(0.0);
        let local_space = if unit.direction.is_horizontal() {
            Rect {
                left: 0.0,
                right: new_main,
                top: 0.0,
                bottom: new_cross,
            }
        } else {
            Rect {
                left: 0.0,
                right: new_cross,
                top: 0.0,
                bottom: new_main,
            }
        };
        LayoutNode {
            id: unit.id.to_owned(),
            local_space,
            children,
        }
    }

    pub fn layout_grid_box(size_available: Vec2, unit: &GridBox) -> Option<LayoutNode> {
        if unit.cols == 0 || unit.rows == 0 {
            return None;
        }

        let cell_width = size_available.x / unit.cols as Scalar;
        let cell_height = size_available.y / unit.rows as Scalar;
        let children = unit
            .items
            .iter()
            .filter_map(|item| {
                let left = item.space_occupancy.left as Scalar * cell_width;
                let right = item.space_occupancy.right as Scalar * cell_width;
                let top = item.space_occupancy.top as Scalar * cell_height;
                let bottom = item.space_occupancy.bottom as Scalar * cell_height;
                let width = (right - left - item.margin.left - item.margin.right).max(0.0);
                let height = (bottom - top - item.margin.top - item.margin.bottom).max(0.0);
                let size = Vec2 {
                    x: width,
                    y: height,
                };
                if let Some(mut child) = Self::layout_node(size, &item.slot) {
                    let diff = size.x - child.local_space.width();
                    let ox = lerp(0.0, diff, item.horizontal_align);
                    let diff = size.y - child.local_space.height();
                    let oy = lerp(0.0, diff, item.vertical_align);
                    child.local_space.left += left + item.margin.left - ox;
                    child.local_space.right += left + item.margin.left - ox;
                    child.local_space.top += top + item.margin.top - oy;
                    child.local_space.bottom += top + item.margin.top - oy;
                    Some(child)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        Some(LayoutNode {
            id: unit.id.to_owned(),
            local_space: Rect {
                left: 0.0,
                right: size_available.x,
                top: 0.0,
                bottom: size_available.y,
            },
            children,
        })
    }

    pub fn layout_size_box(size_available: Vec2, unit: &SizeBox) -> LayoutNode {
        let size = Vec2 {
            x: (size_available.x - unit.margin.left - unit.margin.right).max(0.0),
            y: (size_available.y - unit.margin.top - unit.margin.bottom).max(0.0),
        };
        let (size, children) = if let Some(mut child) = Self::layout_node(size, &unit.slot) {
            child.local_space.left += unit.margin.left;
            child.local_space.right += unit.margin.left;
            child.local_space.top += unit.margin.top;
            child.local_space.bottom += unit.margin.top;
            (child.local_space.size(), vec![child])
        } else {
            (Default::default(), vec![])
        };
        let local_space = Rect {
            left: 0.0,
            right: match unit.width {
                SizeBoxSizeValue::Content => size.x,
                SizeBoxSizeValue::Fill => size_available.x,
                SizeBoxSizeValue::Exact(v) => v,
            },
            top: 0.0,
            bottom: match unit.height {
                SizeBoxSizeValue::Content => size.y,
                SizeBoxSizeValue::Fill => size_available.y,
                SizeBoxSizeValue::Exact(v) => v,
            },
        };
        LayoutNode {
            id: unit.id.to_owned(),
            local_space,
            children,
        }
    }

    pub fn layout_image_box(size_available: Vec2, unit: &ImageBox) -> LayoutNode {
        let local_space = Rect {
            left: 0.0,
            right: match unit.width {
                ImageBoxSizeValue::Fill => size_available.x,
                ImageBoxSizeValue::Exact(v) => v,
            },
            top: 0.0,
            bottom: match unit.height {
                ImageBoxSizeValue::Fill => size_available.y,
                ImageBoxSizeValue::Exact(v) => v,
            },
        };
        LayoutNode {
            id: unit.id.to_owned(),
            local_space,
            children: vec![],
        }
    }

    pub fn layout_text_box(size_available: Vec2, unit: &TextBox) -> LayoutNode {
        let local_space = Rect {
            left: 0.0,
            right: match unit.width {
                TextBoxSizeValue::Fill => size_available.x,
                TextBoxSizeValue::Exact(v) => v,
            },
            top: 0.0,
            bottom: match unit.height {
                TextBoxSizeValue::Fill => size_available.y,
                TextBoxSizeValue::Exact(v) => v,
            },
        };
        LayoutNode {
            id: unit.id.to_owned(),
            local_space,
            children: vec![],
        }
    }

    fn calc_unit_min_width(unit: &WidgetUnit) -> Scalar {
        match unit {
            WidgetUnit::ImageBox(b) => match b.width {
                ImageBoxSizeValue::Fill => 0.0,
                ImageBoxSizeValue::Exact(v) => v,
            },
            WidgetUnit::TextBox(b) => match b.width {
                TextBoxSizeValue::Fill => 0.0,
                TextBoxSizeValue::Exact(v) => v,
            },
            WidgetUnit::SizeBox(b) => {
                b.margin.top
                    + b.margin.bottom
                    + match b.width {
                        SizeBoxSizeValue::Content => Self::calc_unit_min_width(&b.slot),
                        SizeBoxSizeValue::Fill => 0.0,
                        SizeBoxSizeValue::Exact(v) => v,
                    }
            }
            _ => 0.0,
        }
    }

    fn calc_unit_min_height(unit: &WidgetUnit) -> Scalar {
        match unit {
            WidgetUnit::ImageBox(b) => match b.height {
                ImageBoxSizeValue::Fill => 0.0,
                ImageBoxSizeValue::Exact(v) => v,
            },
            WidgetUnit::TextBox(b) => match b.height {
                TextBoxSizeValue::Fill => 0.0,
                TextBoxSizeValue::Exact(v) => v,
            },
            WidgetUnit::SizeBox(b) => {
                b.margin.top
                    + b.margin.bottom
                    + match b.height {
                        SizeBoxSizeValue::Content => Self::calc_unit_min_height(&b.slot),
                        SizeBoxSizeValue::Fill => 0.0,
                        SizeBoxSizeValue::Exact(v) => v,
                    }
            }
            _ => 0.0,
        }
    }

    fn unpack_node(ui_space: Rect, node: LayoutNode, items: &mut HashMap<WidgetId, LayoutItem>) {
        let LayoutNode {
            id,
            local_space,
            children,
        } = node;
        let ui_space = Rect {
            left: local_space.left + ui_space.left,
            right: local_space.right + ui_space.left,
            top: local_space.top + ui_space.top,
            bottom: local_space.bottom + ui_space.top,
        };
        for node in children {
            Self::unpack_node(ui_space, node, items);
        }
        items.insert(
            id,
            LayoutItem {
                local_space,
                ui_space,
            },
        );
    }
}

impl LayoutEngine<()> for DefaultLayoutEngine {
    fn layout(&mut self, ui_space: Rect, tree: &WidgetUnit) -> Result<Layout, ()> {
        if let Some(root) = Self::layout_node(ui_space.size(), tree) {
            let mut items = HashMap::with_capacity(root.count());
            Self::unpack_node(ui_space, root, &mut items);
            Ok(Layout { ui_space, items })
        } else {
            Ok(Layout {
                ui_space,
                items: Default::default(),
            })
        }
    }
}
