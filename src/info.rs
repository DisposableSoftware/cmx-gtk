//! The metadata panel's contents (`WIN-13`).

use crate::metadata::Metadata;
use gtk::prelude::*;

pub fn panel(metadata: Option<&Metadata>) -> gtk::Widget {
    let Some(m) = metadata else {
        return gtk::Label::builder()
            .label("No metadata in this file.")
            .css_classes(["dim-label"])
            .wrap(true)
            .justify(gtk::Justification::Center)
            .vexpand(true)
            .margin_start(12)
            .margin_end(12)
            .build()
            .upcast();
    };
    let num = |n: Option<i64>| n.map(|n| n.to_string());
    let rows = [
        ("Series", m.series.clone()),
        ("Number", m.number.clone()),
        ("Volume", num(m.volume)),
        ("Count", num(m.count)),
        ("Title", m.title.clone()),
        ("Writer", m.writer.clone()),
        ("Penciller", m.penciller.clone()),
        ("Inker", m.inker.clone()),
        ("Colorist", m.colorist.clone()),
        ("Letterer", m.letterer.clone()),
        ("Cover Artist", m.cover_artist.clone()),
        ("Publisher", m.publisher.clone()),
        ("Date", m.date()),
        ("Pages", num(m.page_count)),
        ("Language", m.language.clone()),
        ("Genre", m.genre.clone()),
        ("Web", m.web.clone()),
    ];
    let grid = gtk::Grid::builder()
        .row_spacing(6)
        .column_spacing(12)
        .build();
    for (row, (label, value)) in rows
        .into_iter()
        .filter_map(|(l, v)| v.map(|v| (l, v)))
        .enumerate()
    {
        let label = gtk::Label::builder()
            .label(label)
            .css_classes(["dim-label"])
            .xalign(1.0)
            .yalign(0.0)
            .build();
        grid.attach(&label, 0, row as i32, 1, 1);
        grid.attach(&value_label(&value), 1, row as i32, 1, 1);
    }
    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    content.append(&grid);
    if let Some(summary) = &m.summary {
        content.append(
            &gtk::Label::builder()
                .label("Summary")
                .css_classes(["heading"])
                .xalign(0.0)
                .build(),
        );
        content.append(&value_label(summary));
    }
    content.upcast()
}

fn value_label(text: &str) -> gtk::Label {
    gtk::Label::builder()
        .label(text)
        .selectable(true)
        .wrap(true)
        .wrap_mode(gtk::pango::WrapMode::WordChar)
        .xalign(0.0)
        .hexpand(true)
        .build()
}
