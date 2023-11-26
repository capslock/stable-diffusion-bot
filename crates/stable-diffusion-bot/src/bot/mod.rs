use std::{collections::HashSet, sync::Arc};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use teloxide::{
    dispatching::{
        dialogue::{
            serializer::Json, ErasedStorage, GetChatId, InMemStorage, SqliteStorage, Storage,
        },
        DpHandlerDescription, UpdateHandler,
    },
    prelude::*,
    types::Update,
    utils::command::BotCommands,
};
use tracing::{error, warn};

use stable_diffusion_api::{Api, Img2ImgRequest, Txt2ImgRequest};

mod handlers;
mod helpers;
use handlers::*;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub enum State {
    #[default]
    New,
    Ready {
        bot_state: BotState,
        txt2img: Txt2ImgRequest,
        img2img: Img2ImgRequest,
    },
}

impl State {
    fn new_with_defaults(txt2img: Txt2ImgRequest, img2img: Img2ImgRequest) -> Self {
        Self::Ready {
            txt2img,
            img2img,
            bot_state: BotState::Generate,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub enum BotState {
    #[default]
    Generate,
    SettingsTxt2Img {
        selection: Option<String>,
    },
    SettingsImg2Img {
        selection: Option<String>,
    },
}

fn default_txt2img(txt2img: Txt2ImgRequest) -> Txt2ImgRequest {
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
    .merge(txt2img)
}

fn default_img2img(img2img: Img2ImgRequest) -> Img2ImgRequest {
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
        ..Default::default()
    }
    .merge(img2img)
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
        Self::enter::<Update, ErasedStorage<State>, _>()
            .branch(unauth_command_handler())
            .branch(authenticated_command_handler())
    }

    // Borrowed and adapted from Teloxide's `dialogue::enter()` function.
    // Instead of building a default dialogue if one doesn't exist via `get_or_default()`,
    // we build a dialogue with the defaults that are defined in the `ConfigParameters`.
    fn enter<Upd, S, Output>() -> Handler<'static, DependencyMap, Output, DpHandlerDescription>
    where
        S: Storage<State> + ?Sized + Send + Sync + 'static,
        <S as Storage<State>>::Error: std::fmt::Debug + Send,
        Upd: GetChatId + Clone + Send + Sync + 'static,
        Output: Send + Sync + 'static,
    {
        dptree::filter_map(|storage: Arc<S>, upd: Upd| {
            let chat_id = upd.chat_id()?;
            Some(Dialogue::new(storage, chat_id))
        })
        .filter_map_async(
            |dialogue: Dialogue<State, S>, cfg: ConfigParameters| async move {
                match dialogue.get().await {
                    Ok(dialogue) => Some(dialogue.unwrap_or(State::new_with_defaults(
                        cfg.txt2img_defaults,
                        cfg.img2img_defaults,
                    ))),
                    Err(err) => {
                        error!("dialogue.get() failed: {:?}", err);
                        None
                    }
                }
            },
        )
    }

    /// Runs the StableDiffusionBot
    pub async fn run(self) -> anyhow::Result<()> {
        let StableDiffusionBot {
            bot,
            storage,
            config,
        } = self;

        let mut commands = UnauthenticatedCommands::bot_commands();
        commands.extend(SettingsCommands::bot_commands());
        commands.extend(GenCommands::bot_commands());
        bot.set_my_commands(commands)
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

#[derive(Clone, Debug, Default)]
pub(crate) struct ConfigParameters {
    allowed_users: HashSet<ChatId>,
    api: Api,
    txt2img_defaults: Txt2ImgRequest,
    img2img_defaults: Img2ImgRequest,
    allow_all_users: bool,
}

impl ConfigParameters {
    /// Checks whether a chat is allowed by the config.
    pub fn chat_is_allowed(&self, chat_id: &ChatId) -> bool {
        self.allow_all_users || self.allowed_users.contains(chat_id)
    }
}

/// Struct that builds a StableDiffusionBot instance.
pub struct StableDiffusionBotBuilder {
    api_key: String,
    allowed_users: Vec<i64>,
    db_path: Option<String>,
    sd_api_url: String,
    txt2img_defaults: Option<Txt2ImgRequest>,
    img2img_defaults: Option<Img2ImgRequest>,
    allow_all_users: bool,
}

impl StableDiffusionBotBuilder {
    /// Constructor that returns a new StableDiffusionBotBuilder instance.
    pub fn new(
        api_key: String,
        allowed_users: Vec<i64>,
        sd_api_url: String,
        allow_all_users: bool,
    ) -> Self {
        StableDiffusionBotBuilder {
            api_key,
            allowed_users,
            db_path: None,
            sd_api_url,
            txt2img_defaults: None,
            img2img_defaults: None,
            allow_all_users,
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
    /// # use stable_diffusion_bot::StableDiffusionBotBuilder;
    /// # let api_key = "api_key".to_string();
    /// # let allowed_users = vec![1, 2, 3];
    /// # let sd_api_url = "http://localhost:7860".to_string();
    /// # let allow_all_users = false;
    /// # tokio_test::block_on(async {
    /// let builder = StableDiffusionBotBuilder::new(api_key, allowed_users, sd_api_url, allow_all_users);
    ///
    /// let bot = builder.db_path(Some("database.sqlite".to_string())).build().await.unwrap();
    /// # });
    /// ```
    pub fn db_path(mut self, path: Option<String>) -> Self {
        self.db_path = path;
        self
    }

    /// Builder function that sets the defaults for text to image requests.
    ///
    /// # Arguments
    ///
    /// * `request` - A `Txt2ImgRequest` representing the default settings for text to image conversion.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stable_diffusion_bot::StableDiffusionBotBuilder;
    /// # use stable_diffusion_api::Txt2ImgRequest;
    /// # let api_key = "api_key".to_string();
    /// # let allowed_users = vec![1, 2, 3];
    /// # let sd_api_url = "http://localhost:7860".to_string();
    /// # let allow_all_users = false;
    /// # tokio_test::block_on(async {
    /// let builder = StableDiffusionBotBuilder::new(api_key, allowed_users, sd_api_url, allow_all_users);
    ///
    /// let bot = builder.txt2img_defaults(Txt2ImgRequest::default()).build().await.unwrap();
    /// # });
    /// ```
    pub fn txt2img_defaults(mut self, request: Txt2ImgRequest) -> Self {
        self.txt2img_defaults = Some(self.txt2img_defaults.unwrap_or_default().merge(request));
        self
    }

    /// Builder function that clears the defaults for text to image requests.
    pub fn clear_txt2img_defaults(mut self) -> Self {
        self.txt2img_defaults = None;
        self
    }

    /// Builder function that sets the defaults for image to image requests.
    ///
    /// # Arguments
    ///
    /// * `request` - An `Img2ImgRequest` representing the default settings for image to image conversion.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stable_diffusion_bot::StableDiffusionBotBuilder;
    /// # use stable_diffusion_api::Img2ImgRequest;
    /// # let api_key = "api_key".to_string();
    /// # let allowed_users = vec![1, 2, 3];
    /// # let sd_api_url = "http://localhost:7860".to_string();
    /// # let allow_all_users = false;
    /// # tokio_test::block_on(async {
    /// let builder = StableDiffusionBotBuilder::new(api_key, allowed_users, sd_api_url, allow_all_users);
    ///
    /// let bot = builder.img2img_defaults(Img2ImgRequest::default()).build().await.unwrap();
    /// # });
    /// ```
    pub fn img2img_defaults(mut self, request: Img2ImgRequest) -> Self {
        self.img2img_defaults = Some(self.img2img_defaults.unwrap_or_default().merge(request));
        self
    }

    /// Builder function that clears the defaults for image to image requests.
    pub fn clear_img2img_defaults(mut self) -> Self {
        self.img2img_defaults = None;
        self
    }

    /// Consumes the builder and builds a `StableDiffusionBot` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stable_diffusion_bot::StableDiffusionBotBuilder;
    /// # let api_key = "api_key".to_string();
    /// # let allowed_users = vec![1, 2, 3];
    /// # let sd_api_url = "http://localhost:7860".to_string();
    /// # let allow_all_users = false;
    /// # tokio_test::block_on(async {
    /// let builder = StableDiffusionBotBuilder::new(api_key, allowed_users, sd_api_url, allow_all_users);
    ///
    /// let bot = builder.build().await.unwrap();
    /// # });
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

        let allowed_users = self.allowed_users.into_iter().map(ChatId).collect();

        let client = reqwest::Client::new();

        let api = Api::new_with_client_and_url(client, self.sd_api_url.clone())
            .context("Failed to initialize sd api")?;

        let parameters = ConfigParameters {
            allowed_users,
            api,
            txt2img_defaults: default_txt2img(self.txt2img_defaults.unwrap_or_default()),
            img2img_defaults: default_img2img(self.img2img_defaults.unwrap_or_default()),
            allow_all_users: self.allow_all_users,
        };

        Ok(StableDiffusionBot {
            bot,
            storage,
            config: parameters,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use stable_diffusion_api::{Img2ImgRequest, Txt2ImgRequest};

    #[tokio::test]
    async fn test_stable_diffusion_bot_builder() {
        let api_key = "api_key".to_string();
        let sd_api_url = "http://localhost:7860".to_string();
        let allowed_users = vec![1, 2, 3];
        let allow_all_users = false;

        let builder =
            StableDiffusionBotBuilder::new(api_key, allowed_users, sd_api_url, allow_all_users);

        let bot = builder
            .db_path(Some("database.sqlite".to_string()))
            .build()
            .await
            .unwrap();

        assert_eq!(bot.config.allowed_users.len(), 3);
        assert!(!bot.config.allow_all_users);
        assert_eq!(bot.config.txt2img_defaults.width, Some(512));
        assert_eq!(bot.config.img2img_defaults.width, Some(512));
    }

    #[tokio::test]
    async fn test_stable_diffusion_bot_defaults() {
        let api_key = "api_key".to_string();
        let sd_api_url = "http://localhost:7860".to_string();
        let allowed_users = vec![1, 2, 3];
        let allow_all_users = false;

        let builder = StableDiffusionBotBuilder::new(
            api_key.clone(),
            allowed_users.clone(),
            sd_api_url.clone(),
            allow_all_users,
        );

        let bot = builder.build().await.unwrap();

        assert_eq!(
            bot.config.allowed_users,
            allowed_users.into_iter().map(ChatId).collect()
        );
        assert_eq!(bot.config.allow_all_users, allow_all_users);
        assert_eq!(
            bot.config.txt2img_defaults,
            default_txt2img(Txt2ImgRequest::default())
        );
        assert_eq!(
            bot.config.img2img_defaults,
            default_img2img(Img2ImgRequest::default())
        );
    }

    #[tokio::test]
    async fn test_stable_diffusion_bot_user_defaults() {
        let api_key = "api_key".to_string();
        let sd_api_url = "http://localhost:7860".to_string();
        let allowed_users = vec![1, 2, 3];
        let allow_all_users = false;
        let txt2img_settings = Txt2ImgRequest {
            width: Some(1024),
            height: Some(768),
            ..Default::default()
        };
        let img2img_settings = Img2ImgRequest {
            width: Some(1024),
            height: Some(768),
            ..Default::default()
        };

        let builder = StableDiffusionBotBuilder::new(
            api_key.clone(),
            allowed_users.clone(),
            sd_api_url.clone(),
            allow_all_users,
        );

        let bot = builder
            .txt2img_defaults(txt2img_settings.clone())
            .img2img_defaults(img2img_settings.clone())
            .build()
            .await
            .unwrap();

        assert_eq!(
            bot.config.allowed_users,
            allowed_users.into_iter().map(ChatId).collect()
        );
        assert_eq!(bot.config.allow_all_users, allow_all_users);
        assert_eq!(
            bot.config.txt2img_defaults,
            default_txt2img(txt2img_settings)
        );
        assert_eq!(
            bot.config.img2img_defaults,
            default_img2img(img2img_settings)
        );
    }

    #[tokio::test]
    async fn test_stable_diffusion_bot_user_and_default() {
        let api_key = "api_key".to_string();
        let sd_api_url = "http://localhost:7860".to_string();
        let allowed_users = vec![1, 2, 3];
        let allow_all_users = false;

        let builder = StableDiffusionBotBuilder::new(
            api_key.clone(),
            allowed_users.clone(),
            sd_api_url.clone(),
            allow_all_users,
        );

        let bot = builder
            .txt2img_defaults(Txt2ImgRequest {
                width: Some(1024),
                height: Some(768),
                ..Default::default()
            })
            .img2img_defaults(Img2ImgRequest {
                width: Some(1024),
                height: Some(768),
                ..Default::default()
            })
            .txt2img_defaults(Txt2ImgRequest {
                width: Some(512),
                ..Default::default()
            })
            .img2img_defaults(Img2ImgRequest {
                width: Some(512),
                ..Default::default()
            })
            .build()
            .await
            .unwrap();

        assert_eq!(
            bot.config.allowed_users,
            allowed_users.into_iter().map(ChatId).collect()
        );
        assert_eq!(bot.config.allow_all_users, allow_all_users);
        assert_eq!(
            bot.config.txt2img_defaults,
            default_txt2img(Txt2ImgRequest {
                width: Some(512),
                height: Some(768),
                ..Default::default()
            })
        );
        assert_eq!(
            bot.config.img2img_defaults,
            default_img2img(Img2ImgRequest {
                width: Some(512),
                height: Some(768),
                ..Default::default()
            })
        );
    }

    #[tokio::test]
    async fn test_stable_diffusion_bot_no_user_defaults() {
        let api_key = "api_key".to_string();
        let sd_api_url = "http://localhost:7860".to_string();
        let allowed_users = vec![1, 2, 3];
        let allow_all_users = false;

        let builder = StableDiffusionBotBuilder::new(
            api_key.clone(),
            allowed_users.clone(),
            sd_api_url.clone(),
            allow_all_users,
        );

        let bot = builder
            .txt2img_defaults(Txt2ImgRequest {
                width: Some(1024),
                height: Some(768),
                ..Default::default()
            })
            .img2img_defaults(Img2ImgRequest {
                width: Some(1024),
                height: Some(768),
                ..Default::default()
            })
            .clear_txt2img_defaults()
            .clear_img2img_defaults()
            .build()
            .await
            .unwrap();

        assert_eq!(
            bot.config.allowed_users,
            allowed_users.into_iter().map(ChatId).collect()
        );
        assert_eq!(bot.config.allow_all_users, allow_all_users);
        assert_eq!(
            bot.config.txt2img_defaults,
            default_txt2img(Txt2ImgRequest::default())
        );
        assert_eq!(
            bot.config.img2img_defaults,
            default_img2img(Img2ImgRequest::default())
        );
    }
}
