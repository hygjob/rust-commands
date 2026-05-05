use comfy_table::{Attribute, Cell, CellAlignment, ContentArrangement, Table};

fn show_table(entries: &[(String, u64, String, bool)]) {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);

    table.set_header(vec!["권한", "크기", "수정일", "이름"]);

    for (name, size, time, is_dir) in entries {
        let styled_name = if *is_dir {
            Cell::new(name).add_attribute(Attribute::Bold)
        } else {
            Cell::new(name)
        };

        table.add_row(vec![
            Cell::new("-rw-r--r--"),
            Cell::new(format_size(*size)).set_alignment(CellAlignment::Right),
            Cell::new(time),
            styled_name,
        ]);
    }

    println!("{table}");
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;

    if bytes >= MB {
        format!("{:.1}M", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}K", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

fn main() {
    let entries = vec![
        (
            "src".to_string(),
            4096,
            "2026-05-20 14:30".to_string(),
            true,
        ),
        (
            "Cargo.toml".to_string(),
            512,
            "2026-05-21 09:15".to_string(),
            false,
        ),
        (
            "target".to_string(),
            1024 * 1024 * 50,
            "2026-05-21 10:00".to_string(),
            true,
        ),
    ];


    show_table(&entries);
}
