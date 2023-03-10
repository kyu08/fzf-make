use skim::prelude::*;
use std::io::Cursor;

pub fn main() {
    let preview_command = "bat --style=numbers --color=always --highlight-line $(bat Makefile | rg -n {}: | sed -e 's/:.*//g') Makefile";
    let options = SkimOptionsBuilder::default()
        .height(Some("50%"))
        .multi(true)
        .preview(Some(preview_command))
        .build()
        .unwrap();

    // TODO: これをcommand一覧にする
    let input = "PHONY\nrun\nclear".to_string();

    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(input));

    let selected_items = Skim::run_with(&options, Some(items))
        .map(|out| out.selected_items)
        .unwrap_or_else(Vec::new);

    for item in selected_items.iter() {
        // TODO: ここでmake hogeする
        println!("{}", item.output());
    }
}
