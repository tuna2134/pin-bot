use poise::serenity_prelude as serenity;
use sqlx::SqlitePool;
struct Data {
    pool: SqlitePool,
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(context_menu_command = "pin")]
async fn pin(ctx: Context<'_>, msg: serenity::Message) -> Result<(), Error> {
    let channel_id = msg.channel_id.0 as i64;
    let check = sqlx::query!("SELECT * FROM Message WHERE ChannelId = $1", channel_id)
        .fetch_one(&ctx.data().pool)
        .await;
    match check {
        Ok(_) => {
            ctx.say("既に作成されています。").await?;
        }
        Err(_) => {
            let author_id = msg.author.id.0 as i64;
            sqlx::query!(
                "INSERT INTO Message VALUES($1, $2, $3)",
                channel_id,
                msg.content,
                author_id
            )
            .execute(&ctx.data().pool)
            .await?;
            ctx.say("追加しました").await?;
        }
    }
    Ok(())
}
/// Pinを無効にします。
#[poise::command(slash_command)]
async fn unpin(ctx: Context<'_>) -> Result<(), Error> {
    let channel_id = ctx.channel_id().0 as i64;
    let check = sqlx::query!("SELECT * FROM Message WHERE ChannelId = $1", channel_id)
        .fetch_one(&ctx.data().pool)
        .await;
    match check {
        Ok(_) => {
            sqlx::query!("DELETE FROM Message WHERE ChannelId = $1", channel_id)
                .execute(&ctx.data().pool)
                .await?;
            ctx.say("削除しました。").await?;
        }
        Err(_) => {
            ctx.say("登録されていません。").await?;
        }
    }
    Ok(())
}

async fn all_event_handler(
    ctx: &serenity::Context,
    event: &poise::Event<'_>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        poise::Event::Ready { data_about_bot } => {
            println!("{} is ready", data_about_bot.user.name);
        }
        poise::Event::Message { new_message } => {
            let msg = new_message;
            if msg.author.bot {
                return Ok(());
            }
            println!("hello");
        }
        _ => {}
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let pool = SqlitePool::connect(&std::env::var("DATABASE_URL").expect("missing DATABASE_URL"))
        .await
        .unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![pin(), unpin()],
            event_handler: |ctx, event, _framework, data| {
                Box::pin(all_event_handler(ctx, event, data))
            },
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { pool: pool.clone() })
            })
        });

    framework.run().await.unwrap();
}
