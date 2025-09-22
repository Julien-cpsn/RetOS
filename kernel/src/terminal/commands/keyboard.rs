use goolog::{debug, trace};
use no_std_clap_macros::EnumValuesArg;
use pc_keyboard::{HandleControl, Keyboard, ScancodeSet1};
use pc_keyboard::layouts::{AnyLayout, Azerty, Colemak, DVP104Key, De105Key, Dvorak104Key, FiSe105Key, Jis109Key, No105Key, Uk105Key, Us104Key};
use strum::{EnumString, VariantNames};
use crate::interrupts::idt::KEYBOARD;
use crate::terminal::error::CliError;

const GOOLOG_TARGET: &str = "KEYBOARD";

#[derive(Default, EnumValuesArg, VariantNames, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum KeyboardLayout {
    #[default]
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

pub fn change_layout(layout: KeyboardLayout) -> Result<(), CliError> {
    trace!("KEYBOARD");
    
    let layout = match layout {
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