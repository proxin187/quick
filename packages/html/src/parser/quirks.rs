use crate::tokenizer::Doctype;

use unicase::UniCase;


const PUBLIC_ID_PREFIX: &[&str] = &[
    "+//silmaril//dtd html pro v0r11 19970101//",
    "-//as//dtd html 3.0 aswedit + extensions//",
    "-//advasoft ltd//dtd html 3.0 aswedit + extensions//",
    "-//ietf//dtd html 2.0 level 1//",
    "-//ietf//dtd html 2.0 level 2//",
    "-//ietf//dtd html 2.0 strict level 1//",
    "-//ietf//dtd html 2.0 strict level 2//",
    "-//ietf//dtd html 2.0 strict//",
    "-//ietf//dtd html 2.0//",
    "-//ietf//dtd html 2.1e//",
    "-//ietf//dtd html 3.0//",
    "-//ietf//dtd html 3.2 final//",
    "-//ietf//dtd html 3.2//",
    "-//ietf//dtd html 3//",
    "-//ietf//dtd html level 0//",
    "-//ietf//dtd html level 1//",
    "-//ietf//dtd html level 2//",
    "-//ietf//dtd html level 3//",
    "-//ietf//dtd html strict level 0//",
    "-//ietf//dtd html strict level 1//",
    "-//ietf//dtd html strict level 2//",
    "-//ietf//dtd html strict level 3//",
    "-//ietf//dtd html strict//",
    "-//ietf//dtd html//",
    "-//metrius//dtd metrius presentational//",
    "-//microsoft//dtd internet explorer 2.0 html strict//",
    "-//microsoft//dtd internet explorer 2.0 html//",
    "-//microsoft//dtd internet explorer 2.0 tables//",
    "-//microsoft//dtd internet explorer 3.0 html strict//",
    "-//microsoft//dtd internet explorer 3.0 html//",
    "-//microsoft//dtd internet explorer 3.0 tables//",
    "-//netscape comm. corp.//dtd html//",
    "-//netscape comm. corp.//dtd strict html//",
    "-//o'reilly and associates//dtd html 2.0//",
    "-//o'reilly and associates//dtd html extended 1.0//",
    "-//o'reilly and associates//dtd html extended relaxed 1.0//",
    "-//softquad software//dtd hotmetal pro 6.0::19990601::extensions to html 4.0//",
    "-//softquad//dtd hotmetal pro 4.0::19971010::extensions to html 4.0//",
    "-//spyglass//dtd html 2.0 extended//",
    "-//sq//dtd html 2.0 hotmetal + extensions//",
    "-//sun microsystems corp.//dtd hotjava html//",
    "-//sun microsystems corp.//dtd hotjava strict html//",
    "-//w3c//dtd html 3 1995-03-24//",
    "-//w3c//dtd html 3.2 draft//",
    "-//w3c//dtd html 3.2 final//",
    "-//w3c//dtd html 3.2//",
    "-//w3c//dtd html 3.2s draft//",
    "-//w3c//dtd html 4.0 frameset//",
    "-//w3c//dtd html 4.0 transitional//",
    "-//w3c//dtd html experimental 19960712//",
    "-//w3c//dtd html experimental 970421//",
    "-//w3c//dtd w3 html//",
    "-//w3o//dtd w3 html 3.0//",
    "-//webtechs//dtd mozilla html 2.0//",
    "-//webtechs//dtd mozilla html//",
];

const PUBLIC_ID_MATCH: &[&str] = &[
    "-//w3o//dtd w3 html strict 3.0//en//",
    "-/w3c/dtd html 4.0 transitional/en",
    "html",
];

const PUBLIC_ID_OTHER: &[&str] = &[
    "-//w3c//dtd html 4.01 frameset//",
    "-//w3c//dtd html 4.01 transitional//",
];

const SYSTEM_ID_MATCH: &str = "http://www.ibm.com/data/dtd/v11/ibmxhtml1-transitional.dtd";

pub enum QuirksMode {
    NoQuirks,
    Quirks,
    LimitedQuirks,
}

// TODO: implement tests for quirks
impl<'a> From<Doctype<'a>> for QuirksMode {
    fn from(doctype: Doctype<'a>) -> QuirksMode {
        match doctype {
            Doctype { name, .. } if name != Some(UniCase::new("html")) => QuirksMode::Quirks,
            Doctype { public_id, .. } if public_id.map(|string| PUBLIC_ID_MATCH.contains(&string)).unwrap_or_default() => QuirksMode::Quirks,
            Doctype { system_id, .. } if system_id == Some(UniCase::new(SYSTEM_ID_MATCH)) => QuirksMode::Quirks,
            Doctype { public_id, .. } if public_id.map(|string| PUBLIC_ID_PREFIX.iter().any(|prefix| string.starts_with(prefix))).unwrap_or_default() => QuirksMode::Quirks,
            Doctype { system_id, public_id, .. } if public_id.map(|string| PUBLIC_ID_OTHER.iter().any(|prefix| string.starts_with(prefix))).unwrap_or_default() => {
                system_id.is_none()
                    .then(|| QuirksMode::Quirks)
                    .unwrap_or(QuirksMode::LimitedQuirks)
            },
            _ => QuirksMode::NoQuirks,
        }
    }
}


