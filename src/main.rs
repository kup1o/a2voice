use std::io::Cursor;

use teloxide::{
    dispatching::UpdateHandler,
    net::Download,
    prelude::*,
    types::{InputFile, MediaKind, MessageKind},
    utils::command::BotCommands,
};

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "start the bot.")]
    Start,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting the bot...");

    let bot = Bot::from_env();

    Dispatcher::builder(bot, schema())
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;
    let command_handler =
        teloxide::filter_command::<Command, _>().branch(case![Command::Start].endpoint(start));

    let audio_handler = Message::filter_audio().endpoint(audio);

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(audio_handler)
        .branch(dptree::endpoint(invalid_state));

    message_handler
}

async fn start(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

async fn audio(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Audio handler is working")
        .reply_to_message_id(msg.id)
        .await?;

    add_audio(&bot, &msg).await?;

    Ok(())
}

async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Unable to handle the message. Type /help to see the usage.",
    )
    .await?;
    Ok(())
}

async fn download_file(bot: &Bot, file_id: String) -> Option<Vec<u8>> {
    if let Ok(file) = bot.get_file(file_id).await {
        let mut out: Vec<u8> = Vec::new();
        let mut cursor = Cursor::new(&mut out);
        if let Ok(_) = bot.download_file(&file.path, &mut cursor).await {
            if !out.is_empty() {
                return Some(out);
            }
        }
    }
    None
}

async fn add_audio(bot: &Bot, msg: &Message) -> HandlerResult {
    if let MessageKind::Common(item) = &msg.kind {
        if let MediaKind::Audio(audio) = &item.media_kind {
            if let Some(file_name) = &audio.audio.file_name {
                if let Some(data) = download_file(bot, audio.audio.file.id.clone()).await {
                    bot.send_message(msg.chat.id, "audio_add_success")
                        .reply_to_message_id(msg.id)
                        .await?;

                    bot.send_voice(msg.chat.id, InputFile::memory(data)).await?;
                    return Ok(());
                }
                bot.send_message(msg.chat.id, "audio_add_dw_error")
                    .reply_to_message_id(msg.id)
                    .await?;
                return Ok(());
            }
        }
    }
    bot.send_message(msg.chat.id, "audio_add_format_error")
        .reply_to_message_id(msg.id)
        .await?;
    return Ok(());
}
