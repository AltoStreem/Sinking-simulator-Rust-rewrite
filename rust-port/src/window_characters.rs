//! Original GLFW character filtering and callback order for committed text.
use bevy::prelude::*;
#[derive(Message, Clone)]
pub(crate) struct CharacterInput(pub u32);
#[derive(Message, Clone)]
pub(crate) struct ResetShip;
pub(crate) fn units(codepoint: u32) -> Vec<u16> {
    if codepoint <= 0xffff {
        vec![codepoint as u16]
    } else {
        char::from_u32(codepoint)
            .map(|c| c.encode_utf16(&mut [0; 2]).to_vec())
            .unwrap_or_default()
    }
}
pub(crate) fn read_codepoints(
    messages: Option<&Messages<CharacterInput>>,
    cursor: &mut bevy::ecs::message::MessageCursor<CharacterInput>,
) -> Vec<u32> {
    messages
        .map(|messages| cursor.read(messages).map(|event| event.0).collect())
        .unwrap_or_default()
}
pub(crate) fn read_units(
    messages: Option<&Messages<CharacterInput>>,
    cursor: &mut bevy::ecs::message::MessageCursor<CharacterInput>,
) -> Vec<u16> {
    read_codepoints(messages, cursor)
        .into_iter()
        .flat_map(units)
        .collect()
}
pub(crate) fn read(
    messages: Option<&Messages<CharacterInput>>,
    cursor: &mut bevy::ecs::message::MessageCursor<CharacterInput>,
) -> String {
    String::from_utf16_lossy(&read_units(messages, cursor))
}
pub(crate) fn accepted(character: char) -> bool {
    let point = character as u32;
    point >= 32 && !(127..160).contains(&point)
}
pub(crate) fn dispatch(
    window: &crate::window::SourceWindow,
    text: &str,
    mods: i32,
    plain: bool,
    mut publish: impl FnMut(char),
) {
    for character in text.chars().filter(|&c| accepted(c)) {
        window.emit_char_mods(character as i32, mods & 15);
        if plain {
            window.emit_char(character as i32);
            publish(character);
        }
    }
}
pub(crate) fn native_packet(
    window: &crate::window::SourceWindow,
    packet: crate::window_native_characters::Packet,
    mut publish: impl FnMut(u32),
    mut reset: impl FnMut(),
) {
    window.emit_char_mods(packet.codepoint as i32, packet.mods);
    if packet.mods & 1 != 0 && matches!(packet.codepoint as u16, 82 | 114) {
        reset();
    }
    if packet.plain {
        window.emit_char(packet.codepoint as i32);
        let unit = packet.codepoint as u16;
        if !packet.composing && unit != 0 {
            publish(unit as u32);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};
    #[test]
    fn source_characters_filter_controls_and_dispatch_modifiers_before_plain_unicode() {
        let (window, _, _) = crate::window::tests::fixture();
        let log = Rc::new(RefCell::new(Vec::new()));
        let captured = log.clone();
        window
            .char_mods_callbacks
            .borrow_mut()
            .push(Rc::new(move |_, c, m| {
                captured.borrow_mut().push(format!("mods {c} {m}"))
            }));
        let captured = log.clone();
        window
            .char_callbacks
            .borrow_mut()
            .push(Rc::new(move |_, c| {
                captured.borrow_mut().push(format!("char {c}"))
            }));
        let mut output = String::new();
        dispatch(
            &window,
            "\u{1}\u{1f} A\u{7f}\u{9f}\u{a0}🚢",
            63,
            true,
            |c| output.push(c),
        );
        assert_eq!(output, " A\u{a0}🚢");
        assert_eq!(
            *log.borrow(),
            vec![
                "mods 32 15",
                "char 32",
                "mods 65 15",
                "char 65",
                "mods 160 15",
                "char 160",
                "mods 128674 15",
                "char 128674"
            ]
        );
        log.borrow_mut().clear();
        dispatch(&window, "x", 4, false, |_| {
            panic!("system character must not publish plain text")
        });
        assert_eq!(*log.borrow(), vec!["mods 120 4"]);
    }
    #[test]
    fn native_gui_characters_retain_units_cast_and_shift_reset_before_plain_input() {
        let (window, _, _) = crate::window::tests::fixture();
        let output = Rc::new(RefCell::new(Vec::new()));
        let saved = output.clone();
        window
            .char_mods_callbacks
            .borrow_mut()
            .push(Rc::new(move |_, c, m| {
                saved.borrow_mut().push(format!("mods {c} {m}"))
            }));
        let saved = output.clone();
        window
            .char_callbacks
            .borrow_mut()
            .push(Rc::new(move |_, c| {
                saved.borrow_mut().push(format!("char {c}"))
            }));
        let mut units = Vec::new();
        native_packet(
            &window,
            crate::window_native_characters::Packet {
                codepoint: 0x10052,
                mods: 1,
                plain: true,
                composing: false,
            },
            |c| units.push(c),
            || output.borrow_mut().push("reset".into()),
        );
        assert_eq!(
            *output.borrow(),
            vec!["mods 65618 1", "reset", "char 65618"]
        );
        assert_eq!(units, vec![82]); // Original JVM backend casts the callback int to char.
        output.borrow_mut().clear();
        units.clear();
        native_packet(
            &window,
            crate::window_native_characters::Packet {
                codepoint: 82,
                mods: 1,
                plain: false,
                composing: false,
            },
            |c| units.push(c),
            || output.borrow_mut().push("reset".into()),
        );
        assert_eq!(*output.borrow(), vec!["mods 82 1", "reset"]);
        assert!(units.is_empty());
        for codepoint in [0xd83d, 0xde80, 0xd83d] {
            native_packet(
                &window,
                crate::window_native_characters::Packet {
                    codepoint,
                    mods: 0,
                    plain: true,
                    composing: false,
                },
                |c| units.push(c),
                || panic!(),
            );
        }
        assert_eq!(units, vec![0xd83d, 0xde80, 0xd83d]);
        units.clear();
        native_packet(
            &window,
            crate::window_native_characters::Packet {
                codepoint: 65,
                mods: 0,
                plain: true,
                composing: true,
            },
            |c| units.push(c),
            || panic!(),
        );
        assert!(
            units.is_empty(),
            "The original GUI backend suppresses GLFW text during composition"
        );
        native_packet(
            &window,
            crate::window_native_characters::Packet {
                codepoint: 0x10000,
                mods: 0,
                plain: true,
                composing: false,
            },
            |c| units.push(c),
            || panic!(),
        );
        native_packet(
            &window,
            crate::window_native_characters::Packet {
                codepoint: 65,
                mods: 0,
                plain: true,
                composing: false,
            },
            |c| units.push(c),
            || panic!(),
        );
        assert_eq!(
            units,
            vec![65],
            "IO.addInputCharacter ignores a narrowed NUL without discarding later text"
        );
    }
}
