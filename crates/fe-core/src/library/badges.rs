//! Ready-made labels and badges (SALE, NEW, 5 stars, speech bubble...) made of shapes and text. Each one is a group,
//! so it can be ungrouped and changed (C# `Badges`).

use super::maker::{Maker, c};
use crate::imaging::ColorRgba;
use crate::objects::{ShapeKind::*, StoredObject};

pub struct BadgeInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
}

pub const CATEGORIES: [&str; 4] = ["Sale and deals", "Awards and quality", "Social", "Info and steps"];

const fn b(id: &'static str, name: &'static str, category: &'static str) -> BadgeInfo {
    BadgeInfo { id, name, category }
}

const SALE: &str = "Sale and deals";
const AWARD: &str = "Awards and quality";
const SOCIAL: &str = "Social";
const INFO: &str = "Info and steps";

/// The 52 badges, in the order of the picker.
pub const ALL: [BadgeInfo; 52] = [
    b("sale", "SALE", SALE),
    b("off", "50% OFF", SALE),
    b("hot", "HOT", SALE),
    b("free", "FREE", SALE),
    b("price", "Price", SALE),
    b("arrival", "New arrival", SALE),
    b("limited", "Limited", SALE),
    b("bigsale", "Big sale", SALE),
    b("clearance", "Clearance", SALE),
    b("bogo", "Buy 1 get 1", SALE),
    b("freeship", "Free shipping", SALE),
    b("ordernow", "Order now", SALE),
    b("hotdeal", "Hot deal", SALE),
    b("soldout", "Sold out", SALE),
    b("coming", "Coming soon", SALE),
    b("pct", "-30%", SALE),
    b("best", "Best seller", AWARD),
    b("verified", "Verified", AWARD),
    b("stars", "5 stars", AWARD),
    b("shield", "Shield", AWARD),
    b("hundred", "100%", AWARD),
    b("lightning", "Lightning", AWARD),
    b("bestchoice", "Best choice", AWARD),
    b("toprated", "Top rated", AWARD),
    b("premium", "Premium", AWARD),
    b("number1", "Number 1", AWARD),
    b("guarantee", "Guarantee", AWARD),
    b("winner", "Winner", AWARD),
    b("certified", "Certified", AWARD),
    b("eco", "Eco", AWARD),
    b("organic", "Organic", AWARD),
    b("handmade", "Handmade", AWARD),
    b("rated", "Rating 4.9", AWARD),
    b("new", "NEW", SOCIAL),
    b("thanks", "Thank you", SOCIAL),
    b("hello", "Speech bubble", SOCIAL),
    b("swipe", "Swipe", SOCIAL),
    b("like", "Likes", SOCIAL),
    b("subscribe", "Subscribe", SOCIAL),
    b("follow", "Follow", SOCIAL),
    b("live", "Live", SOCIAL),
    b("newpost", "New post", SOCIAL),
    b("trending", "Trending", SOCIAL),
    b("tip", "Tip", INFO),
    b("note", "Note", INFO),
    b("important", "Important", INFO),
    b("info", "Info", INFO),
    b("question", "Question", INFO),
    b("step1", "Step 1", INFO),
    b("step2", "Step 2", INFO),
    b("step3", "Step 3", INFO),
    b("done", "Done", INFO),
];

/// The display name of a badge.
pub fn name_of(id: &str) -> &'static str {
    ALL.iter().find(|i| i.id == id).map_or("Badge", |i| i.name)
}

/// The badge, centered on a canvas, about `size` pixels wide. `None` for an unknown id.
pub fn build(id: &str, canvas_w: f64, canvas_h: f64, size: f64) -> Option<StoredObject> {
    let white: ColorRgba = c(255, 255, 255);
    let dark: ColorRgba = c(17, 24, 39);
    let mut m = Maker::new(canvas_w, canvas_h, size, 100.0);
    match id {
        "sale" => {
            m.shape("Burst", Burst, 0, 0, 100, 100).fill(c(225, 29, 72));
            m.badge_text("SALE", 50, 50, 24, white);
        }
        "new" => {
            m.shape("Circle", Ellipse, 0, 0, 100, 100).fill(c(37, 99, 235));
            m.badge_text("NEW", 50, 50, 27, white);
        }
        "hot" => {
            m.shape("Burst", Burst, 0, 0, 100, 100).fill(c(249, 115, 22));
            m.badge_text("HOT", 50, 50, 27, white);
        }
        "off" => {
            m.shape("Burst", Burst, 0, 0, 100, 100).fill(c(250, 204, 21));
            m.badge_text("50%", 50, 42, 27, dark);
            m.badge_text("OFF", 50, 66, 17, dark);
        }
        "free" => {
            m.shape("Pill", Rectangle, 0, 25, 100, 50).fill(c(22, 163, 74)).radius(14);
            m.badge_text("FREE", 50, 50, 25, white);
        }
        "best" => {
            m.shape("Tag", Tag, 0, 10, 100, 80).fill(c(124, 58, 237));
            m.badge_text("BEST", 50, 50, 20, white);
            m.badge_text("SELLER", 50, 66, 11, white);
        }
        "limited" => {
            m.shape("Frame", Rectangle, 0, 30, 100, 40).fill(white).stroke(dark, 3);
            m.badge_text("LIMITED", 50, 50, 17, dark);
        }
        "thanks" => {
            m.shape("Heart", Heart, 0, 5, 100, 90).fill(c(239, 68, 68));
            m.badge_text("THANKS", 50, 44, 14, white);
        }
        "verified" => {
            m.shape("Circle", Ellipse, 0, 0, 100, 100).fill(c(34, 197, 94));
            m.shape("Check", Check, 25, 28, 50, 44).fill(white);
        }
        "stars" => {
            for i in 0..5 {
                m.shape(&format!("Star {}", i + 1), Star, i * 20, 40, 20, 20).fill(c(250, 204, 21));
            }
        }
        "hello" => {
            m.shape("Bubble", SpeechBubble, 0, 10, 100, 70).fill(white).stroke(dark, 3).radius(10);
            m.badge_text("Hello!", 50, 42, 19, dark);
        }
        "price" => {
            m.shape("Pill", Rectangle, 0, 25, 100, 50).fill(c(14, 165, 233)).radius(25);
            m.badge_text("$9.99", 50, 50, 24, white);
        }
        "swipe" => {
            m.shape("Pill", Rectangle, 0, 30, 100, 40).fill(dark).radius(20);
            m.badge_text("SWIPE", 40, 50, 15, white);
            m.shape("Arrow", Chevron, 68, 40, 20, 20).fill(white);
        }
        "shield" => {
            m.shape("Shield", Shield, 10, 0, 80, 100).fill(c(37, 99, 235));
            m.shape("Check", Check, 30, 28, 40, 38).fill(white);
        }
        "hundred" => {
            m.shape("Ring", Ellipse, 4, 4, 92, 92).stroke(c(225, 29, 72), 7);
            m.badge_text("100%", 50, 50, 23, c(225, 29, 72));
        }
        "lightning" => {
            m.shape("Circle", Ellipse, 0, 0, 100, 100).fill(c(250, 204, 21));
            m.shape("Bolt", Lightning, 28, 14, 44, 72).fill(dark);
        }
        "arrival" => {
            m.shape("Banner", Rectangle, 0, 35, 100, 30).fill(c(225, 29, 72));
            m.badge_text("NEW ARRIVAL", 50, 50, 11.5, white);
        }
        "bigsale" => {
            m.shape("Burst", Burst, 0, 0, 100, 100).fill(c(190, 18, 60));
            m.badge_text("BIG", 50, 40, 20, white);
            m.badge_text("SALE", 50, 62, 22, c(253, 224, 71));
        }
        "clearance" => {
            m.shape("Banner", Rectangle, 0, 28, 100, 44).fill(c(220, 38, 38)).radius(6);
            m.badge_text("CLEARANCE", 50, 50, 13, white);
        }
        "bogo" => {
            m.shape("Burst", Burst, 0, 0, 100, 100).fill(c(124, 58, 237));
            m.badge_text("BUY 1", 50, 40, 17, white);
            m.badge_text("GET 1", 50, 62, 17, c(253, 224, 71));
        }
        "freeship" => {
            m.shape("Pill", Rectangle, 0, 28, 100, 44).fill(c(22, 163, 74)).radius(22);
            m.badge_text("FREE SHIPPING", 50, 50, 10.5, white);
        }
        "ordernow" => {
            m.shape("Pill", Rectangle, 0, 28, 100, 44).fill(c(37, 99, 235)).radius(22);
            m.badge_text("ORDER NOW", 42, 50, 12, white);
            m.shape("Chevron", Chevron, 76, 41, 14, 18).fill(white);
        }
        "hotdeal" => {
            m.shape("Tag", Tag, 0, 12, 100, 76).fill(c(249, 115, 22));
            m.badge_text("HOT", 52, 42, 22, white);
            m.badge_text("DEAL", 52, 62, 16, c(120, 53, 15));
        }
        "soldout" => {
            m.shape("Frame", Rectangle, 0, 28, 100, 44).fill(white).stroke(c(220, 38, 38), 4).radius(6);
            m.badge_text("SOLD OUT", 50, 50, 15, c(220, 38, 38));
        }
        "coming" => {
            m.shape("Frame", Rectangle, 0, 28, 100, 44).fill(dark).radius(10);
            m.badge_text("COMING", 50, 42, 13, white);
            m.badge_text("SOON", 50, 60, 13, c(250, 204, 21));
        }
        "pct" => {
            m.shape("Circle", Ellipse, 0, 0, 100, 100).fill(c(225, 29, 72));
            m.badge_text("-30%", 50, 50, 26, white);
        }
        "bestchoice" => {
            m.shape("Burst", Burst, 0, 0, 100, 100).fill(c(234, 179, 8));
            m.shape("Star", Star, 30, 14, 40, 40).fill(white);
            m.badge_text("BEST", 50, 66, 15, white);
            m.badge_text("CHOICE", 50, 82, 9, white);
        }
        "toprated" => {
            m.shape("Circle", Ellipse, 0, 0, 100, 100).fill(c(217, 119, 6));
            m.shape("Ring", Ellipse, 8, 8, 84, 84).stroke(c(254, 243, 199), 3);
            m.shape("Star", Star, 36, 14, 28, 28).fill(c(254, 243, 199));
            m.badge_text("TOP", 50, 56, 17, white);
            m.badge_text("RATED", 50, 75, 11, white);
        }
        "premium" => {
            m.shape("Shield", Shield, 10, 0, 80, 100).fill(c(88, 28, 135));
            m.shape("Star", Star, 36, 14, 28, 28).fill(c(250, 204, 21));
            m.badge_text("PREMIUM", 50, 62, 10.5, white);
        }
        "number1" => {
            m.shape("Circle", Ellipse, 0, 0, 100, 100).fill(c(250, 204, 21));
            m.shape("Ring", Ellipse, 7, 7, 86, 86).stroke(c(161, 98, 7), 3);
            m.badge_text("#1", 50, 50, 38, c(113, 63, 18));
        }
        "guarantee" => {
            m.shape("Circle", Ellipse, 0, 0, 100, 100).fill(c(21, 128, 61));
            m.shape("Ring", Ellipse, 7, 7, 86, 86).stroke(white, 2);
            m.badge_text("100%", 50, 40, 22, white);
            m.badge_text("GUARANTEE", 50, 64, 8.5, white);
        }
        "winner" => {
            m.shape("Star", Star, 0, 0, 100, 100).fill(c(250, 204, 21));
            m.badge_text("WINNER", 50, 56, 11, c(113, 63, 18));
        }
        "certified" => {
            m.shape("Burst", Burst, 0, 0, 100, 100).fill(c(37, 99, 235));
            m.shape("Check", Check, 28, 30, 44, 40).fill(white);
        }
        "eco" => {
            m.shape("Drop", Teardrop, 10, 6, 80, 88).fill(c(34, 197, 94));
            m.badge_text("ECO", 50, 58, 20, white);
        }
        "organic" => {
            m.shape("Circle", Ellipse, 0, 0, 100, 100).fill(c(101, 163, 13));
            m.badge_text("100%", 50, 40, 18, white);
            m.badge_text("ORGANIC", 50, 62, 11, white);
        }
        "handmade" => {
            m.shape("Heart", Heart, 0, 4, 100, 92).fill(c(236, 72, 153));
            m.badge_text("HAND", 50, 40, 13, white);
            m.badge_text("MADE", 50, 56, 13, white);
        }
        "rated" => {
            m.shape("Pill", Rectangle, 0, 26, 100, 48).fill(dark).radius(24);
            m.shape("Star", Star, 10, 36, 28, 28).fill(c(250, 204, 21));
            m.badge_text("4.9", 66, 50, 20, white);
        }
        "like" => {
            m.shape("Pill", Rectangle, 0, 26, 100, 48).fill(c(254, 226, 226)).radius(24);
            m.shape("Heart", Heart, 10, 36, 28, 26).fill(c(239, 68, 68));
            m.badge_text("1.2K", 66, 50, 18, c(153, 27, 27));
        }
        "subscribe" => {
            m.shape("Pill", Rectangle, 0, 28, 100, 44).fill(c(220, 38, 38)).radius(8);
            m.shape("Play", RightTriangle, 9, 38, 14, 24).fill(white);
            m.badge_text("SUBSCRIBE", 60, 50, 11, white);
        }
        "follow" => {
            m.shape("Pill", Rectangle, 0, 28, 100, 44).fill(c(59, 130, 246)).radius(22);
            m.badge_text("+ FOLLOW", 50, 50, 14, white);
        }
        "live" => {
            m.shape("Pill", Rectangle, 0, 28, 100, 44).fill(c(220, 38, 38)).radius(8);
            m.shape("Dot", Ellipse, 12, 42, 16, 16).fill(white);
            m.badge_text("LIVE", 62, 50, 20, white);
        }
        "newpost" => {
            m.shape("Bubble", SpeechBubble, 0, 8, 100, 72).fill(c(37, 99, 235)).radius(10);
            m.badge_text("NEW", 50, 34, 16, white);
            m.badge_text("POST", 50, 54, 16, white);
        }
        "trending" => {
            m.shape("Pill", Rectangle, 0, 28, 100, 44).fill(c(249, 115, 22)).radius(22);
            m.shape("Bolt", Lightning, 8, 36, 16, 28).fill(c(254, 240, 138));
            m.badge_text("TRENDING", 58, 50, 10.5, white);
        }
        "tip" => {
            m.shape("Bubble", SpeechBubble, 0, 8, 100, 72).fill(c(250, 204, 21)).radius(10);
            m.badge_text("TIP", 50, 42, 26, c(113, 63, 18));
        }
        "note" => {
            m.shape("Note", Rectangle, 4, 4, 92, 92).fill(c(254, 240, 138)).radius(4);
            m.badge_text("NOTE", 50, 50, 22, c(113, 63, 18));
        }
        "important" => {
            m.shape("Warn", Triangle, 4, 6, 92, 84).fill(c(220, 38, 38));
            m.badge_text("!", 50, 66, 40, white);
        }
        "info" => {
            m.shape("Circle", Ellipse, 0, 0, 100, 100).fill(c(14, 165, 233));
            m.badge_text("i", 50, 50, 56, white);
        }
        "question" => {
            m.shape("Circle", Ellipse, 0, 0, 100, 100).fill(c(124, 58, 237));
            m.badge_text("?", 50, 50, 56, white);
        }
        "step1" => {
            m.shape("Circle", Ellipse, 0, 0, 100, 100).fill(c(37, 99, 235));
            m.badge_text("1", 50, 50, 56, white);
        }
        "step2" => {
            m.shape("Circle", Ellipse, 0, 0, 100, 100).fill(c(22, 163, 74));
            m.badge_text("2", 50, 50, 56, white);
        }
        "step3" => {
            m.shape("Circle", Ellipse, 0, 0, 100, 100).fill(c(249, 115, 22));
            m.badge_text("3", 50, 50, 56, white);
        }
        "done" => {
            m.shape("Box", Rectangle, 4, 4, 92, 92).fill(c(22, 163, 74)).radius(18);
            m.shape("Check", Check, 24, 28, 52, 44).fill(white);
        }
        _ => return None,
    }
    m.done()
}
