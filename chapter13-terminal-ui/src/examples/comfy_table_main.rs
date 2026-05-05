use comfy_table::{Table, ContentArrangement};
//use comfy_table::presets::UTF8_FULL;

fn main() {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    //table.load_preset(UTF8_FULL);

    table.set_header(vec!["이름", "나이", "도시"]);
    table.add_row(vec!["Alice", "30", "Seoul"]);
    table.add_row(vec!["Bob", "25", "Busan"]);
    table.add_row(vec!["Charlie", "35", "Daejeon"]);

    println!("{table}");
}