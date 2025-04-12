use std::fs::{File, OpenOptions, remove_file};
use std::io::{Error, Write};
use std::path::Path;

pub fn delete_file(filename: &str) -> Result<(), Error> {
    // ファイルを削除する
    remove_file(filename)?;

    println!("ファイル '{}' が削除されました", filename);

    Ok(())
}

pub fn create_file(filename: &str) -> Result<(), Error> {
    let path = Path::new(filename);
    let _file = File::create(path)?;

    println!("ファイル '{}' が作成されました", filename);

    Ok(())
}

pub fn append_to_file(filename: &str, content: &str) -> Result<(), Error> {
    // ファイルを追加モードで開く（存在しない場合は作成する）
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)?;

    // ファイルに内容を書き込む
    file.write_all(content.as_bytes())?;

    Ok(())
}
