//! `PhoneField`'s hand-kept country table: ISO code, English name, dial code.
//! No crate backs it and nothing here validates a number (f3-phone-field plan).

/// One row of the table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Country {
    /// ISO 3166-1 alpha-2, upper case.
    pub iso: &'static str,
    /// The English name, which the localization's `country_names` and
    /// `country_label` override per render.
    pub name: &'static str,
    /// The E.164 country calling code, digits only and no `+`.
    pub dial: &'static str,
}

const fn row(iso: &'static str, name: &'static str, dial: &'static str) -> Country {
    Country { iso, name, dial }
}

/// Sorted by name, the picker's order. Checked 2026-09-20 against libphonenumber
/// `806ee32e`; a NANP member with one area code carries it (`1684`).
pub(crate) static COUNTRIES: &[Country] = &[
    row("AF", "Afghanistan", "93"),
    row("AX", "Åland Islands", "358"),
    row("AL", "Albania", "355"),
    row("DZ", "Algeria", "213"),
    row("AS", "American Samoa", "1684"),
    row("AD", "Andorra", "376"),
    row("AO", "Angola", "244"),
    row("AI", "Anguilla", "1264"),
    row("AQ", "Antarctica", "672"),
    row("AG", "Antigua and Barbuda", "1268"),
    row("AR", "Argentina", "54"),
    row("AM", "Armenia", "374"),
    row("AW", "Aruba", "297"),
    row("AU", "Australia", "61"),
    row("AT", "Austria", "43"),
    row("AZ", "Azerbaijan", "994"),
    row("BS", "Bahamas", "1242"),
    row("BH", "Bahrain", "973"),
    row("BD", "Bangladesh", "880"),
    row("BB", "Barbados", "1246"),
    row("BY", "Belarus", "375"),
    row("BE", "Belgium", "32"),
    row("BZ", "Belize", "501"),
    row("BJ", "Benin", "229"),
    row("BM", "Bermuda", "1441"),
    row("BT", "Bhutan", "975"),
    row("BO", "Bolivia", "591"),
    row("BA", "Bosnia and Herzegovina", "387"),
    row("BW", "Botswana", "267"),
    row("BR", "Brazil", "55"),
    row("IO", "British Indian Ocean Territory", "246"),
    row("VG", "British Virgin Islands", "1284"),
    row("BN", "Brunei", "673"),
    row("BG", "Bulgaria", "359"),
    row("BF", "Burkina Faso", "226"),
    row("BI", "Burundi", "257"),
    row("KH", "Cambodia", "855"),
    row("CM", "Cameroon", "237"),
    row("CA", "Canada", "1"),
    row("CV", "Cape Verde", "238"),
    row("BQ", "Caribbean Netherlands", "599"),
    row("KY", "Cayman Islands", "1345"),
    row("CF", "Central African Republic", "236"),
    row("TD", "Chad", "235"),
    row("CL", "Chile", "56"),
    row("CN", "China", "86"),
    row("CX", "Christmas Island", "61"),
    row("CC", "Cocos (Keeling) Islands", "61"),
    row("CO", "Colombia", "57"),
    row("KM", "Comoros", "269"),
    row("CG", "Congo - Brazzaville", "242"),
    row("CD", "Congo - Kinshasa", "243"),
    row("CK", "Cook Islands", "682"),
    row("CR", "Costa Rica", "506"),
    row("CI", "Côte d'Ivoire", "225"),
    row("HR", "Croatia", "385"),
    row("CU", "Cuba", "53"),
    row("CW", "Curaçao", "599"),
    row("CY", "Cyprus", "357"),
    row("CZ", "Czechia", "420"),
    row("DK", "Denmark", "45"),
    row("DJ", "Djibouti", "253"),
    row("DM", "Dominica", "1767"),
    row("DO", "Dominican Republic", "1"),
    row("EC", "Ecuador", "593"),
    row("EG", "Egypt", "20"),
    row("SV", "El Salvador", "503"),
    row("GQ", "Equatorial Guinea", "240"),
    row("ER", "Eritrea", "291"),
    row("EE", "Estonia", "372"),
    row("SZ", "Eswatini", "268"),
    row("ET", "Ethiopia", "251"),
    row("FK", "Falkland Islands", "500"),
    row("FO", "Faroe Islands", "298"),
    row("FJ", "Fiji", "679"),
    row("FI", "Finland", "358"),
    row("FR", "France", "33"),
    row("GF", "French Guiana", "594"),
    row("PF", "French Polynesia", "689"),
    row("GA", "Gabon", "241"),
    row("GM", "Gambia", "220"),
    row("GE", "Georgia", "995"),
    row("DE", "Germany", "49"),
    row("GH", "Ghana", "233"),
    row("GI", "Gibraltar", "350"),
    row("GR", "Greece", "30"),
    row("GL", "Greenland", "299"),
    row("GD", "Grenada", "1473"),
    row("GP", "Guadeloupe", "590"),
    row("GU", "Guam", "1671"),
    row("GT", "Guatemala", "502"),
    row("GG", "Guernsey", "44"),
    row("GN", "Guinea", "224"),
    row("GW", "Guinea-Bissau", "245"),
    row("GY", "Guyana", "592"),
    row("HT", "Haiti", "509"),
    row("HN", "Honduras", "504"),
    row("HK", "Hong Kong SAR China", "852"),
    row("HU", "Hungary", "36"),
    row("IS", "Iceland", "354"),
    row("IN", "India", "91"),
    row("ID", "Indonesia", "62"),
    row("IR", "Iran", "98"),
    row("IQ", "Iraq", "964"),
    row("IE", "Ireland", "353"),
    row("IM", "Isle of Man", "44"),
    row("IL", "Israel", "972"),
    row("IT", "Italy", "39"),
    row("JM", "Jamaica", "1"),
    row("JP", "Japan", "81"),
    row("JE", "Jersey", "44"),
    row("JO", "Jordan", "962"),
    row("KZ", "Kazakhstan", "7"),
    row("KE", "Kenya", "254"),
    row("KI", "Kiribati", "686"),
    row("XK", "Kosovo", "383"),
    row("KW", "Kuwait", "965"),
    row("KG", "Kyrgyzstan", "996"),
    row("LA", "Laos", "856"),
    row("LV", "Latvia", "371"),
    row("LB", "Lebanon", "961"),
    row("LS", "Lesotho", "266"),
    row("LR", "Liberia", "231"),
    row("LY", "Libya", "218"),
    row("LI", "Liechtenstein", "423"),
    row("LT", "Lithuania", "370"),
    row("LU", "Luxembourg", "352"),
    row("MO", "Macao SAR China", "853"),
    row("MG", "Madagascar", "261"),
    row("MW", "Malawi", "265"),
    row("MY", "Malaysia", "60"),
    row("MV", "Maldives", "960"),
    row("ML", "Mali", "223"),
    row("MT", "Malta", "356"),
    row("MH", "Marshall Islands", "692"),
    row("MQ", "Martinique", "596"),
    row("MR", "Mauritania", "222"),
    row("MU", "Mauritius", "230"),
    row("YT", "Mayotte", "262"),
    row("MX", "Mexico", "52"),
    row("FM", "Micronesia", "691"),
    row("MD", "Moldova", "373"),
    row("MC", "Monaco", "377"),
    row("MN", "Mongolia", "976"),
    row("ME", "Montenegro", "382"),
    row("MS", "Montserrat", "1664"),
    row("MA", "Morocco", "212"),
    row("MZ", "Mozambique", "258"),
    row("MM", "Myanmar", "95"),
    row("NA", "Namibia", "264"),
    row("NR", "Nauru", "674"),
    row("NP", "Nepal", "977"),
    row("NL", "Netherlands", "31"),
    row("NC", "New Caledonia", "687"),
    row("NZ", "New Zealand", "64"),
    row("NI", "Nicaragua", "505"),
    row("NE", "Niger", "227"),
    row("NG", "Nigeria", "234"),
    row("NU", "Niue", "683"),
    row("NF", "Norfolk Island", "672"),
    row("KP", "North Korea", "850"),
    row("MK", "North Macedonia", "389"),
    row("MP", "Northern Mariana Islands", "1670"),
    row("NO", "Norway", "47"),
    row("OM", "Oman", "968"),
    row("PK", "Pakistan", "92"),
    row("PW", "Palau", "680"),
    row("PS", "Palestine", "970"),
    row("PA", "Panama", "507"),
    row("PG", "Papua New Guinea", "675"),
    row("PY", "Paraguay", "595"),
    row("PE", "Peru", "51"),
    row("PH", "Philippines", "63"),
    row("PN", "Pitcairn Islands", "64"),
    row("PL", "Poland", "48"),
    row("PT", "Portugal", "351"),
    row("PR", "Puerto Rico", "1"),
    row("QA", "Qatar", "974"),
    row("RE", "Réunion", "262"),
    row("RO", "Romania", "40"),
    row("RU", "Russia", "7"),
    row("RW", "Rwanda", "250"),
    row("BL", "Saint Barthélemy", "590"),
    row("SH", "Saint Helena", "290"),
    row("KN", "Saint Kitts and Nevis", "1869"),
    row("LC", "Saint Lucia", "1758"),
    row("MF", "Saint Martin", "590"),
    row("PM", "Saint Pierre and Miquelon", "508"),
    row("VC", "Saint Vincent and the Grenadines", "1784"),
    row("WS", "Samoa", "685"),
    row("SM", "San Marino", "378"),
    row("ST", "São Tomé and Príncipe", "239"),
    row("SA", "Saudi Arabia", "966"),
    row("SN", "Senegal", "221"),
    row("RS", "Serbia", "381"),
    row("SC", "Seychelles", "248"),
    row("SL", "Sierra Leone", "232"),
    row("SG", "Singapore", "65"),
    row("SX", "Sint Maarten", "1721"),
    row("SK", "Slovakia", "421"),
    row("SI", "Slovenia", "386"),
    row("SB", "Solomon Islands", "677"),
    row("SO", "Somalia", "252"),
    row("ZA", "South Africa", "27"),
    row("KR", "South Korea", "82"),
    row("SS", "South Sudan", "211"),
    row("ES", "Spain", "34"),
    row("LK", "Sri Lanka", "94"),
    row("SD", "Sudan", "249"),
    row("SR", "Suriname", "597"),
    row("SJ", "Svalbard and Jan Mayen", "47"),
    row("SE", "Sweden", "46"),
    row("CH", "Switzerland", "41"),
    row("SY", "Syria", "963"),
    row("TW", "Taiwan", "886"),
    row("TJ", "Tajikistan", "992"),
    row("TZ", "Tanzania", "255"),
    row("TH", "Thailand", "66"),
    row("TL", "Timor-Leste", "670"),
    row("TG", "Togo", "228"),
    row("TK", "Tokelau", "690"),
    row("TO", "Tonga", "676"),
    row("TT", "Trinidad and Tobago", "1868"),
    row("TN", "Tunisia", "216"),
    row("TR", "Türkiye", "90"),
    row("TM", "Turkmenistan", "993"),
    row("TC", "Turks and Caicos Islands", "1649"),
    row("TV", "Tuvalu", "688"),
    row("VI", "U.S. Virgin Islands", "1340"),
    row("UG", "Uganda", "256"),
    row("UA", "Ukraine", "380"),
    row("AE", "United Arab Emirates", "971"),
    row("GB", "United Kingdom", "44"),
    row("US", "United States", "1"),
    row("UY", "Uruguay", "598"),
    row("UZ", "Uzbekistan", "998"),
    row("VU", "Vanuatu", "678"),
    row("VA", "Vatican City", "39"),
    row("VE", "Venezuela", "58"),
    row("VN", "Vietnam", "84"),
    row("WF", "Wallis and Futuna", "681"),
    row("EH", "Western Sahara", "212"),
    row("YE", "Yemen", "967"),
    row("ZM", "Zambia", "260"),
    row("ZW", "Zimbabwe", "263"),
];

/// The row for an ISO code, case-insensitively - a caller writing `"de"` means
/// Germany.
pub(crate) fn find(iso: &str) -> Option<&'static Country> {
    COUNTRIES
        .iter()
        .find(|country| country.iso.eq_ignore_ascii_case(iso))
}

/// The country by the longest dial code prefixing the number. Shared codes are
/// ambiguous, so [`country_for`](super::field::country_for) prefers the pick.
pub(crate) fn by_dial(digits: &str) -> Option<&'static Country> {
    COUNTRIES
        .iter()
        .filter(|country| digits.starts_with(country.dial))
        .max_by_key(|country| (country.dial.len(), !SHARING.contains(&country.iso)))
}

/// Rows that lose a dial-code tie to the main country, or `+39` would read as
/// the Vatican rather than Italy.
pub(crate) const SHARING: &[&str] = &[
    "AQ", "AX", "BL", "BQ", "CA", "CC", "CX", "DO", "EH", "GG", "IM", "JE", "JM", "KZ", "MF", "PN",
    "PR", "SJ", "VA", "YT",
];

/// Only the digits, as ASCII, which is every path's first step: a pasted
/// `(213) 373-4253` and a typed `2133734253` have to reach the same value.
pub(crate) fn digits_of(text: &str) -> String {
    text.chars().filter_map(ascii_digit).collect()
}

/// The zeros of the decimal digit runs a phone keyboard or IME types: Arabic-Indic,
/// Persian, the Indic scripts, Thai, Lao, Tibetan, Myanmar, Khmer, Mongolian, full width.
const ZEROS: &[u32] = &[
    0x0660, 0x06F0, 0x07C0, 0x0966, 0x09E6, 0x0A66, 0x0AE6, 0x0B66, 0x0BE6, 0x0C66, 0x0CE6, 0x0D66,
    0x0DE6, 0x0E50, 0x0ED0, 0x0F20, 0x1040, 0x1090, 0x17E0, 0x1810, 0xFF10,
];

/// A decimal digit of any of those scripts as its ASCII digit.
fn ascii_digit(c: char) -> Option<char> {
    if c.is_ascii_digit() {
        return Some(c);
    }
    let code = u32::from(c);
    ZEROS
        .iter()
        .find(|zero| (**zero..**zero + 10).contains(&code))
        .and_then(|zero| char::from_digit(code - zero, 10))
}

/// `+{dial}{national}`, or an empty string when nothing was typed - an empty
/// field posts nothing rather than a bare dial code.
pub(crate) fn to_e164(country: &Country, national: &str) -> String {
    let national = digits_of(national);
    match national.is_empty() {
        true => String::new(),
        false => format!("+{}{national}", country.dial),
    }
}

/// The digits after a leading `+` or `00`, or `None` for a national number:
/// `+49 171 ...` pasted from a contact card carries its own dial code.
pub(crate) fn international(text: &str) -> Option<String> {
    let text = text.trim_start();
    if let Some(rest) = text.strip_prefix(['+', '\u{FF0B}']) {
        return Some(digits_of(rest));
    }
    // `00` in any script, the rest after its second zero.
    let mut chars = text.char_indices();
    let zero = |(_, c): (usize, char)| ascii_digit(c) == Some('0');
    match (chars.next(), chars.next()) {
        (Some(first), Some((at, second))) if zero(first) && zero((at, second)) => {
            Some(digits_of(&text[at + second.len_utf8()..]))
        }
        _ => None,
    }
}

/// Whether the text is `country`'s own dial code and nothing after it: no number yet.
pub(crate) fn bare_code(country: &Country, text: &str) -> bool {
    international(text).is_some_and(|digits| digits == country.dial)
}

/// The E.164 a typed text stands for: its own dial code when it has one, else
/// `country`'s. A bare dial code is nothing typed.
pub(crate) fn value_of(country: &Country, text: &str) -> String {
    match international(text) {
        Some(digits) if digits.is_empty() || digits == country.dial => String::new(),
        Some(digits) => format!("+{digits}"),
        None => to_e164(country, text),
    }
}

/// The national part of an E.164 number, or `None` when it does not belong to
/// this country at all.
pub(crate) fn national_of(e164: &str, country: &Country) -> Option<String> {
    let digits = digits_of(e164);
    digits
        .strip_prefix(country.dial)
        .map(|national| national.to_string())
}

/// The national number grouped for reading, only where the plan has one fixed
/// shape. `None` otherwise, so the user's own spacing survives (todo 87b).
pub(crate) fn group(country: &Country, national: &str) -> Option<String> {
    let digits = digits_of(national);
    let groups: &[usize] = match (country.dial, digits.len()) {
        // The NANP, whose subscriber number is 3-3-4 in every member country.
        ("1", 10) => &[3, 3, 4],
        // Russia and Kazakhstan, 3-3-2-2.
        ("7", 10) => &[3, 3, 2, 2],
        // France, whose national significant number is nine digits read in
        // pairs after the leading one.
        ("33", 9) => &[1, 2, 2, 2, 2],
        _ => return None,
    };

    let mut rest = digits.as_str();
    let mut out = String::with_capacity(rest.len() + groups.len());
    for size in groups {
        let (head, tail) = rest.split_at(*size);
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(head);
        rest = tail;
    }
    Some(out)
}
