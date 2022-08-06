use crate::emath::NumExt;
use crate::epaint::{Color32, RectShape, Rounding, Shape, Stroke};

use super::{add_rulers_and_text, highlighted_color, Orientation, PlotConfig, RectElement};
use crate::plot::{ChartPlot, PlotPoint, ScreenTransform};

#[derive(Clone, Debug, PartialEq)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl Candle {
    pub fn new(open: f64, high: f64, low: f64, close: f64, volume: f64) -> Self {
        Self {
            open,
            high,
            low,
            close,
            volume,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CandleElem {
    pub x: f64,
    pub candle: Candle,
    pub candle_width: f64,
    pub whisker_width: f64,
    pub stroke: Stroke,
    pub fill: Color32,
}

impl CandleElem {
    pub fn new(candle: Candle) -> Self {
        Self {
            x: 0.0,
            candle,
            candle_width: 0.25,
            whisker_width: 0.15,
            stroke: Stroke::new(1.0, Color32::TRANSPARENT),
            fill: Color32::TRANSPARENT,
        }
    }

    /// Add a custom stroke.
    pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.stroke = stroke.into();
        self
    }

    /// Add a custom fill color.
    pub fn fill(mut self, color: impl Into<Color32>) -> Self {
        self.fill = color.into();
        self
    }

    /// Set the box width.
    pub fn candle_width(mut self, width: f64) -> Self {
        self.candle_width = width;
        self
    }

    /// Set the whisker width.
    pub fn whisker_width(mut self, width: f64) -> Self {
        self.whisker_width = width;
        self
    }

    pub(super) fn add_shapes(
        &self,
        transform: &ScreenTransform,
        highlighted: bool,
        shapes: &mut Vec<Shape>,
    ) {
        let (stroke, fill) = if highlighted {
            highlighted_color(self.stroke, self.fill)
        } else {
            (self.stroke, self.fill)
        };

        let rect = transform.rect_from_values(
            &self.point_at(self.x - self.candle_width / 2.0, self.candle.open),
            &self.point_at(self.x + self.candle_width / 2.0, self.candle.close),
        );

        let rect = Shape::Rect(RectShape {
            rect,
            rounding: Rounding::none(),
            fill,
            stroke,
        });
        shapes.push(rect);

        let line_between = |v1, v2| {
            Shape::line_segment(
                [
                    transform.position_from_point(&v1),
                    transform.position_from_point(&v2),
                ],
                stroke,
            )
        };

        let whisker = line_between(
            self.point_at(self.x, self.candle.low),
            self.point_at(self.x, self.candle.high),
        );
        shapes.push(whisker);
    }

    pub(super) fn add_rulers_and_text(
        &self,
        parent: &ChartPlot,
        plot: &PlotConfig<'_>,
        shapes: &mut Vec<Shape>,
    ) {
        let text: Option<String> = parent
            .element_formatter
            .as_ref()
            .map(|fmt| fmt(self, parent));

        add_rulers_and_text(self, plot, text, shapes);
    }
}

impl RectElement for CandleElem {
    fn name(&self) -> &str {
        ""
    }

    fn bounds_min(&self) -> PlotPoint {
        let x = self.x - self.candle_width.max(self.whisker_width) / 2.0;
        let value = self.candle.low;
        self.point_at(x, value)
    }

    fn bounds_max(&self) -> PlotPoint {
        let x = self.x + self.candle_width.max(self.whisker_width) / 2.0;
        let value = self.candle.high;
        self.point_at(x, value)
    }

    fn values_with_ruler(&self) -> Vec<PlotPoint> {
        let open = self.point_at(self.x, self.candle.open);
        let high = self.point_at(self.x, self.candle.high);
        let low = self.point_at(self.x, self.candle.low);
        let close = self.point_at(self.x, self.candle.close);
        let volume = self.point_at(self.x, self.candle.volume);

        vec![open, high, low, close, volume]
    }

    fn orientation(&self) -> Orientation {
        Orientation::Vertical
    }

    fn corner_value(&self) -> PlotPoint {
        self.point_at(self.x, self.candle.high)
    }

    fn default_values_format(&self, transform: &ScreenTransform) -> String {
        let scale = transform.dvalue_dpos();
        let y_decimals = ((-scale[1].abs().log10()).ceil().at_least(0.0) as usize).at_most(6);
        format!(
            "\nOpen = {open:.decimals$}\
             \nHigh = {high:.decimals$}\
             \nLow = {low:.decimals$}\
             \nClose = {close:.decimals$}\
             \nVolume = {volume:.decimals$}",
            open = self.candle.open,
            high = self.candle.high,
            low = self.candle.low,
            close = self.candle.close,
            volume = self.candle.volume,
            decimals = y_decimals
        )
    }
}
