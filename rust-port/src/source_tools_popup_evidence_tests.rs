//! Scoped original CPU ImGui evidence checks; settled popup frame only.
use super::{GuiMetrics, UiViewport};
use crate::source_tools_layout::Layout;
use bevy::prelude::{Rect, UVec2, Vec2};
use serde_json::Value;

fn assert_rect(actual: Rect, expected: Rect, context: &str) {
    const TOLERANCE: f32 = 0.0002; // One 720-world/source-pixel round trip.
    assert!((actual.min - expected.min).abs().max_element() < TOLERANCE,
        "{context} minimum: actual {actual:?}, original {expected:?}");
    assert!((actual.max - expected.max).abs().max_element() < TOLERANCE,
        "{context} maximum: actual {actual:?}, original {expected:?}");
}

fn pair(value: &Value) -> Vec2 {
    Vec2::new(value[0].as_f64().unwrap() as f32, value[1].as_f64().unwrap() as f32)
}

fn check_original_capture(json: &str, count: usize) {
    let capture: Value = serde_json::from_str(json).unwrap();
    assert_eq!(capture["count"].as_u64(), Some(count as u64));
    assert_eq!(pair(&capture["displaySize"]), Vec2::new(1001.0, 701.0));
    assert_eq!(capture["guiScale"].as_f64(), Some(1.25));
    assert_eq!(capture["fontGlobalScale"].as_f64(), Some(1.0));
    let gui = GuiMetrics::new(1.25, [1001, 701], [1001, 701]);
    let viewport = UiViewport::new(Vec2::new(1001.0, 701.0),
        Vec2::new(800.8, 560.8), UVec2::new(1001, 701)).unwrap();
    let layout = Layout::new(gui, viewport);
    let source_rect = |rect: Rect| Rect::from_corners(
        viewport.world_to_source(rect.min), viewport.world_to_source(rect.max));
    let frames = capture["frames"].as_array().unwrap();
    let settled: Vec<_> = frames.iter().filter(|frame| frame["frame"].as_u64() == Some(6)).collect();
    let popup = settled.iter().find(|frame| frame["item"].as_str() == Some("popup")).unwrap();
    let expected_popup = Rect::from_corners(pair(&popup["windowPos"]),
        pair(&popup["windowPos"]) + pair(&popup["windowSize"]));
    assert_rect(source_rect(layout.popup(count)), expected_popup, "popup window");
    let metadata = capture["popupMetadata"].as_array().unwrap().iter()
        .find(|frame| frame["frame"].as_u64() == Some(6)).unwrap();
    let clip = metadata["innerClipRect"].as_array().unwrap();
    let expected_clip = Rect::from_corners(
        Vec2::new(clip[0].as_f64().unwrap() as f32, clip[1].as_f64().unwrap() as f32),
        Vec2::new(clip[2].as_f64().unwrap() as f32, clip[3].as_f64().unwrap() as f32));
    assert_rect(source_rect(crate::source_tools_popup::inner_clip(&layout, count)),
        expected_clip, "popup inner clip");
    assert_eq!(layout.scroll_max(count), metadata["scrollMax"][1].as_f64().unwrap() as f32);
    assert_eq!(metadata["scroll"][1].as_f64(), Some(0.0), "Capture uses unscrolled popup");

    let mut checked = vec![false; count];
    for frame in settled {
        let Some(index) = frame["item"].as_str().unwrap().strip_prefix("row") else { continue };
        let index: usize = index.parse().unwrap();
        assert!(index < count && !checked[index], "Each original selectable must occur once");
        checked[index] = true;
        assert_rect(source_rect(layout.option(index, count)),
            Rect::from_corners(pair(&frame["itemMin"]), pair(&frame["itemMax"])),
            &format!("{count}-item popup row {index}"));
    }
    assert!(checked.into_iter().all(|checked| checked), "Every selectable rectangle must match original execution");
}

#[test]
fn original_two_item_combo_popup_rectangles_match() {
    check_original_capture(include_str!("../tools/fixtures/source-tools-popup-2-capture.json"), 2);
}
#[test]
fn original_eight_item_combo_popup_rectangles_match() {
    check_original_capture(include_str!("../tools/fixtures/source-tools-popup-8-capture.json"), 8);
}
#[test]
fn original_nine_item_combo_popup_rectangles_match() {
    check_original_capture(include_str!("../tools/fixtures/source-tools-popup-9-capture.json"), 9);
}
#[test]
fn original_twenty_item_combo_popup_rectangles_match() {
    check_original_capture(include_str!("../tools/fixtures/source-tools-popup-20-capture.json"), 20);
}
