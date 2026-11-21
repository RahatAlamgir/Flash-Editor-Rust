//! Charts (C# `ChartObject`, `ChartPalettes`, `ChartData`): the typed view of a `"kind": "chart"` description.

use serde_json::{Map, Value};

use super::json::{self, number};
use crate::imaging::ColorRgba;

/// The kinds of chart, numbered like the C# `ChartKind`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChartKind {
    #[default]
    Columns,
    Bars,
    Line,
    Pie,
    Donut,
    Area,
    Lollipop,
    HalfDonut,
    RadialBars,
    Progress,
    Funnel,
}

impl ChartKind {
    pub const ALL: [ChartKind; 11] = [
        ChartKind::Columns,
        ChartKind::Bars,
        ChartKind::Line,
        ChartKind::Pie,
        ChartKind::Donut,
        ChartKind::Area,
        ChartKind::Lollipop,
        ChartKind::HalfDonut,
        ChartKind::RadialBars,
        ChartKind::Progress,
        ChartKind::Funnel,
    ];

    pub fn from_number(n: u64) -> Self {
        Self::ALL.get(n as usize).copied().unwrap_or_default()
    }

    pub fn number(self) -> u64 {
        Self::ALL.iter().position(|k| *k == self).unwrap_or(0) as u64
    }

    pub fn name(self) -> &'static str {
        match self {
            ChartKind::Columns => "Columns",
            ChartKind::Bars => "Bars",
            ChartKind::Line => "Line",
            ChartKind::Pie => "Pie",
            ChartKind::Donut => "Donut",
            ChartKind::Area => "Area",
            ChartKind::Lollipop => "Lollipop",
            ChartKind::HalfDonut => "Half donut",
            ChartKind::RadialBars => "Radial bars",
            ChartKind::Progress => "Progress bars",
            ChartKind::Funnel => "Funnel",
        }
    }
}

/// One value of a chart with its name.
#[derive(Clone, Debug, PartialEq)]
pub struct ChartItem {
    pub label: String,
    pub value: f64,
}

impl ChartItem {
    pub fn new(label: &str, value: f64) -> Self {
        Self { label: label.to_owned(), value }
    }
}

fn c(r: u8, g: u8, b: u8) -> ColorRgba {
    ColorRgba::rgb(r, g, b)
}

/// Ready-made color sets for charts (the first five are the C# `ChartPalettes`).
pub fn palettes() -> Vec<(&'static str, Vec<ColorRgba>)> {
    vec![
        ("Ocean", ocean()),
        (
            "Sunset",
            vec![c(239, 68, 68), c(249, 115, 22), c(245, 158, 11), c(236, 72, 153), c(217, 70, 239), c(250, 204, 21)],
        ),
        (
            "Forest",
            vec![c(22, 163, 74), c(132, 204, 22), c(13, 148, 136), c(101, 163, 13), c(5, 150, 105), c(163, 230, 53)],
        ),
        (
            "Candy",
            vec![
                c(244, 114, 182),
                c(167, 139, 250),
                c(96, 165, 250),
                c(52, 211, 153),
                c(251, 191, 36),
                c(251, 113, 133),
            ],
        ),
        ("Mono", vec![c(30, 41, 59), c(71, 85, 105), c(100, 116, 139), c(148, 163, 184), c(203, 213, 225)]),
    ]
}

pub fn ocean() -> Vec<ColorRgba> {
    vec![c(59, 130, 246), c(14, 165, 233), c(6, 182, 212), c(20, 184, 166), c(99, 102, 241), c(139, 92, 246)]
}

/// Everything a chart description says (C# `ChartObject` without the box and the looks).
#[derive(Clone, Debug, PartialEq)]
pub struct ChartData {
    pub kind: ChartKind,
    pub items: Vec<ChartItem>,
    pub colors: Vec<ColorRgba>,
    pub title: String,
    pub show_values: bool,
    pub show_labels: bool,
    pub show_grid: bool,
    pub background: Option<ColorRgba>,
    pub text_color: Option<ColorRgba>,
    /// How much of its space a bar takes (0.2 = thin bars, 1 = bars touching).
    pub bar_width: f64,
    pub value_prefix: String,
    pub value_suffix: String,
    /// Size of all text in the chart (1 = normal).
    pub text_scale: f64,
}

impl ChartData {
    /// A chart with sample numbers, to start from.
    pub fn sample(kind: ChartKind) -> Self {
        let items = [("Jan", 12.0), ("Feb", 19.0), ("Mar", 8.0), ("Apr", 15.0), ("May", 22.0)]
            .iter()
            .map(|(l, v)| ChartItem::new(l, *v))
            .collect();
        Self {
            kind,
            items,
            colors: ocean(),
            title: "Sales".into(),
            show_values: true,
            show_labels: true,
            show_grid: true,
            background: None,
            text_color: None,
            bar_width: 0.62,
            value_prefix: String::new(),
            value_suffix: String::new(),
            text_scale: 1.0,
        }
    }

    /// The color of the text in the chart (dark slate when nothing was chosen).
    pub fn ink(&self) -> ColorRgba {
        self.text_color.unwrap_or(ColorRgba::rgb(30, 41, 59))
    }

    /// The color of the `index`-th bar or slice.
    pub fn color_of(&self, index: usize) -> ColorRgba {
        if self.colors.is_empty() {
            let o = ocean();
            o[index % o.len()]
        } else {
            self.colors[index % self.colors.len()]
        }
    }

    /// A number as it is written in the chart (with the prefix and suffix).
    pub fn format(&self, value: f64) -> String {
        let body = if value.abs() >= 1000.0 { group_thousands(value) } else { trim_number(value) };
        format!("{}{}{}", self.value_prefix, body, self.value_suffix)
    }

    pub fn from_json(map: &Map<String, Value>) -> Option<Self> {
        let items = map
            .get("Items")?
            .as_array()?
            .iter()
            .filter_map(|i| {
                Some(ChartItem { label: i.get("Label")?.as_str()?.to_owned(), value: i.get("Value")?.as_f64()? })
            })
            .collect();
        let colors = map.get("Colors")?.as_array()?.iter().filter_map(json::color).collect();
        let flag = |name: &str| map.get(name).and_then(Value::as_bool).unwrap_or(true);
        Some(Self {
            kind: ChartKind::from_number(json::f64_of(map, "Kind", 0.0) as u64),
            items,
            colors,
            title: json::str_of(map, "Title").unwrap_or("").to_owned(),
            show_values: flag("ShowValues"),
            show_labels: flag("ShowLabels"),
            show_grid: flag("ShowGrid"),
            background: json::color_of(map, "Background"),
            text_color: json::color_of(map, "TextColor"),
            bar_width: json::f64_of(map, "BarWidth", 0.62),
            value_prefix: json::str_of(map, "ValuePrefix").unwrap_or("").to_owned(),
            value_suffix: json::str_of(map, "ValueSuffix").unwrap_or("").to_owned(),
            text_scale: json::f64_of(map, "TextScale", 1.0),
        })
    }

    /// Writes the chart's fields into a description (the box and the looks stay).
    pub fn write(&self, map: &mut Map<String, Value>) {
        json::set(map, "Kind", Some(self.kind.number().into()));
        let items = self
            .items
            .iter()
            .map(|i| {
                let mut m = Map::new();
                m.insert("Label".into(), i.label.clone().into());
                m.insert("Value".into(), number(i.value));
                Value::Object(m)
            })
            .collect();
        json::set(map, "Items", Some(Value::Array(items)));
        json::set(map, "Colors", Some(Value::Array(self.colors.iter().map(|c| json::color_value(*c)).collect())));
        json::set(map, "Title", Some(self.title.clone().into()));
        json::set(map, "ShowValues", Some(self.show_values.into()));
        json::set(map, "ShowLabels", Some(self.show_labels.into()));
        json::set(map, "ShowGrid", Some(self.show_grid.into()));
        json::set(map, "Background", self.background.map(json::color_value));
        json::set(map, "TextColor", self.text_color.map(json::color_value));
        json::set(map, "BarWidth", Some(number(self.bar_width)));
        json::set(map, "ValuePrefix", Some(self.value_prefix.clone().into()));
        json::set(map, "ValueSuffix", Some(self.value_suffix.clone().into()));
        json::set(map, "TextScale", Some(number(self.text_scale)));
    }

    /// The layer name: the title, shortened, or "Chart".
    pub fn layer_name(&self) -> String {
        let t = self.title.trim();
        if t.is_empty() {
            "Chart".into()
        } else if t.chars().count() > 24 {
            format!("{}...", t.chars().take(24).collect::<String>())
        } else {
            t.to_owned()
        }
    }
}

/// "12", "12.5", "12.34": at most two decimals, none for whole numbers (C# `0.##`).
fn trim_number(v: f64) -> String {
    let s = format!("{:.2}", (v * 100.0).round() / 100.0);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" || s.is_empty() { "0".into() } else { s.to_owned() }
}

/// "1,234.5" (C# `#,0.##`).
fn group_thousands(v: f64) -> String {
    let s = trim_number(v);
    let (sign, rest) = s.strip_prefix('-').map_or(("", s.as_str()), |r| ("-", r));
    let (whole, frac) = rest.split_once('.').map_or((rest, None), |(w, f)| (w, Some(f)));
    let mut out = String::new();
    for (i, ch) in whole.chars().enumerate() {
        if i > 0 && (whole.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    match frac {
        Some(f) => format!("{sign}{out}.{f}"),
        None => format!("{sign}{out}"),
    }
}

/// Turns the text a user types ("Jan, 12" per line) into chart values: a name and a number separated by a comma,
/// semicolon, colon or tab. Lines without a number are skipped.
pub fn parse_items(text: &str) -> Vec<ChartItem> {
    let mut items = Vec::new();
    for raw in text.replace('\r', "").split('\n') {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let cut = line.rfind([',', ';', ':', '\t']);
        let mut label = cut.map_or("", |i| line[..i].trim()).to_owned();
        let number: String = cut.map_or(line, |i| line[i + 1..].trim()).replace(' ', "");
        if cut.is_none() && number.parse::<f64>().is_ok() {
            label.clear();
        }
        let Ok(value) = number.parse::<f64>() else { continue };
        if !value.is_finite() {
            continue;
        }
        let label = if label.is_empty() { (items.len() + 1).to_string() } else { label };
        items.push(ChartItem { label, value });
        if items.len() >= 60 {
            break;
        }
    }
    items
}

/// The items as text, one per line ("Jan, 12").
pub fn format_items(items: &[ChartItem]) -> String {
    items.iter().map(|i| format!("{}, {}", i.label, trim_number(i.value))).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_are_written_like_the_csharp_version() {
        let mut c = ChartData::sample(ChartKind::Columns);
        assert_eq!(c.format(12.0), "12");
        assert_eq!(c.format(12.345), "12.35");
        assert_eq!(c.format(1234567.5), "1,234,567.5");
        c.value_prefix = "$".into();
        c.value_suffix = " kg".into();
        assert_eq!(c.format(-3.5), "$-3.5 kg");
    }

    #[test]
    fn typed_lines_become_items() {
        let items = parse_items("Jan, 12\nFeb; 19.5\n\n7\nbad line\nMar:8");
        let got: Vec<(String, f64)> = items.iter().map(|i| (i.label.clone(), i.value)).collect();
        assert_eq!(got, vec![("Jan".into(), 12.0), ("Feb".into(), 19.5), ("3".into(), 7.0), ("Mar".into(), 8.0)]);
        assert_eq!(format_items(&items[..2]), "Jan, 12\nFeb, 19.5");
    }

    #[test]
    fn a_chart_survives_json() {
        let mut map = Map::new();
        let chart = ChartData::sample(ChartKind::Donut);
        chart.write(&mut map);
        assert_eq!(ChartData::from_json(&map), Some(chart));
    }
}
