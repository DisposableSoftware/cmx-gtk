use quick_xml::events::Event;
use std::collections::HashMap;

/// Fields from `ComicInfo.xml` (`READ-12`). Every field is optional.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Metadata {
    pub series: Option<String>,
    /// Text on purpose: "7.1" and "Annual 1" occur in the wild.
    pub number: Option<String>,
    pub count: Option<i64>,
    pub volume: Option<i64>,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub writer: Option<String>,
    pub penciller: Option<String>,
    pub inker: Option<String>,
    pub colorist: Option<String>,
    pub letterer: Option<String>,
    pub cover_artist: Option<String>,
    pub publisher: Option<String>,
    pub year: Option<i64>,
    pub month: Option<i64>,
    pub day: Option<i64>,
    /// Shown only, never used for navigation (`READ-13`).
    pub page_count: Option<i64>,
    pub language: Option<String>,
    pub genre: Option<String>,
    pub web: Option<String>,
}

/// Where `ComicInfo.xml` is (`READ-10`): the root, or failing that any folder.
pub fn locate(entries: &[String]) -> Option<&String> {
    entries
        .iter()
        .find(|e| e.eq_ignore_ascii_case("comicinfo.xml"))
        .or_else(|| {
            entries.iter().find(|e| {
                e.rsplit('/')
                    .next()
                    .is_some_and(|n| n.eq_ignore_ascii_case("comicinfo.xml"))
            })
        })
}

/// Lenient parse (`READ-11`): `None` when no field could be read.
pub fn parse(xml: &[u8]) -> Option<Metadata> {
    let fields = fields(xml);
    if fields.is_empty() {
        return None;
    }
    let text = |k: &str| fields.get(k).cloned();
    let int = |k: &str| fields.get(k).and_then(|v| v.parse().ok());
    Some(Metadata {
        series: text("series"),
        number: text("number"),
        count: int("count"),
        volume: int("volume"),
        title: text("title"),
        summary: text("summary"),
        writer: text("writer"),
        penciller: text("penciller"),
        inker: text("inker"),
        colorist: text("colorist"),
        letterer: text("letterer"),
        cover_artist: text("coverartist"),
        publisher: text("publisher"),
        year: int("year"),
        month: int("month"),
        day: int("day"),
        page_count: int("pagecount"),
        language: text("languageiso"),
        genre: text("genre"),
        web: text("web"),
    })
}

/// The trimmed text of the root element's direct children, keyed by lowercased
/// name. Stops at the first error, keeping what was collected.
fn fields(xml: &[u8]) -> HashMap<String, String> {
    let mut reader = quick_xml::Reader::from_reader(xml);
    let mut fields = HashMap::new();
    let mut depth = 0usize;
    let mut text = String::new();
    loop {
        match reader.read_event() {
            Ok(Event::Start(_)) => {
                depth += 1;
                text.clear();
            }
            Ok(Event::End(e)) => {
                if depth == 2 {
                    let value = text.trim();
                    if !value.is_empty() {
                        let name = e.local_name().as_ref().to_lowercase();
                        fields.insert(name, value.to_owned());
                    }
                }
                depth = depth.saturating_sub(1);
                text.clear();
            }
            Ok(Event::Text(e)) => text.push_str(&e.xml10_content()),
            Ok(Event::CData(e)) => text.push_str(&e.into_inner()),
            Ok(Event::GeneralRef(e)) => {
                let resolved = match e.resolve_char_ref() {
                    Ok(Some(c)) => Some(c.to_string()),
                    Ok(None) => {
                        quick_xml::escape::resolve_xml_entity(&e.into_inner()).map(str::to_owned)
                    }
                    Err(_) => None,
                };
                match resolved {
                    Some(s) => text.push_str(&s),
                    None => break,
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            Ok(_) => {}
        }
    }
    fields
}

impl Metadata {
    /// `Series #Number — Title` from the parts that exist (`READ-14`).
    pub fn display_title(&self) -> Option<String> {
        let lead = [
            self.series.clone(),
            self.number.as_ref().map(|n| format!("#{n}")),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ");
        match (lead.is_empty(), &self.title) {
            (false, Some(title)) => Some(format!("{lead} — {title}")),
            (false, None) => Some(lead),
            (true, Some(title)) => Some(title.clone()),
            (true, None) => None,
        }
    }

    /// `Year-Month-Day` from the parts that exist.
    pub fn date(&self) -> Option<String> {
        let parts: Vec<String> = [self.year, self.month, self.day]
            .into_iter()
            .flatten()
            .map(|n| n.to_string())
            .collect();
        (!parts.is_empty()).then(|| parts.join("-"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(xml: &str) -> Option<Metadata> {
        parse(xml.as_bytes())
    }

    #[test]
    fn parses_a_full_document() {
        let m = p(r#"<?xml version="1.0"?>
            <ComicInfo xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
              <Series>Saga of Testing</Series>
              <Number>7</Number>
              <Count>12</Count>
              <Volume>2</Volume>
              <Title>The Mock &amp; Menace</Title>
              <Summary><![CDATA[A comic <about> tests.]]></Summary>
              <Writer>W. Riter</Writer>
              <Penciller>P</Penciller><Inker>I</Inker><Colorist>C</Colorist>
              <Letterer>L</Letterer><CoverArtist>CA</CoverArtist>
              <Publisher>Unit Press</Publisher>
              <Year>1999</Year>
              <Month>4</Month>
              <Day>15</Day>
              <PageCount>22</PageCount>
              <LanguageISO>en</LanguageISO>
              <Genre>Testing</Genre>
              <Web>https://example.com</Web>
            </ComicInfo>"#)
        .unwrap();
        assert_eq!(m.series.as_deref(), Some("Saga of Testing"));
        assert_eq!(m.number.as_deref(), Some("7"));
        assert_eq!(m.count, Some(12));
        assert_eq!(m.volume, Some(2));
        assert_eq!(m.title.as_deref(), Some("The Mock & Menace"));
        assert_eq!(m.summary.as_deref(), Some("A comic <about> tests."));
        assert_eq!(m.cover_artist.as_deref(), Some("CA"));
        assert_eq!(m.date().as_deref(), Some("1999-4-15"));
        assert_eq!(m.page_count, Some(22));
        assert_eq!(m.language.as_deref(), Some("en"));
        assert_eq!(m.web.as_deref(), Some("https://example.com"));
    }

    #[test]
    fn partial_document_keeps_what_exists() {
        let m = p("<ComicInfo><Series>Solo</Series><Title>  </Title></ComicInfo>").unwrap();
        assert_eq!(m.series.as_deref(), Some("Solo"));
        assert_eq!(m.title, None, "empty values are left out");
        assert_eq!(m.year, None);
    }

    #[test]
    fn non_numeric_numeric_fields_are_left_out() {
        let m = p(
            "<ComicInfo><Year>MCMXCIX</Year><PageCount>three</PageCount>\
                   <Number>Annual 1</Number></ComicInfo>",
        )
        .unwrap();
        assert_eq!(m.year, None);
        assert_eq!(m.page_count, None);
        assert_eq!(m.number.as_deref(), Some("Annual 1"));
    }

    #[test]
    fn nested_containers_are_ignored() {
        let m = p(
            r#"<ComicInfo><Title>Real</Title><Pages><Page Image="0">x</Page></Pages></ComicInfo>"#,
        )
        .unwrap();
        assert_eq!(m.title.as_deref(), Some("Real"));
        assert_eq!(
            m,
            Metadata {
                title: Some("Real".into()),
                ..Default::default()
            }
        );
    }

    #[test]
    fn garbage_and_empty_documents_have_no_metadata() {
        assert_eq!(p("this is not xml at all"), None);
        assert_eq!(p("<ComicInfo></ComicInfo>"), None);
        assert_eq!(p(""), None);
        assert_eq!(parse(&[0xff, 0xfe, 0x00, 0x3c]), None);
    }

    #[test]
    fn malformed_documents_keep_the_fields_read_before_the_error() {
        let m = p("<ComicInfo><Series>Cut</Series><Title>Off").unwrap();
        assert_eq!(m.series.as_deref(), Some("Cut"));
        let m = p("<ComicInfo><Series>Ok</Series><Title>a &bogus; b</Title></ComicInfo>").unwrap();
        assert_eq!(m.series.as_deref(), Some("Ok"));
        let m = p("<ComicInfo><Series>Ok</Series></Wrong><Title>t</Title>").unwrap();
        assert_eq!(m.series.as_deref(), Some("Ok"));
    }

    #[test]
    fn finds_comic_info_at_the_root_first() {
        let e = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(
            locate(&e(&["sub/ComicInfo.xml", "comicinfo.XML"])).map(String::as_str),
            Some("comicinfo.XML")
        );
        assert_eq!(
            locate(&e(&["a.png", "sub/ComicInfo.xml"])).map(String::as_str),
            Some("sub/ComicInfo.xml")
        );
        assert_eq!(locate(&e(&["a.png", "ComicInfo.xml.bak"])), None);
    }

    #[test]
    fn display_title_uses_the_parts_that_exist() {
        let m = |series: Option<&str>, number: Option<&str>, title: Option<&str>| Metadata {
            series: series.map(Into::into),
            number: number.map(Into::into),
            title: title.map(Into::into),
            ..Default::default()
        };
        let t = |m: Metadata| m.display_title();
        assert_eq!(
            t(m(Some("Saga"), Some("7"), Some("Hi"))).as_deref(),
            Some("Saga #7 — Hi")
        );
        assert_eq!(
            t(m(Some("Saga"), Some("7"), None)).as_deref(),
            Some("Saga #7")
        );
        assert_eq!(t(m(Some("Saga"), None, None)).as_deref(), Some("Saga"));
        assert_eq!(t(m(None, Some("7"), None)).as_deref(), Some("#7"));
        assert_eq!(
            t(m(Some("Saga"), None, Some("Hi"))).as_deref(),
            Some("Saga — Hi")
        );
        assert_eq!(t(m(None, None, Some("Hi"))).as_deref(), Some("Hi"));
        assert_eq!(t(m(None, None, None)), None);
    }
}
