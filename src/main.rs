use dotenv::dotenv;
use poise::{
    CreateReply,
    serenity_prelude::{self as serenity, CreateEmbed, UserId, futures::{self, Stream, StreamExt}},
};
use sqlx::{Row, SqlitePool};

struct Data {} // User data, which is stored and accessible in all command invocations

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

async fn autocomplete_focus<'a>(
    _ctx: Context<'_>,
    focus: &'a str,
) -> impl Stream<Item = String> + 'a {
    futures::stream::iter(&["sfb", "sfs", "alt", "inroll", "outroll", "onehands", "redirects"])
        .filter(move |name| {
            futures::future::ready(name.to_lowercase().contains(&focus.to_lowercase()))
        })
        .map(|name| name.to_string())
}

#[poise::command(slash_command, prefix_command)]
async fn insert_layout(
    ctx: Context<'_>,
    #[description = "Creator of the layout"] creator: UserId,
    #[description = "Name of the layout"] name: String,
    #[description = "Magic flag"] magic: bool,
    #[description = "Thumb alpha flag"] thumb_alpha: bool,
    #[description = "Focus type"]
    #[autocomplete = "autocomplete_focus"]
    focus: String,
) -> Result<(), Error> {
    let db_path = std::env::var("GARFDB_PATH").unwrap_or("/var/lib/garf/scores.db".into());
    let pool = SqlitePool::connect(&format!("sqlite:{}", db_path)).await?;

    sqlx::query(
        r#"
        INSERT INTO layout (Name, Creator, Magic, ThumbAlpha, Focus)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
    )
    .bind(name)
    .bind(creator.to_string())
    .bind(magic)
    .bind(thumb_alpha)
    .bind(focus)
    .execute(&pool)
    .await?;

    ctx.say("Layout inserted successfully!").await?;
    Ok(())
}
#[poise::command(slash_command, prefix_command)]
async fn get_scores(
    ctx: Context<'_>,
    #[description = "Filter by user"] user_filter: Option<String>,
    #[description = "Filter by layout"] layout_filter: Option<String>,
    #[description = "Filter by magic"] magic_filter: Option<bool>,
    #[description = "Filter by thumb alpha"] thumb_alpha_filter: Option<bool>,
    #[description = "Filter by focus"]
    #[autocomplete = "autocomplete_focus"]
     focus_filter: Option<String>,
    #[description = "Filter by creator"] creator_filter: Option<String>,
) -> Result<(), Error> {
    // Defer the response to indicate the bot is processing
    ctx.defer().await?;

    let db_path = std::env::var("GARFDB_PATH").unwrap_or("/var/lib/garf/scores.db".into());
    let pool = SqlitePool::connect(&format!("sqlite:{}", db_path)).await?;

    // Extract the raw user ID from the creator_filter string
    let creator_id = match creator_filter {
        Some(ref creator) => {
            if creator.starts_with("<@") && creator.ends_with('>') {
                Some(&creator[2..creator.len() - 1])
            } else {
                creator_filter.as_deref()
            }
        }
        None => creator_filter.as_deref(),
    };

    // Extract the raw user ID from the user_filter string
    let user_id = match user_filter {
        Some(ref user) => {
            if user.starts_with("<@") && user.ends_with('>') {
                Some(&user[2..user.len() - 1])
            } else {
                user_filter.as_deref()
            }
        }
        None => user_filter.as_deref(),
    };

    // Execute the query
    let rows = sqlx::query(
        r#"
        SELECT 
            User,
            Speed,
            layout.Name AS Layout,
            Magic, 
            ThumbAlpha, 
            Focus, 
            Creator 
        FROM 
            score
            INNER JOIN layout USING (LayoutId)
        WHERE User = COALESCE(?1, User)
            AND Layout = COALESCE(?2, Layout)
            AND Magic = COALESCE(?3, Magic)
            AND ThumbAlpha = COALESCE(?4, ThumbAlpha)
            AND Focus = COALESCE(?5, Focus)
            AND Creator = COALESCE(?6, Creator)
        ORDER BY Speed DESC
        "#,
    )
    .bind(user_id)
    .bind(layout_filter)
    .bind(magic_filter)
    .bind(thumb_alpha_filter)
    .bind(focus_filter)
    .bind(creator_id)
    .fetch_all(&pool)
    .await?;

    // Build the message
    let mut message = String::new();
    let mut i = 1;
    for row in rows {
        message.push_str(&format!(
            "#{} **{} WPM**: <@{}> on {}\n",
            i,
            &row.get::<i64, _>("Speed"),
            row.get::<String, _>("User"),
            &row.get::<String, _>("Layout")
        ));
        i += 1;
    }

    // Create the embed
    let embed = CreateEmbed::new()
        .title("Leaderboard")
        .field("Scores", message, false);
    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn insert_score(
    ctx: Context<'_>,
    #[description = "Name of the layout"] layout: String,
    #[description = "Speed of the score"] speed: u16,
) -> Result<(), Error> {
    let db_path = std::env::var("GARFDB_PATH").unwrap_or("/var/lib/garf/scores.db".into());
    let pool = SqlitePool::connect(&format!("sqlite:{}", db_path)).await?;
    let user_id = ctx.author().id.to_string();

    // Get the LayoutId for the given layout name
    let layout_id = sqlx::query(
        r#"
        SELECT LayoutId FROM layout WHERE Name = ?1
        "#,
    )
    .bind(&layout)
    .fetch_one(&pool)
    .await?
    .get::<i64, _>("LayoutId");

    // Insert the score
    sqlx::query(
        r#"
        INSERT INTO score (LayoutId, User, Speed)
        VALUES (?1, ?2, ?3)
        "#,
    )
    .bind(layout_id)
    .bind(user_id)
    .bind(speed)
    .execute(&pool)
    .await?;

    ctx.say("Score inserted successfully!").await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![get_scores(), insert_layout(), insert_score()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
