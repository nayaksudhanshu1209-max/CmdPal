//! Declarative page layout for the settings GUI, plus the searchable settings index.

#[derive(Debug, Clone, Copy)]
pub enum Item {
    Check(&'static str, &'static str),
    Edit(&'static str, &'static str),
    List(&'static str, &'static str),
    Text(&'static str),
    Buttons(&'static [(&'static str, &'static str)]),
    IndexStatus,
    NotesManager,
}

pub struct Page {
    pub id: &'static str,
    pub title: &'static str,
    pub items: Vec<Item>,
}

pub fn pages() -> Vec<Page> {
    vec![
        Page {
            id: "general",
            title: "General",
            items: vec![
                Item::Edit("hotkey", "Global shortcut (e.g. Ctrl+Alt+Space)"),
                Item::Check("startup", "Start CmdPal when I sign in to Windows"),
                Item::Check("near_cursor", "Show the palette near the mouse cursor"),
                Item::Check("show_suggestions", "Show suggestions when the palette opens"),
                Item::Check("compact", "Compact density"),
                Item::Check("animations", "Short fade animation (turn off for reduced motion)"),
                Item::Text("Click Apply to save. Settings are stored in %APPDATA%\\CmdPal\\settings.toml (you can also edit that file)."),
            ],
        },
        Page { id: "exclusions", title: "Hidden results", items: vec![Item::Text("Results you hid (right-click a result in the palette). Remove an entry to show it again."), Item::List("exclusions", "Hidden results")] },
        Page {
            id: "bangs",
            title: "Web bangs",
            items: vec![
                Item::Edit("default_search", "Default search alias"),
                Item::List("bangs", "Aliases. Type the alias then Space in the palette to search that provider."),
            ],
        },
        Page { id: "bookmarks", title: "Bookmarks", items: vec![Item::List("bookmarks", "Bookmarks (shown for an empty bang; opened in your default browser)")] },
        Page { id: "commands", title: "Commands", items: vec![Item::List("commands", "Your commands. Each argument is passed separately - nothing is run through a shell.")] },
        Page {
            id: "files",
            title: "Files",
            items: vec![
                Item::Text("CmdPal only indexes the folders listed here - never the whole drive."),
                Item::Buttons(&[("add_desktop", "Add Desktop"), ("add_documents", "Add Documents"), ("add_downloads", "Add Downloads")]),
                Item::List("roots", "Indexing folders (searched recursively)"),
                Item::Edit("scan_interval", "Rescan every N minutes (0 = only at startup / on demand)"),
                Item::Edit("max_entries", "Maximum indexed entries (limits memory)"),
                Item::Check("content_index", "Also index text content (uses more CPU, disk reads and memory)"),
                Item::IndexStatus,
                Item::Buttons(&[("pause", "Pause / resume"), ("rebuild", "Rebuild index"), ("clear", "Clear index")]),
            ],
        },
        Page {
            id: "file_exclusions",
            title: "File exclusions",
            items: vec![Item::List("patterns", "Excluded subfolders (full path) or name patterns"), Item::List("extensions", "Excluded file types")],
        },
        Page {
            id: "quick_folders",
            title: "Quick folders",
            items: vec![
                Item::Text("Quick folders are shortcuts. They are separate from indexing folders: removing one never removes the other."),
                Item::List("quick_folders", "Quick folders"),
            ],
        },
        Page { id: "notes", title: "Notes", items: vec![Item::Edit("notes_alias", "Note alias (type it then Space in the palette)"), Item::NotesManager] },
    ]
}

/// (title, keywords, page id) used by settings search from the palette.
pub fn search_entries() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("Global shortcut", "hotkey keyboard activate open palette", "general"),
        ("Start with Windows", "startup autostart login boot", "general"),
        ("Palette near mouse cursor", "cursor position mouse", "general"),
        ("Suggestions at startup", "suggestions show hide empty", "general"),
        ("Compact mode", "density compact small", "general"),
        ("Animations and reduced motion", "animation fade motion", "general"),
        ("Hidden results", "exclusions hide hidden apps", "exclusions"),
        ("Web bangs and aliases", "bang alias google youtube search provider", "bangs"),
        ("Bookmarks", "bookmark favorites urls links", "bookmarks"),
        ("Custom commands", "commands tasks run program", "commands"),
        ("File indexing folders", "files index roots folders search", "files"),
        ("File index status and controls", "index rebuild pause clear status", "files"),
        ("Content indexing", "full text content index", "files"),
        ("File exclusions", "exclude patterns extensions ignore", "file_exclusions"),
        ("Quick folders", "quick folders shortcuts", "quick_folders"),
        ("Notes", "notes markdown alias rename delete", "notes"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::settings::logic::binding;

    #[test]
    fn pages_are_consistent() {
        let ps = pages();
        let mut ids: Vec<_> = ps.iter().map(|p| p.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), ps.len());
        for p in &ps {
            for it in &p.items {
                match it {
                    Item::List(k, _) => assert!(!binding::fields(k).is_empty(), "{k}"),
                    Item::Check(k, _) | Item::Edit(k, _) => {
                        let mut s = crate::core::settings::Settings::default();
                        assert!(binding::set_value(&mut s, k, "1").is_ok() || *k == "hotkey" || *k == "default_search", "{k}")
                    }
                    _ => {}
                }
            }
        }
        for (_, _, page) in search_entries() {
            assert!(ps.iter().any(|p| p.id == page), "{page}");
        }
    }
}
