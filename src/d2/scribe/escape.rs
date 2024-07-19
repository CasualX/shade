/*!

Some ideas for escape sequences inside text strings:
\e[color=#fff]
\e[outline=#000]
\e[letter_spacing=1.5]
\e[letter_spacing=normal]
\e[font_size=*1.5]

 */

use super::*;

pub(crate) fn process(s: &str, initial: &Scribe, scribe: &mut Scribe, cv: Option<&mut TextBuffer>) {
	let Some((key, value)) = s.split_once('=') else { return };
	let Some((value, _)) = value.split_once(']') else { return };
}
