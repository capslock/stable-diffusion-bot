use std::collections::HashSet;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use teloxide::{
    dispatching::{
        dialogue::{self, serializer::Json, ErasedStorage, InMemStorage, SqliteStorage, Storage},
        UpdateHandler,
    },
    prelude::*,
    types::{Update, UserId},
    utils::command::BotCommands,
};
use tracing::warn;

use stable_diffusion_api::{Api, Img2ImgRequest, Txt2ImgRequest};

mod handlers;
mod helpers;
use handlers::*;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub enum State {
    #[default]
    New,
    Ready {
        txt2img: Txt2ImgRequest,
        img2img: Img2ImgRequest,
    },
    SettingsTxt2Img {
        selection: Option<String>,
        txt2img: Txt2ImgRequest,
        img2img: Img2ImgRequest,
    },
    SettingsImg2Img {
        selection: Option<String>,
        txt2img: Txt2ImgRequest,
        img2img: Img2ImgRequest,
    },
}

impl State {
    fn new_with_defaults(txt2img: Txt2ImgRequest, img2img: Img2ImgRequest) -> Self {
        Self::Ready { txt2img, img2img }
    }
}

fn default_txt2img() -> Txt2ImgRequest {
    Txt2ImgRequest {
        styles: Some(Vec::new()),
        seed: Some(-1),
        sampler_index: Some("Euler".to_owned()),
        batch_size: Some(1),
        n_iter: Some(1),
        steps: Some(50),
        cfg_scale: Some(7.0),
        width: Some(512),
        height: Some(512),
        restore_faces: Some(false),
        tiling: Some(false),
        negative_prompt: Some("".to_owned()),
        ..Default::default()
    }
}

fn default_img2img() -> Img2ImgRequest {
    Img2ImgRequest {
        denoising_strength: Some(0.75),
        styles: Some(Vec::new()),
        seed: Some(-1),
        sampler_index: Some("Euler".to_owned()),
        batch_size: Some(1),
        n_iter: Some(1),
        steps: Some(50),
        cfg_scale: Some(7.0),
        width: Some(512),
        height: Some(512),
        restore_faces: Some(false),
        tiling: Some(false),
        negative_prompt: Some("".to_owned()),
        resize_mode: Some(1),
        ..Default::default()
    }
}

type DialogueStorage = std::sync::Arc<ErasedStorage<State>>;

type DiffusionDialogue = Dialogue<State, ErasedStorage<State>>;

/// Struct to run a StableDiffusionBot
#[derive(Clone)]
pub struct StableDiffusionBot {
    bot: Bot,
    storage: DialogueStorage,
    config: ConfigParameters,
}

impl StableDiffusionBot {
    /// Creates an UpdateHandler for the bot
    fn schema() -> UpdateHandler<anyhow::Error> {
        let auth_filter = dptree::filter(|cfg: ConfigParameters, upd: Update| {
            upd.user()
                .map(|user| cfg.allowed_users.contains(&user.id))
                .unwrap_or_default()
        });

        let unauth_command_handler = Update::filter_message().chain(
            teloxide::filter_command::<UnauthenticatedCommands, _>()
                .endpoint(unauthenticated_commands_handler),
        );

        let authenticated = auth_filter.branch(settings_schema()).branch(image_schema());

        dialogue::enter::<Update, ErasedStorage<State>, State, _>()
            .branch(unauth_command_handler)
            .branch(authenticated)
    }

    /// Runs the StableDiffusionBot
    pub async fn run(self) -> anyhow::Result<()> {
        let StableDiffusionBot {
            bot,
            storage,
            config,
        } = self;

        bot.set_my_commands(UnauthenticatedCommands::bot_commands())
            .scope(teloxide::types::BotCommandScope::Default)
            .await?;

        bot.set_my_commands(SettingsCommands::bot_commands())
            .scope(teloxide::types::BotCommandScope::Default)
            .await?;

        Dispatcher::builder(bot, Self::schema())
            .dependencies(dptree::deps![config, storage])
            .default_handler(|upd| async move {
                warn!("Unhandled update: {:?}", upd);
            })
            .error_handler(LoggingErrorHandler::with_custom_text(
                "An error has occurred in the dispatcher",
            ))
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ConfigParameters {
    allowed_users: HashSet<UserId>,
    api: Api,
    txt2img_defaults: Txt2ImgRequest,
    img2img_defaults: Img2ImgRequest,
}

/// Struct that builds a StableDiffusionBot instance.
pub struct StableDiffusionBotBuilder {
    api_key: String,
    allowed_users: Vec<u64>,
    db_path: Option<String>,
    sd_api_url: String,
    txt2img_defaults: Option<Txt2ImgRequest>,
    img2img_defaults: Option<Img2ImgRequest>,
}

impl StableDiffusionBotBuilder {
    /// Constructor that returns a new StableDiffusionBotBuilder instance.
    pub fn new(api_key: String, allowed_users: Vec<u64>, sd_api_url: String) -> Self {
        StableDiffusionBotBuilder {
            api_key,
            allowed_users,
            db_path: None,
            sd_api_url,
            txt2img_defaults: None,
            img2img_defaults: None,
        }
    }

    /// Builder function that sets the path of the storage database for the bot.
    ///
    /// # Arguments
    ///
    /// * `path` - An optional `String` representing the path to the storage database.
    ///
    /// # Examples
    ///
    /// ```
    /// let builder = StableDiffusionBotBuilder::new(api_key, allowed_users, sd_api_url);
    ///
    /// let bot = builder.db_path(Some("database.sqlite".to_string())).build().await.unwrap();
    /// ```
    pub fn db_path(mut self, path: Option<String>) -> Self {
        self.db_path = path;
        self
    }

    /// Builder function that sets the defaults for text to image requests.
    ///
    /// # Arguments
    ///
    /// * `request` - An optional `Txt2ImgRequest` representing the default settings for text to image conversion.
    ///
    /// # Examples
    ///
    /// ```
    /// let builder = StableDiffusionBotBuilder::new(api_key, allowed_users, sd_api_url);
    ///
    /// let bot = builder.txt2img_defaults(Some(txt2img_request)).build().await.unwrap();
    /// ```
    pub fn txt2img_defaults(mut self, request: Option<Txt2ImgRequest>) -> Self {
        self.txt2img_defaults = request;
        self
    }

    /// Builder function that sets the defaults for image to image requests.
    ///
    /// # Arguments
    ///
    /// * `request` - An optional `Img2ImgRequest` representing the default settings for image to image conversion.
    ///
    /// # Examples
    ///
    /// ```
    /// let builder = StableDiffusionBotBuilder::new(api_key, allowed_users, sd_api_url);
    ///
    /// let bot = builder.img2img_defaults(Some(img2img_request)).build().await.unwrap();
    /// ```
    pub fn img2img_defaults(mut self, request: Option<Img2ImgRequest>) -> Self {
        self.img2img_defaults = request;
        self
    }

    /// Consumes the builder and builds a `StableDiffusionBot` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// let builder = StableDiffusionBotBuilder::new(api_key, allowed_users, sd_api_url);
    ///
    /// let bot = builder.build().await.unwrap();
    /// ```
    pub async fn build(self) -> anyhow::Result<StableDiffusionBot> {
        let storage: DialogueStorage = if let Some(path) = self.db_path {
            SqliteStorage::open(&path, Json)
                .await
                .context("failed to open db")?
                .erase()
        } else {
            InMemStorage::new().erase()
        };

        let bot = Bot::new(self.api_key.clone());

        let allowed_users = self.allowed_users.into_iter().map(UserId).collect();

        let client = reqwest::Client::new();

        let api = Api::new_with_client_and_url(client, self.sd_api_url.clone())
            .context("Failed to initialize sd api")?;

        let parameters = ConfigParameters {
            allowed_users,
            api,
            txt2img_defaults: self.txt2img_defaults.unwrap_or_else(default_txt2img),
            img2img_defaults: self.img2img_defaults.unwrap_or_else(default_img2img),
        };

        Ok(StableDiffusionBot {
            bot,
            storage,
            config: parameters,
        })
    }
}
