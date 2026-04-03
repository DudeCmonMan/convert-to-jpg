use crate::utils;
use cursive::theme::{BaseColor, Color, PaletteColor};
use cursive::view::{Nameable, Resizable};
use cursive::views::{LinearLayout, Panel, SelectView, TextView};
use cursive::Cursive;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub struct ImageEntry {
    pub path: PathBuf,
    pub filename: String,
    pub format: String,
    pub file_size: u64,
}

fn item_label(entry: &ImageEntry, accepted: bool) -> String {
    let marker = if accepted { "✓" } else { "✗" };
    format!("[{}]  {}", marker, entry.filename)
}

fn detail_text(entry: &ImageEntry) -> String {
    vec![
        format!("  Path:    {}", entry.path.display()),
        format!("  Format:  {}", entry.format),
        format!("  Size:    {}", utils::format_file_size(entry.file_size)),
    ]
    .join("\n")
}

fn rebuild_list(siv: &mut Cursive, entries: &[ImageEntry], accepted: &[bool], focus: usize) {
    siv.call_on_name("list", |v: &mut SelectView<usize>| {
        v.clear();
        for (i, entry) in entries.iter().enumerate() {
            v.add_item(item_label(entry, accepted[i]), i);
        }
        if !entries.is_empty() {
            v.set_selection(focus.min(entries.len() - 1));
        }
    });
    update_detail(siv, entries, focus);
}

fn update_detail(siv: &mut Cursive, entries: &[ImageEntry], focus: usize) {
    let text = entries.get(focus).map(detail_text).unwrap_or_default();
    siv.call_on_name("detail", |v: &mut TextView| {
        v.set_content(text);
    });
}

fn current_focus(siv: &mut Cursive) -> usize {
    siv.call_on_name("list", |v: &mut SelectView<usize>| {
        v.selected_id().unwrap_or(0)
    })
    .unwrap_or(0)
}

/// All items start accepted. Returns indices of accepted entries, or empty vec on cancel.
pub fn select_files(entries: &[ImageEntry]) -> anyhow::Result<Vec<usize>> {
    if entries.is_empty() {
        println!("No convertible files found.");
        return Ok(vec![]);
    }

    let entries: Arc<Vec<ImageEntry>> = Arc::new(
        entries
            .iter()
            .map(|e| ImageEntry {
                path: e.path.clone(),
                filename: e.filename.clone(),
                format: e.format.clone(),
                file_size: e.file_size,
            })
            .collect(),
    );

    let n = entries.len();
    let accepted: Arc<Mutex<Vec<bool>>> = Arc::new(Mutex::new(vec![true; n]));
    let cancelled: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    let mut list: SelectView<usize> = SelectView::new();
    for (i, entry) in entries.iter().enumerate() {
        list.add_item(item_label(entry, true), i);
    }

    {
        let entries = Arc::clone(&entries);
        list.set_on_select(move |s, &idx| {
            update_detail(s, &entries, idx);
        });
    }

    let list = list.with_name("list");

    let first_detail = entries.first().map(detail_text).unwrap_or_default();
    let detail = TextView::new(first_detail).with_name("detail");

    let legend = TextView::new(
        " ↑/↓ j/k: navigate   Space: toggle   a: accept all\n Enter: confirm       q/Esc: cancel",
    );

    let n_label = format!("{} file{}", n, if n == 1 { "" } else { "s" });
    let layout = LinearLayout::vertical()
        .child(Panel::new(list).title(format!("Convert to JPEG ({})", n_label)).full_width())
        .child(Panel::new(detail).title("File Details").full_width())
        .child(legend.full_width());

    let mut siv = cursive::default();

    let mut theme = siv.current_theme().clone();
    theme.palette[PaletteColor::Background] = Color::Light(BaseColor::Black);
    theme.palette[PaletteColor::View] = Color::RgbLowRes(3, 3, 3);
    theme.palette[PaletteColor::Primary] = Color::Dark(BaseColor::Black);
    theme.palette[PaletteColor::TitlePrimary] = Color::Dark(BaseColor::Black);
    theme.palette[PaletteColor::Secondary] = Color::Dark(BaseColor::Black);
    theme.palette[PaletteColor::Highlight] = Color::RgbLowRes(5, 3, 0);
    theme.palette[PaletteColor::HighlightInactive] = Color::RgbLowRes(4, 2, 0);
    theme.palette[PaletteColor::HighlightText] = Color::Dark(BaseColor::Black);
    siv.set_theme(theme);

    siv.add_fullscreen_layer(layout.full_screen());

    {
        let entries = Arc::clone(&entries);
        let accepted = Arc::clone(&accepted);
        siv.add_global_callback(' ', move |s| {
            let focus = current_focus(s);
            {
                let mut acc = accepted.lock().unwrap();
                acc[focus] = !acc[focus];
            }
            let acc = accepted.lock().unwrap();
            rebuild_list(s, &entries, &acc, focus);
        });
    }

    siv.add_global_callback('j', |s| {
        s.call_on_name("list", |v: &mut SelectView<usize>| {
            let cur = v.selected_id().unwrap_or(0);
            v.set_selection((cur + 1).min(v.len().saturating_sub(1)));
        });
    });

    siv.add_global_callback('k', |s| {
        s.call_on_name("list", |v: &mut SelectView<usize>| {
            let cur = v.selected_id().unwrap_or(0);
            v.set_selection(cur.saturating_sub(1));
        });
    });

    {
        let accepted = Arc::clone(&accepted);
        siv.add_global_callback('a', move |s| {
            for v in accepted.lock().unwrap().iter_mut() {
                *v = true;
            }
            s.quit();
        });
    }

    siv.add_global_callback(cursive::event::Key::Enter, |s| s.quit());

    {
        let cancelled = Arc::clone(&cancelled);
        siv.add_global_callback('q', move |s| {
            cancelled.store(true, Ordering::Relaxed);
            s.quit();
        });
    }

    {
        let cancelled = Arc::clone(&cancelled);
        siv.add_global_callback(cursive::event::Key::Esc, move |s| {
            cancelled.store(true, Ordering::Relaxed);
            s.quit();
        });
    }

    siv.run();

    if cancelled.load(Ordering::Relaxed) {
        return Ok(vec![]);
    }

    let result = accepted
        .lock()
        .unwrap()
        .iter()
        .enumerate()
        .filter_map(|(i, &a)| if a { Some(i) } else { None })
        .collect();

    Ok(result)
}
