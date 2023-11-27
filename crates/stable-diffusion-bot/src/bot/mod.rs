use std::{collections::HashSet, path::PathBuf, sync::Arc};

use anyhow::{anyhow, Context};
use comfyui_api::comfy::getter::{LoadImageExt, PromptExt, SeedExt};
use sal_e_api::{ComfyPromptApi, GenParams, Img2ImgApi, StableDiffusionWebUiApi, Txt2ImgApi};
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
use tokio::fs::File;
use tokio::io::AsyncReadExt;
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
        txt2img: Box<dyn GenParams>,
        img2img: Box<dyn GenParams>,
    },
}

impl State {
    fn new_with_defaults(txt2img: Box<dyn GenParams>, img2img: Box<dyn GenParams>) -> Self {
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
        seed: Some(-1),
        sampler_index: Some("Euler".to_owned()),
        batch_size: Some(1),
        n_iter: Some(1),
        steps: Some(50),
        cfg_scale: Some(7.0),
        width: Some(512),
        height: Some(512),
        negative_prompt: Some("".to_owned()),
        ..Default::default()
    }
    .merge(txt2img)
}

fn default_img2img(img2img: Img2ImgRequest) -> Img2ImgRequest {
    Img2ImgRequest {
        denoising_strength: Some(0.75),
        seed: Some(-1),
        sampler_index: Some("Euler".to_owned()),
        batch_size: Some(1),
        n_iter: Some(1),
        steps: Some(50),
        cfg_scale: Some(7.0),
        width: Some(512),
        height: Some(512),
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
                    Ok(dialogue) => {
                        let mut dialogue = if let Some(dialogue) = dialogue {
                            dialogue
                        } else {
                            return Some(State::new_with_defaults(
                                cfg.txt2img_api.gen_params(None),
                                cfg.img2img_api.gen_params(None),
                            ));
                        };
                        match dialogue {
                            State::New => {}
                            State::Ready {
                                ref mut txt2img,
                                ref mut img2img,
                                ..
                            } => {
                                let txt2img_params = cfg.txt2img_api.gen_params(None);
                                if txt2img.as_any().type_id() != txt2img_params.as_any().type_id() {
                                    warn!("txt2img settings type mismatch, resetting to default");
                                    *txt2img = txt2img_params;
                                } else {
                                    *txt2img = cfg.txt2img_api.gen_params(Some(txt2img.as_ref()));
                                }
                                let img2img_params = cfg.img2img_api.gen_params(None);
                                if img2img.as_any().type_id() != img2img_params.as_any().type_id() {
                                    warn!("img2img settings type mismatch, resetting to default");
                                    *img2img = img2img_params;
                                } else {
                                    *img2img = cfg.img2img_api.gen_params(Some(img2img.as_ref()));
                                }
                            }
                        }
                        Some(dialogue)
                    }
                    Err(err) => {
                        error!("dialogue.get() failed: {:?}", err);
                        let defaults = State::new_with_defaults(
                            cfg.txt2img_api.gen_params(None),
                            cfg.img2img_api.gen_params(None),
                        );
                        match dialogue.update(defaults.clone()).await {
                            Ok(_) => {
                                warn!("dialogue reset to default state: {:?}", defaults);
                                Some(defaults)
                            }
                            Err(err) => {
                                error!("dialogue.update() failed: {:?}", err);
                                None
                            }
                        }
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
pub struct SettingsParameters {
    pub disable_user_settings: bool,
}

#[derive(Clone, Debug, Default)]
pub struct UiParameters {
    pub hide_rerun_button: bool,
    pub hide_reuse_button: bool,
    pub hide_settings_button: bool,
    pub hide_all_buttons: bool,
}

#[derive(Clone, Debug, Default)]
pub struct MessageParameters {
    pub hide_generation_info: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct ConfigParameters {
    allowed_users: HashSet<ChatId>,
    txt2img_api: Box<dyn sal_e_api::Txt2ImgApi>,
    img2img_api: Box<dyn sal_e_api::Img2ImgApi>,
    allow_all_users: bool,
    administrator_users: HashSet<ChatId>,
    settings: SettingsParameters,
    ui: UiParameters,
    messages: MessageParameters,
}

impl ConfigParameters {
    /// Checks whether a chat is allowed by the config.
    pub fn chat_is_allowed(&self, chat_id: &ChatId) -> bool {
        self.allow_all_users || self.allowed_users.contains(chat_id)
    }

    /// Checks whether a user is an admin by the config.
    pub fn user_is_admin(&self, chat_id: &ChatId) -> bool {
        self.administrator_users.contains(chat_id)
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub enum ApiType {
    ComfyUI,
    #[default]
    StableDiffusionWebUi,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ComfyUIConfig {
    pub txt2img_prompt_file: Option<PathBuf>,
    pub img2img_prompt_file: Option<PathBuf>,
}

/// Struct that builds a StableDiffusionBot instance.
pub struct StableDiffusionBotBuilder {
    api_key: String,
    allowed_users: Vec<i64>,
    db_path: Option<String>,
    sd_api_url: String,
    api_type: ApiType,
    txt2img_defaults: Option<Txt2ImgRequest>,
    img2img_defaults: Option<Img2ImgRequest>,
    comfyui_img2img_prompt_file: Option<PathBuf>,
    comfyui_txt2img_prompt_file: Option<PathBuf>,
    allow_all_users: bool,
    administrator_users: Vec<i64>,
    settings: SettingsParameters,
    ui: UiParameters,
    messages: MessageParameters,
}

impl StableDiffusionBotBuilder {
    /// Constructor that returns a new StableDiffusionBotBuilder instance.
    pub fn new(
        api_key: String,
        allowed_users: Vec<i64>,
        sd_api_url: String,
        api_type: ApiType,
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
            api_type,
            comfyui_txt2img_prompt_file: None,
            comfyui_img2img_prompt_file: None,
            administrator_users: Vec::new(),
            settings: SettingsParameters::default(),
            ui: UiParameters::default(),
            messages: MessageParameters::default(),
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
    /// ```ignore
    /// # use stable_diffusion_bot::StableDiffusionBotBuilder;
    /// # let api_key = "api_key".to_string();
    /// # let allowed_users = vec![1, 2, 3];
    /// # let sd_api_url = "http://localhost:7860".to_string();
    /// # let allow_all_users = false;
    /// # tokio_test::block_on(async {
    /// let builder = StableDiffusionBotBuilder::new(api_key, allowed_users, sd_api_url);
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
    /// # let api_type = stable_diffusion_bot::ApiType::StableDiffusionWebUi;
    /// # tokio_test::block_on(async {
    /// let builder = StableDiffusionBotBuilder::new(api_key, allowed_users, sd_api_url, api_type, allow_all_users);
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
    /// # let api_type = stable_diffusion_bot::ApiType::StableDiffusionWebUi;
    /// # tokio_test::block_on(async {
    /// let builder = StableDiffusionBotBuilder::new(api_key, allowed_users, sd_api_url, api_type, allow_all_users);
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

    pub fn comfyui_config(
        mut self,
        ComfyUIConfig {
            txt2img_prompt_file,
            img2img_prompt_file,
        }: ComfyUIConfig,
    ) -> Self {
        self.comfyui_txt2img_prompt_file = txt2img_prompt_file;
        self.comfyui_img2img_prompt_file = img2img_prompt_file;
        self
    }

    /// Builder function that sets whether all users are allowed to use the bot.
    pub fn allow_all_users(mut self, allow_all_users: bool) -> Self {
        self.allow_all_users = allow_all_users;
        self
    }

    /// Builder function that sets the settings configuration for the bot.
    ///
    /// # Arguments
    ///
    /// * `settings` - A `SettingsParameters` struct representing the settings configuration for the bot.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stable_diffusion_bot::StableDiffusionBotBuilder;
    /// # use stable_diffusion_bot::SettingsParameters;
    /// #
    /// # let api_key = "api_key".to_string();
    /// # let allowed_users = vec![1, 2, 3];
    /// # let sd_api_url = "http://localhost:7860".to_string();
    /// # let allow_all_users = false;
    /// # tokio_test::block_on(async {
    /// let builder = StableDiffusionBotBuilder::new(api_key, allowed_users, sd_api_url);
    ///
    /// let bot = builder.configure_settings(SettingsParameters::default()).build().await.unwrap();
    /// # });
    /// ```
    pub fn configure_settings(mut self, settings: SettingsParameters) -> Self {
        self.settings = settings;
        self
    }

    /// Builder function that sets the UI configuration for the bot.
    ///
    /// # Arguments
    ///
    /// * `ui` - A `UiParameters` struct representing the UI configuration for the bot.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stable_diffusion_bot::StableDiffusionBotBuilder;
    /// # use stable_diffusion_bot::UiParameters;
    /// #
    /// # let api_key = "api_key".to_string();
    /// # let allowed_users = vec![1, 2, 3];
    /// # let sd_api_url = "http://localhost:7860".to_string();
    /// # let allow_all_users = false;
    /// # tokio_test::block_on(async {
    /// let builder = StableDiffusionBotBuilder::new(api_key, allowed_users, sd_api_url);
    ///
    /// let bot = builder.configure_ui(UiParameters::default()).build().await.unwrap();
    /// # });
    /// ```
    pub fn configure_ui(mut self, ui: UiParameters) -> Self {
        self.ui = ui;
        self
    }

    /// Builder function that sets the messages configuration for the bot.
    ///
    /// # Arguments
    ///
    /// * `messages` - A `MessageParameters` struct representing the messages configuration for the bot.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stable_diffusion_bot::StableDiffusionBotBuilder;
    /// # use stable_diffusion_bot::MessageParameters;
    /// #
    /// # let api_key = "api_key".to_string();
    /// # let allowed_users = vec![1, 2, 3];
    /// # let sd_api_url = "http://localhost:7860".to_string();
    /// # let allow_all_users = false;
    /// # tokio_test::block_on(async {
    /// let builder = StableDiffusionBotBuilder::new(api_key, allowed_users, sd_api_url);
    ///
    /// let bot = builder.configure_messages(MessageParameters::default()).build().await.unwrap();
    /// # });
    /// ```
    pub fn configure_messages(mut self, messages: MessageParameters) -> Self {
        self.messages = messages;
        self
    }

    /// Builder function that sets the administrator users for the bot.
    ///
    /// # Arguments
    ///
    /// * `administrator_users` - A `Vec<i64>` representing the administrator users for the bot.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stable_diffusion_bot::StableDiffusionBotBuilder;
    /// #
    /// # let api_key = "api_key".to_string();
    /// # let allowed_users = vec![1, 2, 3];
    /// # let sd_api_url = "http://localhost:7860".to_string();
    /// # let allow_all_users = false;
    /// # tokio_test::block_on(async {
    /// let builder = StableDiffusionBotBuilder::new(api_key, allowed_users, sd_api_url);
    ///
    /// let bot = builder.administrator_users(vec![1, 2, 3]).build().await.unwrap();
    /// # });
    pub fn administrator_users(mut self, administrator_users: Vec<i64>) -> Self {
        self.administrator_users = administrator_users;
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
    /// # let api_type = stable_diffusion_bot::ApiType::StableDiffusionWebUi;
    /// # tokio_test::block_on(async {
    /// let builder = StableDiffusionBotBuilder::new(api_key, allowed_users, sd_api_url, api_type, allow_all_users);
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
        let administrator_users = self.administrator_users.into_iter().map(ChatId).collect();

        let client = reqwest::Client::new();

        let (txt2img_api, img2img_api): (Box<dyn Txt2ImgApi>, Box<dyn Img2ImgApi>) = match self
            .api_type
        {
            ApiType::ComfyUI => {
                let mut txt2img_prompt = String::new();

                File::open(
                    self.comfyui_txt2img_prompt_file
                        .ok_or_else(|| anyhow!("No ComfyUI prompt file provided."))?,
                )
                .await
                .context("Failed to open comfyui prompt file")?
                .read_to_string(&mut txt2img_prompt)
                .await?;

                let mut img2img_prompt = String::new();

                File::open(
                    self.comfyui_img2img_prompt_file
                        .ok_or_else(|| anyhow!("No ComfyUI prompt file provided."))?,
                )
                .await
                .context("Failed to open comfyui prompt file")?
                .read_to_string(&mut img2img_prompt)
                .await?;

                let txt2img_prompt =
                    serde_json::from_str::<comfyui_api::models::Prompt>(&txt2img_prompt)
                        .context("Failed to deserialize prompt")?;

                _ = txt2img_prompt
                    .prompt()
                    .context("Failed to find a valid txt2img prompt node.")?;
                _ = txt2img_prompt
                    .seed()
                    .context("Failed to find a valid txt2img seed node.")?;

                let txt2img_api = ComfyPromptApi::new_with_client_and_url(
                    client.clone(),
                    self.sd_api_url.clone(),
                    txt2img_prompt,
                )?;

                let img2img_prompt =
                    serde_json::from_str::<comfyui_api::models::Prompt>(&img2img_prompt)
                        .context("Failed to deserialize prompt")?;

                _ = img2img_prompt
                    .prompt()
                    .context("Failed to find a valid img2img prompt node.")?;
                _ = img2img_prompt
                    .image()
                    .context("Failed to find a valid img2img image node.")?;
                _ = img2img_prompt
                    .seed()
                    .context("Failed to find a valid img2img seed node.")?;

                let img2img_api = ComfyPromptApi::new_with_client_and_url(
                    client,
                    self.sd_api_url,
                    img2img_prompt,
                )?;
                (Box::new(txt2img_api), Box::new(img2img_api))
            }
            ApiType::StableDiffusionWebUi => {
                let api = Api::new_with_client_and_url(client, self.sd_api_url)
                    .context("Failed to initialize sd api")?;
                let txt2img_api = StableDiffusionWebUiApi {
                    client: api.clone(),
                    txt2img_defaults: default_txt2img(
                        self.txt2img_defaults.clone().unwrap_or_default(),
                    ),
                    img2img_defaults: default_img2img(
                        self.img2img_defaults.clone().unwrap_or_default(),
                    ),
                };

                let img2img_api = StableDiffusionWebUiApi {
                    client: api,
                    txt2img_defaults: default_txt2img(self.txt2img_defaults.unwrap_or_default()),
                    img2img_defaults: default_img2img(self.img2img_defaults.unwrap_or_default()),
                };

                (Box::new(txt2img_api), Box::new(img2img_api))
            }
        };

        let parameters = ConfigParameters {
            allowed_users,
            txt2img_api,
            img2img_api,
            allow_all_users: self.allow_all_users,
            administrator_users,
            settings: self.settings,
            ui: self.ui,
            messages: self.messages,
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
        let api_type = ApiType::StableDiffusionWebUi;

        let builder = StableDiffusionBotBuilder::new(
            api_key,
            allowed_users,
            sd_api_url,
            api_type,
            allow_all_users,
        );

        let bot = builder
            .db_path(Some("database.sqlite".to_string()))
            .build()
            .await
            .unwrap();

        assert_eq!(bot.config.allowed_users.len(), 3);
        assert!(!bot.config.allow_all_users);
    }

    #[tokio::test]
    async fn test_stable_diffusion_bot_defaults() {
        let api_key = "api_key".to_string();
        let sd_api_url = "http://localhost:7860".to_string();
        let allowed_users = vec![1, 2, 3];
        let allow_all_users = false;
        let api_type = ApiType::StableDiffusionWebUi;

        let builder = StableDiffusionBotBuilder::new(
            api_key.clone(),
            allowed_users.clone(),
            sd_api_url.clone(),
            api_type,
            allow_all_users,
        );

        let bot = builder.build().await.unwrap();

        assert_eq!(
            bot.config.allowed_users,
            allowed_users.into_iter().map(ChatId).collect()
        );
        assert!(!bot.config.allow_all_users);
        assert_eq!(
            bot.config
                .txt2img_api
                .as_any()
                .downcast_ref::<StableDiffusionWebUiApi>()
                .unwrap()
                .txt2img_defaults,
            default_txt2img(Txt2ImgRequest::default())
        );
        assert_eq!(
            bot.config
                .img2img_api
                .as_any()
                .downcast_ref::<StableDiffusionWebUiApi>()
                .unwrap()
                .img2img_defaults,
            default_img2img(Img2ImgRequest::default())
        );
    }

    #[tokio::test]
    async fn test_stable_diffusion_bot_user_defaults() {
        let api_key = "api_key".to_string();
        let sd_api_url = "http://localhost:7860".to_string();
        let allowed_users = vec![1, 2, 3];
        let allow_all_users = false;
        let api_type = ApiType::StableDiffusionWebUi;

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
            api_type,
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
        assert!(!bot.config.allow_all_users);
        assert_eq!(
            bot.config
                .txt2img_api
                .as_any()
                .downcast_ref::<StableDiffusionWebUiApi>()
                .unwrap()
                .txt2img_defaults,
            default_txt2img(txt2img_settings)
        );
        assert_eq!(
            bot.config
                .img2img_api
                .as_any()
                .downcast_ref::<StableDiffusionWebUiApi>()
                .unwrap()
                .img2img_defaults,
            default_img2img(img2img_settings)
        );
    }
}
