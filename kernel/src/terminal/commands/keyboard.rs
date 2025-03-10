use goolog::{debug, set_target};
use pc_keyboard::{HandleControl, Keyboard, ScancodeSet1};
use pc_keyboard::layouts::{AnyLayout, Azerty, Colemak, DVP104Key, De105Key, Dvorak104Key, FiSe105Key, Jis109Key, No105Key, Uk105Key, Us104Key};
use strum::{EnumString, VariantNames};
use crate::arg_from_enum;
use crate::interrupts::idt::KEYBOARD;
use crate::terminal::error::CliError;


#[derive(VariantNames, EnumString)]
#[strum(serialize_all = "lowercase")]
enum KeyboardLayout {
    Azerty,
    Colemak,
    De,
    Dvp,
    Dvorak,
    FiSe,
    Jis,
    No,
    Uk,
    Us,
}

arg_from_enum!(KeyboardLayout);

pub fn change_layout(layout: KeyboardLayoutArg) -> Result<(), CliError> {
    set_target!("KEYBOARD");

    let layout = match layout.0 {
        KeyboardLayout::Azerty => AnyLayout::Azerty(Azerty),
        KeyboardLayout::Colemak => AnyLayout::Colemak(Colemak),
        KeyboardLayout::De => AnyLayout::De105Key(De105Key),
        KeyboardLayout::Dvp => AnyLayout::DVP104Key(DVP104Key),
        KeyboardLayout::Dvorak => AnyLayout::Dvorak104Key(Dvorak104Key),
        KeyboardLayout::FiSe => AnyLayout::FiSe105Key(FiSe105Key),
        KeyboardLayout::Jis => AnyLayout::Jis109Key(Jis109Key),
        KeyboardLayout::No => AnyLayout::No105Key(No105Key),
        KeyboardLayout::Uk => AnyLayout::Uk105Key(Uk105Key),
        KeyboardLayout::Us => AnyLayout::Us104Key(Us104Key)
    };
    
    debug!("Locking and setting KEYBOARD mutex...");
    *KEYBOARD.write() = Keyboard::new(ScancodeSet1::new(), layout, HandleControl::Ignore);
    debug!("KEYBOARD mutex set and freed");

    Ok(())
}