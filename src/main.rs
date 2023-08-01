use std::io::Cursor;
use teloxide::{net::Download, prelude::*, types::{InputFile, MediaKind, MessageKind}};

#[tokio::main]
async fn main() {
    let bot = Bot::from_env();
    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        if let MessageKind::Common(item) = &msg.kind {
            if let MediaKind::Audio(audio) = &item.media_kind {
                if let Ok(file) = bot.get_file(audio.audio.file.id.clone()).await {
                    let mut out: Vec<u8> = Vec::new();
                    let mut cursor = Cursor::new(&mut out);
                    if (bot.download_file(&file.path, &mut cursor).await).is_ok() && !out.is_empty() {
                        bot.send_voice(msg.chat.id, InputFile::memory(out)).await?;
                    }
                }
            }
        }
        Ok(())
    }).await
}
