use bevy::math::{IRect, IVec2};


pub trait FromBevyIRect {
    fn from_bevy_irect(rect: IRect) -> Self;
}
impl FromBevyIRect for uiautomation::types::Rect {
    fn from_bevy_irect(rect: IRect) -> Self {
        uiautomation::types::Rect::new(rect.min.x, rect.min.y, rect.max.x, rect.max.y)
    }
}

pub trait IntoBevyIRect {
    fn to_bevy_irect(&self) -> IRect;
}

impl IntoBevyIRect for uiautomation::types::Rect {
    fn to_bevy_irect(&self) -> IRect {
        IRect::new(
            self.get_left(),
            self.get_top(),
            self.get_right(),
            self.get_bottom(),
        )
    }
}

pub trait IntoUiRect {
    fn to_ui_irect(&self) -> uiautomation::types::Rect;
}
impl IntoUiRect for IRect {
    fn to_ui_irect(&self) -> uiautomation::types::Rect {
        uiautomation::types::Rect::new(self.min.x, self.min.y, self.max.x, self.max.y)
    }
}

pub trait IntoUiPoint {
    fn to_ui_point(&self) -> uiautomation::types::Point;
}
impl IntoUiPoint for IVec2 {
    fn to_ui_point(&self) -> uiautomation::types::Point {
        uiautomation::types::Point::new(self.x, self.y)
    }
}

pub trait IntoBevyIVec2 {
    fn to_bevy_ivec2(&self) -> IVec2;
}
impl IntoBevyIVec2 for uiautomation::types::Point {
    fn to_bevy_ivec2(&self) -> IVec2 {
        IVec2::new(self.get_x(), self.get_y())
    }
}
