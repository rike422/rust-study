mod parser;

use std::net::TcpListener;
use std::thread;
use std::io::{Read, Write};
use std::io;

fn server_start() -> io::Result<()> {
    let lis = TcpListener::bind("127.0.0.1:8080")?;  // ①

    // ②
    for stream in lis.incoming() {
        // ③
        let mut stream = match stream {
            Ok(stream) => stream,
            // ④
            Err(e) => {
                println!("An error occurred while accepting a connection: {}", e);
                continue;
            }
        };

        // ⑤
        let _ = thread::spawn(
            // ⑥
            move || -> io::Result<()> {
                use parser::ParseResult::*;
                // リクエスト全体を格納するバッファ
                let mut buf = Vec::new();
                loop {
                    // 1回のread分を格納する一時バッファ
                    let mut b = [0; 1024];
                    // 入力をバッファに読み込む
                    // nには読み込んだバイト数が入る
                    let n = stream.read(&mut b)?;
                    if n == 0 {
                        // 読み込んだバイト数が0ならストリームを終了してスレッドから抜ける
                        return Ok(());
                    }
                    // リクエスト全体のバッファに、いま読み込んだ分を追記
                    buf.extend_from_slice(&b[0..n]);
                    // それ以外ではHTTP/0.9のリクエストの処理
                    match parser::parse(buf.as_slice()) {
                        // 入力の途中なら新たな入力を待つため次のイテレーションへ
                        Partial => continue,
                        // エラーなら不正な入力なので何も返さずスレッドから抜ける
                        // スレッドから抜けると stream のライフタイムが終わるため、コネクションが自動で閉じられる
                        Error => {
                            return Ok(());
                        }
                        // リクエストが届けば処理をする
                        Complete(req) => {
                            // レスポンスを返す処理をここに書く
                            // 本来はファイルの中身を返すが、ここではリクエストの内容を含んだ文字列を返す
                            write!(stream, "OK {}\r\n", req.0)?;
                            // 処理が完了したらスレッドから抜ける
                            return Ok(());
                        },
                    };
                }
            }
        );
    }
    Ok(())
}

fn main() {
    match server_start() {
        Ok(_) => (),
        Err(e) => println!("{:?}", e),
    }
}