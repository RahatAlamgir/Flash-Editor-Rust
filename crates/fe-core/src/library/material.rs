//! The Google (Material) icons: about 2,200 everyday icons (home, search, settings, share, mail...) from Google's Material Icons
//! font, which is built into the program. Like the Windows icons they are inserted as editable text in the font "Material Icons",
//! so their color, size, outline and shadow can be changed. The names come from the font's own list of icon names.

use std::sync::OnceLock;

/// The font file (Apache License 2.0, see `assets/fonts/README.md`).
pub const FONT: &[u8] = include_bytes!("../../../../assets/fonts/MaterialIcons-Regular.ttf");

/// The name of the font family.
pub const FONT_FAMILY: &str = "Material Icons";

const CODEPOINTS: &str = include_str!("../../../../assets/fonts/MaterialIcons-Regular.codepoints");

/// One Google icon.
pub struct MaterialIcon {
    /// "add_circle": the name the font uses (words joined with `_`).
    pub key: &'static str,
    /// "Add circle": for people.
    pub name: String,
    pub code: u32,
}

impl MaterialIcon {
    /// The icon as the one character of the font.
    pub fn glyph(&self) -> String {
        char::from_u32(self.code).map(String::from).unwrap_or_default()
    }
}

/// Every icon, in the order of the font's list (alphabetical).
pub fn all() -> &'static [MaterialIcon] {
    static LIST: OnceLock<Vec<MaterialIcon>> = OnceLock::new();
    LIST.get_or_init(|| {
        CODEPOINTS
            .lines()
            .filter_map(|line| {
                let (key, code) = line.split_once(' ')?;
                let code = u32::from_str_radix(code.trim(), 16).ok()?;
                let words = key.replace('_', " ");
                let mut chars = words.chars();
                let name = chars.next().map(|f| f.to_uppercase().collect::<String>() + chars.as_str())?;
                Some(MaterialIcon { key, name, code })
            })
            .collect()
    })
}

/// The icons most people look for first, in a sensible order (names missing from the font are skipped).
pub fn popular() -> Vec<&'static MaterialIcon> {
    const KEYS: &[&str] = &[
        "home",
        "search",
        "settings",
        "menu",
        "close",
        "add",
        "remove",
        "check",
        "done",
        "done_all",
        "favorite",
        "favorite_border",
        "star",
        "star_border",
        "share",
        "mail",
        "email",
        "call",
        "phone",
        "chat",
        "message",
        "person",
        "group",
        "account_circle",
        "notifications",
        "shopping_cart",
        "shopping_bag",
        "store",
        "delete",
        "edit",
        "create",
        "content_copy",
        "content_paste",
        "content_cut",
        "save",
        "download",
        "file_download",
        "upload",
        "file_upload",
        "cloud",
        "cloud_upload",
        "cloud_download",
        "lock",
        "lock_open",
        "visibility",
        "visibility_off",
        "info",
        "warning",
        "error",
        "help",
        "thumb_up",
        "thumb_down",
        "location_on",
        "place",
        "map",
        "navigation",
        "schedule",
        "access_time",
        "event",
        "calendar_today",
        "date_range",
        "photo_camera",
        "image",
        "photo_library",
        "videocam",
        "movie",
        "mic",
        "play_arrow",
        "play_circle",
        "pause",
        "stop",
        "skip_next",
        "skip_previous",
        "fast_forward",
        "fast_rewind",
        "volume_up",
        "volume_off",
        "music_note",
        "headset",
        "language",
        "public",
        "wifi",
        "bluetooth",
        "battery_full",
        "flight",
        "directions_car",
        "directions_bus",
        "directions_bike",
        "directions_walk",
        "train",
        "local_shipping",
        "restaurant",
        "local_cafe",
        "local_bar",
        "local_pizza",
        "cake",
        "fastfood",
        "school",
        "work",
        "business",
        "build",
        "code",
        "bug_report",
        "lightbulb",
        "rocket_launch",
        "emoji_events",
        "emoji_emotions",
        "mood",
        "sentiment_satisfied",
        "sentiment_dissatisfied",
        "pets",
        "eco",
        "park",
        "wb_sunny",
        "nights_stay",
        "cloud_queue",
        "umbrella",
        "ac_unit",
        "water_drop",
        "local_fire_department",
        "bolt",
        "flash_on",
        "arrow_forward",
        "arrow_back",
        "arrow_upward",
        "arrow_downward",
        "expand_more",
        "expand_less",
        "chevron_right",
        "chevron_left",
        "refresh",
        "sync",
        "autorenew",
        "undo",
        "redo",
        "open_in_new",
        "link",
        "attach_file",
        "bookmark",
        "label",
        "flag",
        "folder",
        "folder_open",
        "description",
        "article",
        "list",
        "view_list",
        "grid_view",
        "dashboard",
        "bar_chart",
        "pie_chart",
        "show_chart",
        "trending_up",
        "trending_down",
        "analytics",
        "payments",
        "credit_card",
        "paid",
        "savings",
        "account_balance",
        "receipt",
        "qr_code",
        "print",
        "fingerprint",
        "verified",
        "security",
        "shield",
        "key",
        "tune",
        "filter_list",
        "sort",
        "more_horiz",
        "more_vert",
        "apps",
        "fullscreen",
        "zoom_in",
        "zoom_out",
        "palette",
        "brush",
        "format_paint",
        "draw",
        "text_fields",
        "format_bold",
        "format_italic",
        "format_underlined",
        "format_align_left",
        "format_align_center",
        "format_align_right",
        "format_list_bulleted",
        "gif",
        "support_agent",
        "headphones",
        "gamepad",
        "sports_esports",
        "sports_soccer",
        "fitness_center",
        "spa",
        "hotel",
        "luggage",
        "celebration",
        "redeem",
        "card_giftcard",
        "handshake",
    ];
    let list = all();
    KEYS.iter().filter_map(|k| list.iter().find(|i| i.key == *k)).collect()
}

/// Every word of the search must be in the icon's name.
pub fn matches(query: &str, icon: &MaterialIcon) -> bool {
    let key = icon.key.replace('_', " ");
    query.split_whitespace().all(|w| key.contains(&w.to_lowercase()))
}
